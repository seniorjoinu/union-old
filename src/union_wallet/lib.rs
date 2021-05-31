use candid::Principal;
use ic_cdk_macros::{init, update};
use union_utils::fns::{log, only_by, remote_call};
use union_utils::types::{RemoteCallPayload, RemoteCallResult, VotingId};

static mut VOTING_MANAGER: Option<Principal> = None;

#[init]
fn init() {
    log(format!("union_wallet<>.init()").as_str());

    unsafe { VOTING_MANAGER = None }
}

#[update]
async fn _union_call(p: Vec<RemoteCallPayload>, _: VotingId) -> Vec<RemoteCallResult> {
    let voting_manager = unsafe { VOTING_MANAGER.clone() };

    only_by(voting_manager);

    let mut results: Vec<RemoteCallResult> = Vec::new();
    
    for p in p.iter() {
        results.push(remote_call(p.clone()).await);
    }
    
    results
}