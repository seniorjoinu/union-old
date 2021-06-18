use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk_macros::{init, update};

use union_utils::fns::{log, only_by, remote_call};
use union_utils::types::{RemoteCallPayload, RemoteCallResult, VotingId};

/*
 type UnionCallPayload {
   program : vec RemoteCallPayload;
   voting_id : VotingId;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UnionCallPayload {
    program: Vec<RemoteCallPayload>,
    voting_id: VotingId,
}

static mut CALL_CONTROLLER: Option<Principal> = None;

#[init]
fn init(call_controller: Principal) {
    log("union_wallet<>.init()");

    unsafe { CALL_CONTROLLER = Some(call_controller) }
}

#[update]
async fn _union_call(payload: UnionCallPayload) -> Vec<RemoteCallResult> {
    let call_controller = unsafe { CALL_CONTROLLER };

    only_by(call_controller);

    let mut results: Vec<RemoteCallResult> = Vec::new();

    for instruction in payload.program.into_iter() {
        results.push(remote_call(instruction).await);
    }

    results
}
