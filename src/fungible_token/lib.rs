use std::collections::HashMap;

use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::Principal;
use ic_cdk::{call, caller};
use ic_cdk_macros::{init, query, update};

use union_utils::fns::log;
use union_utils::types::{
    Account, OnMoveListener, OnMoveListenersInfo, TokenMoveEventAndListeners,
};

use crate::utils::{Controllers, Error, FungibleToken, FungibleTokenInfo};

mod utils;

static mut TOKEN: Option<FungibleToken> = None;

#[init]
fn init(info: FungibleTokenInfo, controllers: Option<Controllers>) {
    log("fungible_token.init()");

    let c = controllers.unwrap_or(Controllers::single(Account::Some(caller())));

    unsafe {
        TOKEN = Some(FungibleToken {
            balances: HashMap::new(),
            total_supply: 0,
            on_move_listeners: OnMoveListenersInfo::default(),
            info,
            controllers: c,
        });
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
fn mint(to: Principal, quantity: u64) -> Result<(), Error> {
    log("fungible_token.mint()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let ev_and_listeners = token.mint(to, quantity, caller())?;
    send_events(ev_and_listeners);

    Ok(())
}

#[update]
fn send(to: Principal, quantity: u64) -> Result<(), Error> {
    log("fungible_token.send()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let ev_and_listeners = token.send(caller(), to, quantity)?;
    send_events(ev_and_listeners);

    Ok(())
}

#[update]
fn burn(quantity: u64) -> Result<(), Error> {
    log("fungible_token.burn()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    let ev_and_listeners = token.burn(caller(), quantity)?;
    send_events(ev_and_listeners);

    Ok(())
}

#[update]
fn subscribe_on_move(listener: OnMoveListener) -> Result<u64, Error> {
    log("fungible_token.subscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.subscribe_on_move(listener, caller())
}

#[update]
fn unsubscribe_on_move(listener_id: u64) -> Result<OnMoveListener, Error> {
    log("fungible_token.unsubscribe_on_move()");

    let token = unsafe { TOKEN.as_mut().unwrap() };

    token.unsubscribe_on_move(listener_id, caller())
}

fn send_events(ev_and_listeners: TokenMoveEventAndListeners) {
    for listener in ev_and_listeners.listeners.iter() {
        call::<_, ()>(
            listener.endpoint.canister_id.clone(),
            listener.endpoint.method_name.as_str(),
            (ev_and_listeners.event.clone(),),
        );
    }
}
