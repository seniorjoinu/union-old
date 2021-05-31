use std::time::{Duration, UNIX_EPOCH};

use chrono::prelude::DateTime;
use chrono::Utc;
use ic_cdk::api::call::call_raw;
use ic_cdk::api::time;
use ic_cdk::export::candid::{IDLArgs, Principal};
use ic_cdk::{caller, print, trap};

use crate::types::*;

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
        entry.endpoint.canister_id.to_text(),
        entry.endpoint.method_name,
        idl_args.to_string()
    )
    .as_str());

    let result = call_raw(
        entry.endpoint.canister_id.clone(),
        entry.endpoint.method_name.as_str(),
        raw_args,
        entry.payment,
    )
    .await
    .map_err(|(_, err)| RemoteCallError::RemoteCallReject(err))?;

    log(format!("{:02X?}", result).as_str());

    Ok(result)
}

pub fn only_by(controller_opt: Option<Principal>) {
    if let Some(controller) = controller_opt {
        if controller != caller() {
            trap("Access denied");
        }
    }
}

pub fn is_passing_threshold(small: u64, big: u64, threshold: f64) -> bool {
    small as f64 / big as f64 >= threshold
}
