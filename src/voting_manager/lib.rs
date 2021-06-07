use std::collections::HashMap;

use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk::export::Principal;
use ic_cdk_macros::{init, update};

use union_utils::fns::{log, remote_call};
use union_utils::types::{RemoteCallEndpoint, RemoteCallPayload};

use crate::utils::{Error, NewVotingParams, Vote, Voting, VotingConfigType, VotingManager};

mod utils;

static mut VOTING_MANAGER: Option<VotingManager> = None;

#[init]
fn init() {
    log("voting_manager.init()");

    unsafe {
        VOTING_MANAGER = Some(VotingManager {
            votings: HashMap::new(),
            voting_config: HashMap::new(),
            membership_guards: HashMap::new(),
            event_listeners: HashMap::new(),
        })
    }
}

#[update]
fn create_voting(params: NewVotingParams) -> Result<(), Error> {
    log("votings.do_vote()");

    let voting_manager = unsafe { VOTING_MANAGER.as_mut().unwrap() };
    match voting_manager.voting_config.get(&params.union_wallet) {
        None => Err(Error::VotingConfigDoesNotExist),
        Some(config) => match config.data {
            VotingConfigType::None(c) => c,
        },
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

#[update]
fn update() -> Result<(), Error> {
    if caller != self.proposer {
        return Err(Error::CallerIsNotCreator);
    }
}
