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
}
 */
pub trait IMembershipGuard {
    fn _union_voting_power_of_at(of: Principal, at: i64) -> u64;
}
