use ic_cdk::api::call::CallResult;
use ic_cdk::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

/*
 * type RemoteCallEndpoint = record {
 *      canister_id: principal;
 *      method_name: text;
 * };
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallEndpoint {
    pub canister_id: Principal,
    pub method_name: String,
}

/*
 * type RemoteCallPayload = record {
 *      endpoint: RemoteCallEndpoint;
 *      idl_str_args: text;
 *      payment: int64;
 * };
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallPayload {
    pub endpoint: RemoteCallEndpoint,
    pub idl_str_args: String,
    pub payment: i64,
}

/*
 * type RemoteCallError = variant {
 *      UnableToParseArgs;
 *      UnableToSerializeArgs;
 *      RemoteCallReject : text;
 * };
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum RemoteCallError {
    UnableToParseArgs,
    UnableToSerializeArgs,
    RemoteCallReject(String),
}

/*
 * type RemoteCallResult = variant {
 *      Ok : blob;
 *      Err : RemoteCallError;
 * };
 */
pub type RemoteCallResult = Result<Vec<u8>, RemoteCallError>;

/*
 * type Controlled_* = record {
 *      data : *;
 *      controller : opt principal;
 * };
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Controlled<T> {
    pub data: T,
    pub controller: Option<Principal>,
}

impl<T> Controlled<T> {
    pub fn by_caller(data: T) -> Controlled<T> {
        Controlled {
            controller: Some(caller()),
            data,
        }
    }

    pub fn by_no_one(data: T) -> Controlled<T> {
        Controlled {
            controller: None,
            data,
        }
    }
}

/*
 * type CanisterInfo = record {
 *      canister_id : principal;
 *      description : text;
 * };
 */
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CanisterInfo {
    pub canister_id: Principal,
    pub description: String,
}

pub type VotingId = usize;