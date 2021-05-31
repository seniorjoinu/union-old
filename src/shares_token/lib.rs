use std::collections::HashMap;

use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::{init, query, update};

use union_utils::fns::log;

use crate::utils::{Error, SharesToken, SharesTokenHistory};

mod utils;

static mut TOKEN: Option<SharesToken> = None;

#[init]
fn init() {
    log("shares_token.init()");

    unsafe {
        TOKEN = Some(SharesToken {
            name: "Default token".into(),
            balances: HashMap::new(),
            total_supplies: SharesTokenHistory::new(),
        });
    }
}

#[query]
fn balance_of(token_holder: Principal) -> u64 {
    log("shares_token.balance_of()");

    unsafe { TOKEN.as_ref().unwrap().balance_of_at(&token_holder, None) }
}

#[query]
fn _union_voting_power_of_at(p: Principal, t: i64) -> u64 {
    log("shares_token._union_voting_power_of_at()");

    unsafe { TOKEN.as_ref().unwrap().balance_of_at(&p, Some(t)) }
}

#[query]
fn total_supply() -> u64 {
    log("shares_token.total_supply()");

    unsafe { TOKEN.as_ref().unwrap().total_supply_at(None) }
}

#[query]
fn _union_total_voting_power(t: i64) -> u64 {
    log("shares_token._union_voting_power_of_at()");

    unsafe { TOKEN.as_ref().unwrap().total_supply_at(Some(t)) }
}

#[update]
fn mint(to: Principal, quantity: u64) -> Result<(), Error> {
    log("shares_token.mint()");

    unsafe {
        // TODO: add access control
        TOKEN.as_mut().unwrap().mint(&to, quantity, time());

        Ok(())
    }
}

#[update]
fn send(to: Principal, quantity: u64) -> Result<(), Error> {
    log("shares_token.send()");

    unsafe {
        TOKEN
            .as_mut()
            .unwrap()
            .send(&caller(), &to, quantity, time())
    }
}

#[update]
fn burn(quantity: u64) -> Result<(), Error> {
    log("shares_token.burn()");

    unsafe { TOKEN.as_mut().unwrap().burn(&caller(), quantity, time()) }
}
