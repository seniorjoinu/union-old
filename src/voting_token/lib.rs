mod utils;

use crate::utils::{Error, IVotingToken, VotingToken};
use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::candid::{Nat, Principal};
use ic_cdk_macros::{init, query, update};
use std::collections::HashMap;

static mut TOKEN: Option<VotingToken> = None;

#[init]
fn init() {
    ic_cdk::print("!!! Calling init");

    unsafe {
        TOKEN = Some(VotingToken {
            name: "Default token".into(),
            balances: HashMap::new(),
        });
    }
}

#[query]
fn balance_of(token_holder: Principal) -> Nat {
    unsafe { TOKEN.as_mut().unwrap().balance_of(&token_holder, None) }
}

#[update]
fn mint(to: Principal, quantity: Nat) -> Option<Error> {
    unsafe {
        // TODO: add access control
        TOKEN.as_mut().unwrap().mint(&to, &quantity, time());

        None
    }
}

#[update]
fn send(to: Principal, quantity: Nat) -> Option<Error> {
    unsafe {
        TOKEN
            .as_mut()
            .unwrap()
            .send(&caller(), &to, &quantity, time())
    }
}

#[update]
fn burn(quantity: Nat) -> Option<Error> {
    unsafe { TOKEN.as_mut().unwrap().burn(&caller(), &quantity, time()) }
}
