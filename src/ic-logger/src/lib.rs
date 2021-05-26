use chrono::prelude::DateTime;
use chrono::Utc;
use ic_cdk::api::time;
use ic_cdk::{caller, print};
use std::time::{Duration, UNIX_EPOCH};

fn make_time(nanosec: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_nanos(nanosec.unsigned_abs());
    let datetime = DateTime::<Utc>::from(d);

    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

pub fn log_fn(canister_name: &str, fn_name: &str) {
    print(format!(
        "{}: {}.{}() [caller: {}]",
        make_time(time()),
        canister_name,
        fn_name,
        caller()
    ))
}
