use crate::abi::{Abi, ParamsOfDecodeMessage};
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::{
    Error, DEFAULT_EXPIRATION_RETRIES_LIMIT, DEFAULT_NETWORK_RETRIES_LIMIT,
    DEFAULT_NETWORK_RETRIES_TIMEOUT,
};
use serde_json::Value;
use std::sync::Arc;
use ton_block::Serializable;
use ton_sdk::{Block, MessageId};

pub(crate) fn get_message_id(message: &ton_block::Message) -> ApiResult<String> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

/// Increments `retries` and returns `true` if `retries` isn't reach `limit`.
pub(crate) fn can_retry_more(retries: &mut u8, limit: i8) -> bool {
    *retries = retries.checked_add(1).unwrap_or(*retries);
    limit < 0 || *retries <= limit as u8
}

pub fn can_retry_network_error(context: &Arc<ClientContext>, retries: &mut u8) -> bool {
    can_retry_more(
        retries,
        resolve(
            context.config.network.as_ref(),
            |_| None,
            DEFAULT_NETWORK_RETRIES_LIMIT,
        ),
    )
}

pub fn resolve_network_retries_timeout(context: &Arc<ClientContext>) -> u32 {
    resolve(
        context.config.network.as_ref(),
        |_| None,
        DEFAULT_NETWORK_RETRIES_TIMEOUT,
    )
}

pub(crate) fn can_retry_expired_message(context: &Arc<ClientContext>, retries: &mut u8) -> bool {
    can_retry_more(
        retries,
        resolve(
            context.config.network.as_ref(),
            |x| Some(x.message_retries_count() as i8),
            DEFAULT_EXPIRATION_RETRIES_LIMIT,
        ),
    )
}

fn resolve<C, R>(config: Option<&C>, resolve_cfg: fn(cfg: &C) -> Option<R>, def: R) -> R {
    let cfg = config.map_or(None, |x| resolve_cfg(x));
    cfg.unwrap_or(def)
}

pub fn find_transaction(
    block: &Block,
    message_id: &str,
    shard_block_id: &String,
) -> ApiResult<Option<String>> {
    let msg_id: MessageId = message_id.into();
    for msg_descr in &block.in_msg_descr {
        if Some(&msg_id) == msg_descr.msg_id.as_ref() {
            return Ok(Some(
                msg_descr
                    .transaction_id
                    .as_ref()
                    .ok_or(Error::invalid_block_received(
                        "No field `transaction_id` in block's `in_msg_descr`.",
                        message_id,
                        shard_block_id,
                    ))?
                    .to_string(),
            ));
        }
    }
    Ok(None)
}

#[derive(Deserialize)]
struct ComputePhase {
    exit_code: i32,
}

#[derive(Deserialize)]
struct Transaction {
    compute: ComputePhase,
}

pub(crate) fn get_exit_code(
    parsed_transaction: &Value,
    shard_block_id: &String,
    message_id: &str,
) -> ApiResult<i32> {
    Ok(
        serde_json::from_value::<Transaction>(parsed_transaction.clone())
            .map_err(|err| {
                Error::fetch_transaction_result_failed(
                    format!("Transaction can't be parsed: {}", err),
                    message_id,
                    shard_block_id,
                )
            })?
            .compute
            .exit_code,
    )
}

pub(crate) fn get_message_expiration_time(
    context: Arc<ClientContext>,
    abi: Option<&Abi>,
    message: &str,
) -> ApiResult<Option<u64>> {
    Ok(match abi {
        Some(abi) => crate::abi::decode_message(
            context.clone(),
            ParamsOfDecodeMessage {
                abi: abi.clone(),
                message: message.to_string(),
            },
        )
        .map(|x| x.header)?,
        None => None,
    }
    .as_ref()
    .map_or(None, |x| x.expire)
    .map(|x| x as u64 * 1000))
}
