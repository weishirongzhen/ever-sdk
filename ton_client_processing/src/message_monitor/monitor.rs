use crate::message_monitor::message::{MessageMonitoringParams, MessageMonitoringResult};
use crate::message_monitor::queue::MonitoringQueue;
use crate::sdk_services::{MessageMonitorSdkServices, NetSubscription};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex, RwLock};

/// The main message monitor object.
/// Incorporates and serves all message monitoring queues.
///
pub struct MessageMonitor<Sdk: MessageMonitorSdkServices> {
    /// External SDK services used by message monitor
    sdk: Sdk,
    /// Active queues
    queues: Arc<RwLock<HashMap<String, MonitoringQueue>>>,
    notify_resolved: Arc<tokio::sync::watch::Sender<bool>>,
    listen_resolved: tokio::sync::watch::Receiver<bool>,
    active_subscription: Mutex<Option<NetSubscription>>,
}

#[derive(Deserialize, Serialize, ApiType)]
pub struct MonitoringQueueInfo {
    /// Count of the unresolved messages.
    pub unresolved: u32,
    /// Count of resolved results.
    pub resolved: u32,
}

#[derive(Deserialize, Serialize, ApiType, Copy, Clone)]
pub enum MonitorFetchWaitMode {
    /// If there are no resolved results yet, then monitor awaits for the next resolved result.
    AtLeastOne,

    /// Monitor waits until all unresolved messages will be resolved.
    /// If there are no unresolved messages then monitor will wait.
    All,

    // Monitor does not any awaits even if there are no resolved results yet.
    NoWait,
}

// pub
impl<SdkServices: MessageMonitorSdkServices> MessageMonitor<SdkServices> {
    pub fn new(sdk: SdkServices) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(false);
        Self {
            sdk,
            queues: Arc::new(RwLock::new(HashMap::new())),
            active_subscription: Mutex::new(None),
            notify_resolved: Arc::new(sender),
            listen_resolved: receiver,
        }
    }

    pub async fn monitor_messages(
        &self,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        {
            let mut queues = self.queues.write().unwrap();
            let queue = if let Some(queue) = queues.get_mut(queue) {
                queue
            } else {
                queues.insert(queue.to_string(), MonitoringQueue::new());
                queues.get_mut(queue).unwrap()
            };
            for message in messages {
                queue.add_unresolved(&self.sdk, message)?;
            }
        }
        self.resubscribe().await?;
        Ok(())
    }

    pub async fn fetch_next_monitor_results(
        &self,
        queue: &str,
        wait_mode: MonitorFetchWaitMode,
    ) -> crate::error::Result<Vec<MessageMonitoringResult>> {
        let mut listen_resolved = self.listen_resolved.clone();
        loop {
            if let Some(fetched) = self.fetch_next(queue, wait_mode) {
                return Ok(fetched);
            }
            listen_resolved.changed().await.unwrap();
        }
    }

    pub fn get_queue_info(&self, queue: &str) -> crate::error::Result<MonitoringQueueInfo> {
        let queues = self.queues.read().unwrap();
        let (unresolved, resolved) = if let Some(queue) = queues.get(queue) {
            (queue.unresolved.len() as u32, queue.resolved.len() as u32)
        } else {
            (0, 0)
        };
        Ok(MonitoringQueueInfo {
            unresolved,
            resolved,
        })
    }

    pub fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        let mut queues = self.queues.write().unwrap();
        queues.remove(queue);
        Ok(())
    }
}

// priv
impl<SdkServices: MessageMonitorSdkServices> MessageMonitor<SdkServices> {
    async fn resubscribe(&self) -> crate::error::Result<()> {
        let new_subscription = self.subscribe().await?;
        let old_subscription = {
            mem::replace(
                &mut *self.active_subscription.lock().unwrap(),
                new_subscription,
            )
        };
        if let Some(old_subscription) = old_subscription {
            self.sdk.unsubscribe(old_subscription).await?;
        }
        Ok(())
    }

    async fn subscribe(&self) -> crate::error::Result<Option<NetSubscription>> {
        let messages = self.collect_unresolved();
        if messages.is_empty() {
            return Ok(None);
        }
        let queues = self.queues.clone();
        let notify_resolved = self.notify_resolved.clone();
        let callback = move |results| {
            if let Ok(results) = results {
                for queue in queues.write().unwrap().values_mut() {
                    queue.resolve(&results);
                }
                notify_resolved.send(true).ok();
            }
            async {}
        };
        Ok(Some(
            self.sdk
                .subscribe_for_recent_ext_in_message_statuses(messages, callback)
                .await?,
        ))
    }

    fn collect_unresolved(&self) -> Vec<MessageMonitoringParams> {
        let mut messages = Vec::new();
        for queue in self.queues.read().unwrap().values() {
            for message in queue.unresolved.values() {
                messages.push(message.clone());
            }
        }
        messages
    }

    fn fetch_next(
        &self,
        queue: &str,
        wait_mode: MonitorFetchWaitMode,
    ) -> Option<Vec<MessageMonitoringResult>> {
        let mut queues = self.queues.write().unwrap();
        let (fetched, queue_should_be_removed) = if let Some(queue) = queues.get_mut(queue) {
            let next = queue.fetch_next(wait_mode);
            let should_be_removed = queue.resolved.is_empty() && queue.unresolved.is_empty();
            (next, should_be_removed)
        } else if let MonitorFetchWaitMode::NoWait = wait_mode {
            (Some(vec![]), false)
        } else {
            (None, false)
        };
        if queue_should_be_removed {
            queues.remove(queue);
        }
        fetched
    }
}
