use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::{init, query, update};
use std::collections::HashMap;
use union_utils::fns::log;

use crate::utils::{Error, VotingToken};

mod utils;

static mut TOKEN: Option<VotingToken> = None;

#[init]
fn init() {
    log("voting_token.init()");

    unsafe {
        TOKEN = Some(VotingToken {
            name: "Default token".into(),
            balances: HashMap::new(),
        });
    }
}

#[query]
fn balance_of(token_holder: Principal, timestamp: Option<i64>) -> u64 {
    log("voting_token.balance_of()");

    unsafe { TOKEN.as_mut().unwrap().balance_of(&token_holder, timestamp) }
}

#[update]
fn mint(to: Principal, quantity: u64) -> Result<(), Error> {
    log("voting_token.mint()");

    unsafe {
        // TODO: add access control
        TOKEN.as_mut().unwrap().mint(&to, quantity, time());

        Ok(())
    }
}

#[update]
fn send(to: Principal, quantity: u64) -> Result<(), Error> {
    log("voting_token.send()");

    unsafe {
        TOKEN
            .as_mut()
            .unwrap()
            .send(&caller(), &to, quantity, time())
    }
}

#[update]
fn burn(quantity: u64) -> Result<(), Error> {
    log("voting_token.burn()");

    unsafe { TOKEN.as_mut().unwrap().burn(&caller(), quantity, time()) }
}
