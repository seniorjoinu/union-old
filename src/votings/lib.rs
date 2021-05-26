use crate::utils::{Error, NewVotingParams, Vote, Voting, VotingPayloadEntry};
use candid::IDLArgs;
use ic_cdk::api::call::call_raw;
use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::candid::{CandidType, Nat};
use ic_cdk::export::Principal;
use ic_cdk_macros::{init, update};
use ic_logger::log_fn;

mod utils;

static mut VOTING: Option<Voting> = None;

#[init]
fn init() {
    log_fn("votings", "init");

    unsafe {
        VOTING = Some(Voting::new(NewVotingParams {
            used_token_id: Nat::from(0),
            used_token_total_supply: Nat::from(100),
            creator: caller(),
            duration: 1000 * 1000 * 1000 * 60 * 5, // 5 min
            title: String::from("Test voting"),
            description: String::from("Test desc"),
            payload: vec![VotingPayloadEntry {
                canister_id: Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
                method_name: String::from("mint"),
                args: String::from("(principal \"rrkah-fqaaa-aaaaa-aaaaq-cai\", 100)"),
                payment: 0,
            }],
            timestamp: time(),
        }));
    }
}

#[update]
fn do_vote(voting_power: Nat, vote: Vote) -> Option<Error> {
    log_fn("votings", "do_vote");

    unsafe {
        VOTING
            .as_mut()?
            .vote(&caller(), voting_power, vote, 0.2f32, time())
    }
}

#[update]
async fn execute() -> Option<Error> {
    log_fn("votings", "execute");

    unsafe {
        let voting = VOTING.as_mut()?;

        match voting.execute(time()) {
            Some(e) => Some(e),
            None => {
                for (idx, entry) in voting.payload.iter().enumerate() {
                    let args = entry.args.parse::<IDLArgs>();

                    match args {
                        Err(_) => return Some(Error::ArgsAreNotValid),
                        Ok(idl_args) => {
                            let raw_args = idl_args.to_bytes();

                            if raw_args.is_err() {
                                return Some(Error::ArgsAreNotValid);
                            }

                            ic_cdk::print(format!(
                                "Calling remote canister: {}.{}{}",
                                entry.canister_id.to_text(),
                                entry.method_name,
                                idl_args.to_string()
                            ));

                            let bytes = raw_args.unwrap();
                            ic_cdk::print(format!("{:x?}", &bytes));

                            let result = call_raw(
                                entry.canister_id.clone(),
                                entry.method_name.as_str(),
                                bytes,
                                entry.payment,
                            )
                            .await;

                            if result.is_err() {
                                return Some(Error::PayloadEntryFailed(idx));
                            }
                        }
                    }
                }

                None
            }
        }
    }
}
