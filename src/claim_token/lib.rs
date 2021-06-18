use std::collections::HashMap;

use futures::future::join_all;
use ic_cdk::export::candid::Principal;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{init, query, update};

use union_utils::fns::{log, send_events};
use union_utils::types::{Account, OnMoveListener, OnMoveListenersInfo};

use crate::utils::{ClaimToken, ClaimTokenInfo, ClaimTokenInitPayload, Controllers, Error};

mod utils;

static mut TOKEN: Option<ClaimToken> = None;

#[init]
fn init(payload: ClaimTokenInitPayload) {
    log("claim_token.init()");

    let c = payload
        .controllers
        .unwrap_or_else(|| Controllers::single(Account::Some(caller())));

    let mut token = ClaimToken {
        claims: HashMap::new(),
        total_supply: 0,
        on_move_listeners: OnMoveListenersInfo::default(),
        info: payload.info,
        controllers: c,
    };

    if !payload.on_move_listeners.is_empty() {
        let did_threw = payload
            .on_move_listeners
            .into_iter()
            .map(|listener| token.subscribe_on_move(listener, caller()))
            .any(|res| res.is_err());

        if did_threw {
            trap("Invalid on move listeners!");
        }
    }

    unsafe {
        TOKEN = Some(token);
    }
}

#[query]
fn has_claim(token_holder: Principal) -> bool {
    log("claim_token.has_claim()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.has_claim(&token_holder)
}

#[query]
fn total_supply() -> u64 {
    log("claim_token.total_supply()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.total_supply
}

#[query]
fn info() -> ClaimTokenInfo {
    log("claim_token.info()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.info.clone()
}

#[update]
fn update_info(new_info: ClaimTokenInfo) -> Result<ClaimTokenInfo, Error> {
    log("claim_token.update_info()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_info(new_info, caller())
}

#[query]
fn controllers() -> Controllers {
    log("claim_token.controllers()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.controllers.clone()
}

#[update]
fn update_info_controller(new_controller: Account) -> Result<(), Error> {
    log("claim_token.update_info_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_info_controller(new_controller, caller())
}

#[update]
fn update_issue_controller(new_controller: Account) -> Result<(), Error> {
    log("claim_token.update_issue_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_issue_controller(new_controller, caller())
}

#[update]
fn update_revoke_controller(new_controller: Account) -> Result<(), Error> {
    log("claim_token.update_revoke_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_revoke_controller(new_controller, caller())
}

#[update]
fn update_on_move_controller(new_controller: Account) -> Result<(), Error> {
    log("claim_token.update_on_move_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_on_move_controller(new_controller, caller())
}

#[update]
async fn issue(recipients: Vec<Principal>) -> Vec<Result<(), Error>> {
    log("claim_token.issue()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let results: Vec<_> = recipients
        .into_iter()
        .map(|to| token.issue(to, caller()))
        .map(|res| async {
            match res {
                Ok(ev_n_list) => {
                    send_events(ev_n_list).await;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        })
        .collect();

    join_all(results).await
}

#[update]
async fn revoke(holders: Vec<Principal>) -> Vec<Result<(), Error>> {
    log("claim_token.revoke()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let results: Vec<_> = holders
        .into_iter()
        .map(|from| token.revoke(from, caller()))
        .map(|res| async {
            match res {
                Ok(ev_n_list) => {
                    send_events(ev_n_list).await;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        })
        .collect();

    join_all(results).await
}

#[update]
fn subscribe_on_move(listeners: Vec<OnMoveListener>) -> Vec<Result<u64, Error>> {
    log("claim_token.subscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    listeners
        .into_iter()
        .map(|listener| token.subscribe_on_move(listener, caller()))
        .collect()
}

#[update]
fn unsubscribe_on_move(listener_ids: Vec<u64>) -> Vec<Result<OnMoveListener, Error>> {
    log("claim_token.unsubscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    listener_ids
        .into_iter()
        .map(|listener_id| token.unsubscribe_on_move(listener_id, caller()))
        .collect()
}
