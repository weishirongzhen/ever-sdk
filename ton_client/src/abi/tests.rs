use crate::abi::decode_message::{DecodedMessageBody, MessageBodyType, ParamsOfDecodeMessage};
use crate::abi::encode_message::{
    CallSet, DeploySet, ParamsOfAttachSignature, ParamsOfEncodeMessage, ResultOfAttachSignature, 
    ResultOfEncodeMessage, ParamsOfEncodeInternalMessage, ResultOfEncodeInternalMessage
};
use crate::abi::internal::{is_empty_pubkey, resolve_pubkey, create_tvc_image};
use crate::abi::{FunctionHeader, ParamsOfDecodeMessageBody, Signer};
use crate::boc::internal::get_boc_hash;
use crate::boc::{parse_message, ParamsOfParse, ResultOfGetCodeFromTvc, ParamsOfGetCodeFromTvc};
use crate::ClientContext;
use crate::crypto::KeyPair;
use crate::error::ClientError;
use crate::tests::{TestClient, EVENTS, HELLO};
use crate::utils::conversion::abi_uint;

use std::io::Cursor;
use std::sync::Arc;

use ton_block::{Message, Deserializable, Serializable};
use ton_sdk::ContractImage;
use ton_types::Result;

use super::*;

#[test]
fn encode_v2() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = KeyPair {
        public: "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into(),
        secret: "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7".into(),
    };
    let abi = events_abi.clone();
    let time: u64 = 1599458364291;
    let expire: u32 = 1599458404;

    let signing_box: crate::crypto::boxes::RegisteredSigningBox = client.request(
        "crypto.get_signing_box",
        keys.clone()
    ).unwrap();

    let msg: ParamsOfEncodeMessage = serde_json::from_str(
        r#"{
        "abi": { "type": "Handle", "value": 0 },
        "signer": {
            "type": "Keys",
            "keys": {
                "public": "4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499",
                "secret": "cc8929d635719612a9478b9cd17675a39cfad52d8959e8a177389b8c0b9122a7"
            }
        }
    }
    "#,
    )
    .unwrap();

    assert_eq!(msg.signer, Signer::Keys { keys: keys.clone() });

    // check deploy params

    let deploy_params = |signing: Signer| ParamsOfEncodeMessage {
        abi: abi.clone(),
        address: None,
        deploy_set: Some(DeploySet {
            tvc: events_tvc.clone(),
            ..Default::default()
        }),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                pubkey: Some(keys.public.clone()),
                time: Some(time),
                expire: Some(expire),
            }),
            input: None,
        }),
        signer: signing,
        processing_try_index: None,
    };

    let unsigned: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::External {
                public_key: keys.public.clone(),
            }),
        )
        .unwrap();
    assert_eq!(unsigned.message, "te6ccgECFwEAA2gAAqeIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEZTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gGAQEBwAICA88gBQMBAd4EAAPQIABB2mPiBH+O713GsgL3S844tQp+62YECSCD0w6eEqy4TKTMAib/APSkICLAAZL0oOGK7VNYMPShCQcBCvSkIPShCAAAAgEgDAoByP9/Ie1E0CDXScIBjhDT/9M/0wDRf/hh+Gb4Y/hijhj0BXABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABjh2BAgDXGCD5AQHTAAGU0/8DAZMC+ELiIPhl+RDyqJXTAAHyeuLTPwELAGqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwH4I7zyudMfAfAB+EdukvI83gIBIBINAgEgDw4AvbqLVfP/hBbo417UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLe+Ebyc3H4ZtH4APhCyMv/+EPPCz/4Rs8LAMntVH/4Z4AgEgERAA5biABrW/CC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb30gyupo6H0gb+j8IpA3SRg4b3whXXlwMnwAZGT9ghBkZ8KEZ0aCBAfQAAAAAAAAAAAAAAAAACBni2TAgEB9gBh8IWRl//wh54Wf/CNnhYBk9qo//DPAAxbmTwqLfCC3Rwn2omhp/+mf6YBov/ww/DN8Mfwxb2uG/8rqaOhp/+/o/ABkRe4AAAAAAAAAAAAAAAAIZ4tnwOfI48sYvRDnhf/kuP2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8AIBSBYTAQm4t8WCUBQB/PhBbo4T7UTQ0//TP9MA0X/4Yfhm+GP4Yt7XDf+V1NHQ0//f0fgAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPkceWMXohzwv/yXH7AMiL3AAAAAAAAAAAAAAAABDPFs+Bz5JW+LBKIc8L/8lx+wAw+ELIy//4Q88LP/hGzwsAye1UfxUABPhnAHLccCLQ1gIx0gAw3CHHAJLyO+Ah1w0fkvI84VMRkvI74cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=");
    assert_eq!(
        unsigned.data_to_sign.as_ref().unwrap(),
        "KCGM36iTYuCYynk+Jnemis+mcwi3RFCke95i7l96s4Q="
    );
    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    assert_eq!(signature, "6272357bccb601db2b821cb0f5f564ab519212d242cf31961fe9a3c50a30b236012618296b4f769355c0e9567cd25b366f3c037435c498c82e5305622adbc70e");
    let signed: ResultOfAttachSignature = client
        .request(
            "abi.attach_signature",
            ParamsOfAttachSignature {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned.message,
                signature,
            },
        )
        .unwrap();
    assert_eq!(signed.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::Keys { keys: keys.clone() }),
        )
        .unwrap();
    assert_eq!(signed.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    let signed_with_box: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            deploy_params(Signer::SigningBox { handle: signing_box.handle.clone() }),
        )
        .unwrap();
    assert_eq!(signed_with_box.message, "te6ccgECGAEAA6wAA0eIAAt9aqvShfTon7Lei1PVOhUEkEEZQkhDKPgNyzeTL6YSEbAHAgEA4bE5Gr3mWwDtlcEOWHr6slWoyQlpIWeYyw/00eKFGFkbAJMMFLWnu0mq4HSrPmktmzeeAboa4kxkFymCsRVt44dTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGRotV8/gAQHAAwIDzyAGBAEB3gUAA9AgAEHaY+IEf47vXcayAvdLzji1Cn7rZgQJIIPTDp4SrLhMpMwCJv8A9KQgIsABkvSg4YrtU1gw9KEKCAEK9KQg9KEJAAACASANCwHI/38h7UTQINdJwgGOENP/0z/TANF/+GH4Zvhj+GKOGPQFcAGAQPQO8r3XC//4YnD4Y3D4Zn/4YeLTAAGOHYECANcYIPkBAdMAAZTT/wMBkwL4QuIg+GX5EPKoldMAAfJ64tM/AQwAao4e+EMhuSCfMCD4I4ED6KiCCBt3QKC53pL4Y+CANPI02NMfAfgjvPK50x8B8AH4R26S8jzeAgEgEw4CASAQDwC9uotV8/+EFujjXtRNAg10nCAY4Q0//TP9MA0X/4Yfhm+GP4Yo4Y9AVwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsAye1Uf/hngCASASEQDluIAGtb8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFvfSDK6mjofSBv6PwikDdJGDhvfCFdeXAyfABkZP2CEGRnwoRnRoIEB9AAAAAAAAAAAAAAAAAAIGeLZMCAQH2AGHwhZGX//CHnhZ/8I2eFgGT2qj/8M8ADFuZPCot8ILdHCfaiaGn/6Z/pgGi//DD8M3wx/DFva4b/yupo6Gn/7+j8AGRF7gAAAAAAAAAAAAAAAAhni2fA58jjyxi9EOeF/+S4/YAYfCFkZf/8IeeFn/wjZ4WAZPaqP/wzwAgFIFxQBCbi3xYJQFQH8+EFujhPtRNDT/9M/0wDRf/hh+Gb4Y/hi3tcN/5XU0dDT/9/R+ADIi9wAAAAAAAAAAAAAAAAQzxbPgc+Rx5YxeiHPC//JcfsAyIvcAAAAAAAAAAAAAAAAEM8Wz4HPklb4sEohzwv/yXH7ADD4QsjL//hDzws/+EbPCwDJ7VR/FgAE+GcActxwItDWAjHSADDcIccAkvI74CHXDR+S8jzhUxGS8jvhwQQighD////9vLGS8jzgAfAB+EdukvI83g==");

    // check run params

    let address = "0:05beb555e942fa744fd96f45a9ea9d0a8248208ca12421947c06e59bc997d309";
    let run_params = |signing: Signer| ParamsOfEncodeMessage {
        address: Some(address.into()),
        abi: abi.clone(),
        deploy_set: None,
        call_set: Some(CallSet {
            function_name: "returnValue".into(),
            header: Some(FunctionHeader {
                pubkey: None,
                time: Some(time),
                expire: Some(expire),
            }),
            input: Some(json!({
                "id": "0"
            })),
        }),
        signer: signing,
        processing_try_index: None,
    };
    let body_params = |run_params: ParamsOfEncodeMessage| {
        ParamsOfEncodeMessageBody {
            abi: run_params.abi,
            call_set: run_params.call_set.unwrap(),
            is_internal: false,
            processing_try_index: run_params.processing_try_index,
            signer: run_params.signer,
        }
    };
    let extract_body = |message| {
        let unsigned_parsed: crate::boc::ResultOfParse = client.request(
            "boc.parse_message",
            crate::boc::ParamsOfParse {
                boc: message
            }).unwrap();
        unsigned_parsed.parsed["body"].as_str().unwrap().to_owned()
    };

    let unsigned_message = "te6ccgEBAgEAeAABpYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIFMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let signed_message = "te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    let data_to_sign = "i4Hs3PB12QA9UBFbOIpkG3JerHHqjm4LgvF4MA7TDsY=";

    // encoding unsigned and attaching the signature

    let unsigned: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::External {
                public_key: keys.public.clone(),
            }),
        )
        .unwrap();
    assert_eq!(unsigned.message, unsigned_message);
    assert_eq!(unsigned.data_to_sign, Some(data_to_sign.to_owned()));

    let unsigned_body = extract_body(unsigned.message.clone());

    let unsigned_body_encoded: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::External {
                public_key: keys.public.clone(),
            })),
        )
        .unwrap();
    assert_eq!(unsigned_body_encoded.body, unsigned_body);
    assert_eq!(unsigned_body_encoded.data_to_sign, unsigned.data_to_sign);

    let signature = client.sign_detached(&unsigned.data_to_sign.unwrap(), &keys);
    assert_eq!(signature, "5bbfb7f184f2cb5f019400b9cd497eeaa41f3d5885619e9f7d4fab8dd695f4b3a02159a1422996c1dd7d1be67898bc79c6adba6c65a18101ac5f0a2a2bb8910b");
    let signed: ResultOfAttachSignature = client
        .request(
            "abi.attach_signature",
            ParamsOfAttachSignature {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned.message,
                signature: signature.clone(),
            },
        )
        .unwrap();
    assert_eq!(signed.message, signed_message);

    let signed_body = extract_body(signed.message);

    let signed: ResultOfAttachSignatureToMessageBody = client
        .request(
            "abi.attach_signature_to_message_body",
            ParamsOfAttachSignatureToMessageBody {
                abi: abi.clone(),
                public_key: keys.public.clone(),
                message: unsigned_body_encoded.body,
                signature,
            },
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    // encoding signed

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::Keys { keys: keys.clone() }),
        )
        .unwrap();
    assert_eq!(signed.message, signed_message);

    let signed: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::Keys { keys: keys.clone() })),
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    let signed: ResultOfEncodeMessage = client
        .request(
            "abi.encode_message",
            run_params(Signer::SigningBox { handle: signing_box.handle.clone() }),
        )
        .unwrap();
    assert_eq!(signed.message, signed_message);

    let signed: ResultOfEncodeMessageBody = client
        .request(
            "abi.encode_message_body",
            body_params(run_params(Signer::SigningBox { handle: signing_box.handle.clone() })),
        )
        .unwrap();
    assert_eq!(signed.body, signed_body);

    let no_pubkey: ResultOfEncodeMessage = client
        .request("abi.encode_message", run_params(Signer::None))
        .unwrap();
    assert_eq!(no_pubkey.message, "te6ccgEBAQEAVQAApYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIAAAAC6M5Llwa+q5jIK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB");

    let no_pubkey_body: ResultOfEncodeMessageBody = client
        .request("abi.encode_message_body", body_params(run_params(Signer::None)))
        .unwrap();
    assert_eq!(no_pubkey_body.body, extract_body(no_pubkey.message));
}

#[test]
fn decode_v2() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, _events_tvc) = TestClient::package(EVENTS, Some(2));

    let decode_events = |message: &str| {
        let result: DecodedMessageBody = client
            .request(
                "abi.decode_message",
                ParamsOfDecodeMessage {
                    abi: events_abi.clone(),
                    message: message.into(),
                },
            )
            .unwrap();
        let parsed: crate::boc::ResultOfParse = client.request(
                "boc.parse_message",
                crate::boc::ParamsOfParse {
                    boc: message.into()
                }).unwrap();
        let body = parsed.parsed["body"].as_str().unwrap().to_owned();
        let result_body: DecodedMessageBody = client
            .request(
                "abi.decode_message_body",
                ParamsOfDecodeMessageBody {
                    abi: events_abi.clone(),
                    body,
                    is_internal: parsed.parsed["msg_type_name"] == "Internal",
                },
            )
            .unwrap();
        assert_eq!(result, result_body);
        result
    };
    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Input,
        name: "returnValue".into(),
        value: Some(json!({
            "id": abi_uint(0, 256),
        })),
        header: Some(FunctionHeader {
            expire: Some(1599458404),
            time: Some(1599458364291),
            pubkey: Some("4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into()),
        }),
    };
    assert_eq!(expected, decode_events("te6ccgEBAwEAvAABRYgAC31qq9KF9Oifst6LU9U6FQSQQRlCSEMo+A3LN5MvphIMAQHhrd/b+MJ5Za+AygBc5qS/dVIPnqxCsM9PvqfVxutK+lnQEKzQoRTLYO6+jfM8TF4841bdNjLQwIDWL4UVFdxIhdMfECP8d3ruNZAXul5xxahT91swIEkEHph08JVlwmUmQAAAXRnJcuDX1XMZBW+LBKACAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="));

    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Event,
        name: "EventThrown".into(),
        value: Some(json!({
            "id": abi_uint(0, 256)
        })),
        header: None,
    };
    assert_eq!(expected, decode_events("te6ccgEBAQEAVQAApeACvg5/pmQpY4m61HmJ0ne+zjHJu3MNG8rJxUDLbHKBu/AAAAAAAAAMJL6z6ro48sYvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABA"));

    let result: DecodedMessageBody = client.request("abi.decode_message_body", ParamsOfDecodeMessageBody {
        abi: events_abi.clone(),
        body: "te6ccgEBAgEAlgAB4a3f2/jCeWWvgMoAXOakv3VSD56sQrDPT76n1cbrSvpZ0BCs0KEUy2Duvo3zPExePONW3TYy0MCA1i+FFRXcSIXTHxAj/Hd67jWQF7peccWoU/dbMCBJBB6YdPCVZcJlJkAAAF0ZyXLg19VzGQVviwSgAQBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        is_internal: false,
    }).unwrap();
    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Input,
        name: "returnValue".into(),
        value: Some(json!({
            "id": abi_uint(0, 256)
        })),
        header: Some(FunctionHeader {
            expire: Some(1599458404),
            time: Some(1599458364291),
            pubkey: Some("4c7c408ff1ddebb8d6405ee979c716a14fdd6cc08124107a61d3c25597099499".into()),
        }),
    };
    assert_eq!(expected, result);

    let expected = DecodedMessageBody {
        body_type: MessageBodyType::Output,
        name: "returnValue".into(),
        value: Some(json!({
            "value0": abi_uint(0, 256)
        })),
        header: None,
    };
    assert_eq!(expected, decode_events("te6ccgEBAQEAVQAApeACvg5/pmQpY4m61HmJ0ne+zjHJu3MNG8rJxUDLbHKBu/AAAAAAAAAMKr6z6rxK3xYJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABA"));
}

#[test]
fn test_is_empty_pubkey() -> Result<()> {
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;

    assert!(is_empty_pubkey(&pubkey));

    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    assert!(!is_empty_pubkey(&pubkey));

    let mut array = [0; 32];
    array[0] = 1;
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&array)?;
    assert!(!is_empty_pubkey(&pubkey));

    Ok(())
}

#[test]
fn test_resolve_pubkey() -> Result<()> {
    let tvc = base64::encode(include_bytes!("../tests/contracts/abi_v2/Hello.tvc"));
    let mut deploy_set = DeploySet {
        tvc: tvc.clone(),
        ..Default::default()
    };
    let mut image = create_tvc_image("", None, &tvc)?;
    assert!(resolve_pubkey(&deploy_set, &image, &None )?.is_none());

    let external_pub_key = Some("1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".to_owned());
    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;
    assert_eq!(resolved, external_pub_key);

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, external_pub_key);

    let tvc_pubkey_empty = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;
    image.set_public_key(&tvc_pubkey_empty)?;

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, external_pub_key);

    let tvc_pubkey_1 = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    image.set_public_key(&tvc_pubkey_1)?;

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, Some(hex::encode(tvc_pubkey_1.as_bytes())));

    let initial_pub_key = Some("1234567890123456789012345678901234567890123456789012345678901234".to_owned());
    deploy_set.initial_pubkey = initial_pub_key.clone();

    let resolved = resolve_pubkey(&deploy_set, &image, &external_pub_key)?;

    assert_eq!(resolved, initial_pub_key);

    Ok(())
}

#[test]
fn test_encode_message_pubkey() -> Result<()> {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package(HELLO, None);

    let initial_pubkey = Some(gen_pubkey());
    let tvc_pubkey = Some(gen_pubkey());
    let signer_pubkey = Some(gen_pubkey());

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &None,
        &None,
        &signer_pubkey,
        &signer_pubkey,
    )?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &None,
        &tvc_pubkey,
        &signer_pubkey,
        &tvc_pubkey,
    )?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &None,
        &signer_pubkey,
        &initial_pubkey,
    )?;

    test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &tvc_pubkey,
        &signer_pubkey,
        &initial_pubkey,
    )?;

    // Expected error, if signer's public key is not provided:
    let error = test_encode_message_pubkey_internal(
        &client,
        &abi,
        &tvc,
        &initial_pubkey,
        &tvc_pubkey,
        &None,
        &None,
    )
        .unwrap_err()
        .downcast::<ClientError>()?;

    assert_eq!(error.code, 305);

    Ok(())
}

fn test_encode_message_pubkey_internal(
    client: &TestClient,
    abi: &Abi,
    tvc: &String,
    initial_pubkey: &Option<ed25519_dalek::PublicKey>,
    tvc_pubkey: &Option<ed25519_dalek::PublicKey>,
    signer_pubkey: &Option<ed25519_dalek::PublicKey>,
    expected_pubkey: &Option<ed25519_dalek::PublicKey>,
) -> Result<()> {
    let mut image = create_tvc_image(&abi.json_string()?, None, &tvc)?;
    if let Some(tvc_pubkey) = tvc_pubkey {
        image.set_public_key(tvc_pubkey)?;
    }

    let tvc = base64::encode(&image.serialize()?);

    let deploy_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: Some(DeploySet {
            tvc,
            workchain_id: None,
            initial_data: None,
            initial_pubkey: initial_pubkey.map(|key| hex::encode(key.as_bytes())),
        }),
        signer: if let Some(key) = signer_pubkey {
            Signer::External {
                public_key: hex::encode(key.as_bytes())
            }
        } else {
            Signer::None
        },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };

    let result: ResultOfEncodeMessage = client.request("abi.encode_message", deploy_params)?;

    let message = Message::construct_from_base64(&result.message)?;
    let state_init = message.state_init()
        .expect("Expected State Init")
        .write_to_bytes()?;
    let image = ContractImage::from_state_init(&mut Cursor::new(state_init))?;
    let public_key = image.get_public_key()?;

    assert_eq!(&public_key, expected_pubkey);

    Ok(())
}

fn gen_pubkey() -> ed25519_dalek::PublicKey {
    ed25519_dalek::Keypair::generate(&mut rand::thread_rng()).public
}

#[test]
fn test_encode_internal_message() -> Result<()> {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package(HELLO, None);
    let image = create_tvc_image(&abi.json_string()?, None, &tvc)?;

    test_encode_internal_message_deploy(
        &client,
        &image,
        &abi,
        &tvc,
        None,
        Some(
            "te6ccgECHAEABGkAAmECADYO5IoxskLmUfURre2fOB04OmP32VjPwA/lDM/Cpvh8AAAAAAAAAAAAAAAAAAIyBgEBAc\
            ACAgPPIAUDAQHeBAAD0CAAQdgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAIm/wD0pCAiwAGS9KDhiu1TW\
            DD0oQkHAQr0pCD0oQgAAAIBIAwKAej/fyHTAAGOJoECANcYIPkBAXDtRND0BYBA9A7yitcL/wHtRyJvde1XAwH5EPKo\
            3u1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7Vfi0z8B7UdvEyG\
            5IAsAYJ8wIPgjgQPoqIIIG3dAoLneme1HIW9TIO1XMJSANPLw4jDTHwH4I7zyudMfAfFAAQIBIBgNAgEgEQ4BCbqLVf\
            P4DwH67UdvYW6OO+1E0CDXScIBjhb0BNM/0wDtRwFvcQFvdgFvcwFvcu1Xjhj0Be1HAW9ycG9zcG92yIAgz0DJ0G9x7\
            Vfi3u1HbxaS8jOX7Udxb1btV+IA+ADR+CO1H+1HIG8RMAHIyx/J0G9R7VftR28SyPQA7UdvE88LP+1HbxYQABzPCwDt\
            R28RzxbJ7VRwagIBahUSAQm0ABrWwBMB/O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3LtV44Y9AX\
            tRwFvcnBvc3BvdsiAIM9AydBvce1X4t7tR29lIG6SMHDecO1HbxKAQPQO8orXC/+68uBk+AD6QNEgyMn7BIED6HCBAI\
            DIcc8LASLPCgBxz0D4KBQAjs8WJM8WI/oCcc9AcPoCcPoCgEDPQPgjzwsfcs9AIMki+wBfBTDtR28SyPQA7UdvE88LP\
            +1HbxbPCwDtR28RzxbJ7VRwatswAQm0ZfaLwBYB+O1Hb2FujjvtRNAg10nCAY4W9ATTP9MA7UcBb3EBb3YBb3MBb3Lt\
            V44Y9AXtRwFvcnBvc3BvdsiAIM9AydBvce1X4t7R7UdvEdcLH8iCEFDL7ReCEIAAAACxzwsfIc8LH8hzzwsB+CjPFnL\
            PQPglzws/gCHPQCDPNSLPMbwXAHiWcc9AIc8XlXHPQSHN4iDJcfsAWyHA/44e7UdvEsj0AO1HbxPPCz/tR28WzwsA7U\
            dvEc8Wye1U3nFq2zACASAbGQEJu3MS5FgaAPjtR29hbo477UTQINdJwgGOFvQE0z/TAO1HAW9xAW92AW9zAW9y7VeOG\
            PQF7UcBb3Jwb3Nwb3bIgCDPQMnQb3HtV+Le+ADR+CO1H+1HIG8RMAHIyx/J0G9R7VftR28SyPQA7UdvE88LP+1HbxbP\
            CwDtR28RzxbJ7VRwatswAMrdcCHXSSDBII4rIMAAjhwj0HPXIdcLACDAAZbbMF8H2zCW2zBfB9sw4wTZltswXwbbMOM\
            E2eAi0x80IHS7II4VMCCCEP////+6IJkwIIIQ/////rrf35bbMF8H2zDgIyHxQAFfBw=="
        )
    )?;

    test_encode_internal_message_deploy(
        &client,
        &image,
        &abi,
        &tvc,
        Some(CallSet {
            function_name: "constructor".into(),
            header: None,
            input: None,
        }),
        None,
    )?;

    test_encode_internal_message_run(
        &client,
        &abi,
        Some(CallSet {
            function_name: "sayHello".into(),
            header: None,
            input: None,
        }),
        Some("te6ccgEBAQEANgAAaAIACRorPEhV5veJGis8SFXm94kaKzxIVeb3iRorPEhV5veAAAAAAAAAAAAAAAAAAFDL7Rc="),
    )
}

fn test_encode_internal_message_run(
    client: &TestClient,
    abi: &Abi,
    call_set: Option<CallSet>,
    expected_boc: Option<&str>,
) -> Result<()> {
    fn check_str_field(map: &serde_json::Map<String, serde_json::Value>, field_name: &str, expected: &str) {
        let value = map.get(field_name)
            .expect(&format!("Expected `{}` field", field_name));
        let value = value.as_str()
            .expect(&format!("Expected `{}` to have `String` type", field_name));
        assert_eq!(value, expected, "Check failed: {}", field_name);
    }

    let address = String::from("0:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    let result: ResultOfEncodeInternalMessage = client.request(
        "abi.encode_internal_message",
        ParamsOfEncodeInternalMessage {
            abi: abi.clone(),
            address: Some(address.clone()),
            deploy_set: None,
            call_set,
            value: "1000000000".to_string(),
            bounce: Some(true),
            enable_ihr: None
        },
    )?;

    assert_eq!(result.address, address);
    assert_eq!(result.message_id, get_boc_hash(&base64::decode(&result.message)?)?);
    if let Some(expected_boc) = expected_boc {
        assert_eq!(&result.message, expected_boc);
    }

    let parsed = parse_message(
        Arc::new(ClientContext::new(Default::default())?),
        ParamsOfParse {
            boc: result.message
        }
    )?;

    let json_obj = parsed.parsed.as_object()
        .expect("Expected JSON Object");

    check_str_field(json_obj, "msg_type_name", "internal");
    check_str_field(json_obj, "src", "");
    check_str_field(json_obj, "dst", &address);

    Ok(())
}


fn test_encode_internal_message_deploy(
    client: &TestClient,
    image: &ContractImage,
    abi: &Abi,
    tvc: &String,
    call_set: Option<CallSet>,
    expected_boc: Option<&str>,
) -> Result<()> {
    let result: ResultOfEncodeInternalMessage = client.request(
        "abi.encode_internal_message",
        ParamsOfEncodeInternalMessage {
            abi: abi.clone(),
            address: None,
            deploy_set: Some(DeploySet {
                tvc: tvc.clone(),
                workchain_id: None,
                initial_data: None,
                initial_pubkey: None,
            }),
            call_set,
            value: "0".to_string(),
            bounce: None,
            enable_ihr: None
        },
    )?;

    assert_eq!(result.address, image.msg_address(0).to_string());
    assert_eq!(result.message_id, get_boc_hash(&base64::decode(&result.message)?)?);
    if let Some(expected_boc) = expected_boc {
        assert_eq!(&result.message, expected_boc);
    }

    let parsed = parse_message(
        Arc::new(ClientContext::new(Default::default())?),
        ParamsOfParse {
            boc: result.message
        }
    )?;

    let parsed = parsed.parsed.as_object()
        .expect("Expected JSON Object");
    let code = parsed.get("code")
        .expect("Expected `code` field");
    let code_str = code.as_str()
        .expect("Expected `code` to have `String` type");

    let code_from_tvc: ResultOfGetCodeFromTvc = client.request(
        "boc.get_code_from_tvc",
        ParamsOfGetCodeFromTvc {
            tvc: tvc.clone(),
        }
    )?;

    assert_eq!(code_str, code_from_tvc.code);

    Ok(())
}
