use std::collections::HashMap;

use futures::future::join_all;
use ic_cdk::export::candid::Principal;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{init, query, update};

use union_utils::fns::{log, send_events};
use union_utils::types::{Account, OnMoveListener, OnMoveListenersInfo};

use crate::utils::{
    Controllers, Error, FungibleToken, FungibleTokenInfo, FungibleTokenInitPayload,
    FungibleTokenTransferEntry,
};

mod utils;

static mut TOKEN: Option<FungibleToken> = None;

#[init]
fn init(payload: FungibleTokenInitPayload) {
    log("fungible_token.init()");

    let c = payload
        .controllers
        .unwrap_or_else(|| Controllers::single(Account::Some(caller())));

    let mut token = FungibleToken {
        balances: HashMap::new(),
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
fn balance_of(token_holder: Principal) -> u64 {
    log("fungible_token.balance_of()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.balance_of(&token_holder)
}

#[query]
fn total_supply() -> u64 {
    log("fungible_token.total_supply()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.total_supply
}

#[query]
fn info() -> FungibleTokenInfo {
    log("fungible_token.info()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.info.clone()
}

#[update]
fn update_info(new_info: FungibleTokenInfo) -> Result<FungibleTokenInfo, Error> {
    log("fungible_token.update_info()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_info(new_info, caller())
}

#[query]
fn controllers() -> Controllers {
    log("fungible_token.controllers()");

    let token = unsafe { TOKEN.as_ref().unwrap() };

    token.controllers.clone()
}

#[update]
fn update_info_controller(new_controller: Account) -> Result<(), Error> {
    log("fungible_token.update_info_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_info_controller(new_controller, caller())
}

#[update]
fn update_mint_controller(new_controller: Account) -> Result<(), Error> {
    log("fungible_token.update_mint_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_mint_controller(new_controller, caller())
}

#[update]
fn update_on_move_controller(new_controller: Account) -> Result<(), Error> {
    log("fungible_token.update_on_move_controller()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.update_on_move_controller(new_controller, caller())
}

#[update]
async fn mint(entries: Vec<FungibleTokenTransferEntry>) -> Vec<Result<(), Error>> {
    log("fungible_token.mint()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let results: Vec<_> = entries
        .into_iter()
        .map(|entry| token.mint(entry.to, entry.qty, caller()))
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
async fn send(entries: Vec<FungibleTokenTransferEntry>) -> Vec<Result<(), Error>> {
    log("fungible_token.send()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let results: Vec<_> = entries
        .into_iter()
        .map(|entry| token.send(caller(), entry.to, entry.qty))
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
async fn burn(quantity: u64) -> Result<(), Error> {
    log("fungible_token.burn()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let ev_and_listeners = token.burn(caller(), quantity)?;
    send_events(ev_and_listeners).await;

    Ok(())
}

#[update]
fn subscribe_on_move(listeners: Vec<OnMoveListener>) -> Vec<Result<u64, Error>> {
    log("fungible_token.subscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    listeners
        .into_iter()
        .map(|listener| token.subscribe_on_move(listener, caller()))
        .collect()
}

#[update]
fn unsubscribe_on_move(listener_ids: Vec<u64>) -> Vec<Result<OnMoveListener, Error>> {
    log("fungible_token.unsubscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    listener_ids
        .into_iter()
        .map(|listener_id| token.unsubscribe_on_move(listener_id, caller()))
        .collect()
}
