use std::collections::HashMap;

use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::{init, query, update};

use union_utils::fns::log;
use union_utils::types::{Account, TokenMoveEvent};

use crate::utils::{Error, GlobalVotingPowerLedger};

mod utils;

static mut LEDGER: Option<GlobalVotingPowerLedger> = None;

#[init]
fn init() {
    log("voting_power_ledger.init()");

    unsafe {
        LEDGER = Some(GlobalVotingPowerLedger(HashMap::new()));
    }
}

#[query]
fn voting_power_of_at(canister_id: Principal, p: Principal, t: i64) -> Result<u64, Error> {
    log("voting_power_ledger.voting_power_of_at()");

    unsafe {
        LEDGER
            .as_ref()
            .unwrap()
            .get_voting_power_at(&canister_id, p, t)
    }
}

#[query]
fn total_voting_power_at(canister_id: Principal, t: i64) -> Result<u64, Error> {
    log("voting_power_ledger.total_voting_power_at()");

    unsafe {
        LEDGER
            .as_ref()
            .unwrap()
            .get_total_voting_power_at(&canister_id, t)
    }
}

#[update]
fn register_emitter() -> Result<(), Error> {
    log("voting_power_ledger.register_emitter()");

    unsafe {
        LEDGER
            .as_mut()
            .unwrap()
            .register_voting_power_emitter(caller())
    }
}

#[update]
fn unregister_emitter() -> Result<(), Error> {
    log("voting_power_ledger.unregister_emitter()");

    unsafe {
        LEDGER
            .as_mut()
            .unwrap()
            .unregister_voting_power_emitter(caller())
    }
}

#[update]
fn handle_on_move(event: TokenMoveEvent) -> Result<(), Error> {
    log("voting_power_ledger.handle_on_move()");

    let ledger = unsafe { LEDGER.as_mut().unwrap() };
    let canister_id = caller();
    let time = time();

    // transfer
    if event.from.is_some() && event.to.is_some() {
        let from = event.from.unwrap();
        let to = event.to.unwrap();

        let from_vp = ledger.get_voting_power_at(&canister_id, &from, time)?;
        let to_vp = ledger.get_voting_power_at(&canister_id, &to, time)?;

        ledger.supply_voting_power_entry(canister_id, from, from_vp - event.qty, time)?;
        ledger.supply_voting_power_entry(canister_id.clone(), to, to_vp + event.qty, time)?;

    // burn
    } else if event.from.is_some() {
        let from = event.from.unwrap();

        let from_vp = ledger.get_voting_power_at(&canister_id, &from, time)?;
        let total_vp = ledger.get_total_voting_power_at(&canister_id, time)?;

        ledger.supply_voting_power_entry(canister_id, from, from_vp - event.qty, time)?;
        ledger.supply_total_voting_power_entry(canister_id.clone(), total_vp - event.qty, time)?;

    // mint
    } else if event.to.is_some() {
        let to = event.to.unwrap();

        let to_vp = ledger.get_voting_power_at(&canister_id, &to, time)?;
        let total_vp = ledger.get_total_voting_power_at(&canister_id, time)?;

        ledger.supply_voting_power_entry(canister_id, to, to_vp + event.qty, time)?;
        ledger.supply_total_voting_power_entry(canister_id.clone(), total_vp + event.qty, time)?;
    }

    Ok(())
}
