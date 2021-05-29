use chrono::prelude::DateTime;
use chrono::Utc;
use ic_cdk::api::call::call_raw;
use ic_cdk::api::time;
use ic_cdk::export::candid::IDLArgs;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::{caller, print};
use std::time::{Duration, UNIX_EPOCH};

fn make_time(nanosec: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_nanos(nanosec.unsigned_abs());
    let datetime = DateTime::<Utc>::from(d);

    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

pub fn log(msg: &str) {
    print(format!(
        "[caller: {}] - {}: {}",
        caller(),
        make_time(time()),
        msg
    ))
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallPayload {
    pub canister_id: Principal,
    pub method_name: String,
    pub idl_str_args: String,
    pub payment: i64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum RemoteCallError {
    UnableToParseArgs,
    UnableToSerializeArgs,
    RemoteCallReject(String),
}

pub async fn remote_call(entry: RemoteCallPayload) -> Result<Vec<u8>, RemoteCallError> {
    let idl_args = entry
        .idl_str_args
        .parse::<IDLArgs>()
        .map_err(|_| RemoteCallError::UnableToParseArgs)?;

    let raw_args = idl_args
        .to_bytes()
        .map_err(|_| RemoteCallError::UnableToSerializeArgs)?;

    log(format!(
        "Calling remote canister: {}.{}{}",
        entry.canister_id.to_text(),
        entry.method_name,
        idl_args.to_string()
    )
    .as_str());

    let result = call_raw(
        entry.canister_id.clone(),
        entry.method_name.as_str(),
        raw_args,
        entry.payment,
    )
    .await
    .map_err(|(_, err)| RemoteCallError::RemoteCallReject(err))?;

    log(format!("{:02X?}", result).as_str());

    Ok(result)
}
