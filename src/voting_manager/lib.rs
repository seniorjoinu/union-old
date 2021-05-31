use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::Principal;
use ic_cdk_macros::{init, update};
use union_utils::fns::{log, remote_call};
use union_utils::types::{RemoteCallEndpoint, RemoteCallPayload};

use crate::utils::{Error, NewVotingParams, Vote, Voting};

mod utils;

static mut VOTING: Option<Voting> = None;

#[init]
fn init() {
    log("votings.init()");

    unsafe {
        VOTING = Some(Voting::new(NewVotingParams {
            creator: caller(),
            duration: 1000 * 1000 * 1000 * 60 * 30, // 30 min
            title: String::from("Test voting"),
            description: String::from("Test desc"),
            payload: Some(RemoteCallPayload {
                endpoint: RemoteCallEndpoint {
                    canister_id: Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
                    method_name: String::from("mint"),
                },
                idl_str_args: String::from("(principal \"rwlgt-iiaaa-aaaaa-aaaaa-cai\", 15 : nat)"),
                payment: 0,
            }),
            timestamp: time(),
        }));
    }
}

#[update]
fn do_vote(voting_power: u64, vote: Vote) -> Result<(), Error> {
    log("votings.do_vote()");

    unsafe {
        VOTING
            .as_mut()
            .unwrap()
            .vote(&caller(), voting_power, vote, 0.2, time())
    }
}

#[update]
async fn execute() -> Result<Option<Vec<u8>>, Error> {
    log("votings.execute()");

    let voting = unsafe { VOTING.as_mut().unwrap() };

    voting.execute(time())?;

    let entry_opt = voting.payload.clone();

    if let Some(entry) = entry_opt {
        return remote_call(entry)
            .await
            .map_err(Error::VotingExecutionError)
            .map(Some);
    }

    Ok(None)
}
