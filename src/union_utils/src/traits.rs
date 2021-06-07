use ic_cdk::export::candid::Principal;

use crate::types::{RemoteCallPayload, RemoteCallResult, VotingId};

/*
type RemoteCallEndpoint = record {
    canister_id: principal;
    method_name: text;
};

type RemoteCallPayload = record {
    endpoint: RemoteCallEndpoint;
    idl_str_args: text;
    payment: int64;
};

type RemoteCallError = variant {
    UnableToParseArgs;
    UnableToSerializeArgs;
    RemoteCallReject : text;
};

type RemoteCallResult = variant {
    Ok : blob;
    Err : RemoteCallError;
};

service : {
    "_union_call" : (vec RemoteCallPayload, nat32) -> (vec RemoteCallResult);
}
 */
pub trait IUnionWallet {
    fn _union_call(p: Vec<RemoteCallPayload>, voting_id: VotingId) -> Vec<RemoteCallResult>;
}

/*
service : {
    "_union_voting_power_of_at" : (principal, int64) -> (nat64);
    "_union_total_voting_power_at" : (int64) -> (nat64);
}
 */
pub trait IMembershipGuard {
    fn _union_voting_power_of_at(of: Principal, at: i64) -> u64;
    fn _union_total_voting_power_at(at: i64) -> u64;
}

/*
type VotingId = record {
    union_wallet : principal;
    idx : nat64;
};

type Event = variant {
    VotingCreated
};

service : {
    "_union_on_event" : (id: VotingId) -> ();
}
 */
pub trait IEventListener {
    fn _union_on_voting_created(id: VotingId);
    fn _union_on_voting_updated(id: VotingId);
    fn _union_on_voting_state_changed(id: VotingId);
    fn _union_on_vote_placed(id: VotingId);
}