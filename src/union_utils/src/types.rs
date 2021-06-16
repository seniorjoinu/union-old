use std::collections::{HashMap, HashSet};

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

/*
type RemoteCallEndpoint = record {
     canister_id: principal;
     method_name: text;
};
*/
#[derive(Clone, Hash, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct RemoteCallEndpoint {
    pub canister_id: Principal,
    pub method_name: String,
}

/*
type RemoteCallPayload = record {
     endpoint: RemoteCallEndpoint;
     idl_str_args: text;
     payment: int64;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallPayload {
    pub endpoint: RemoteCallEndpoint,
    pub idl_str_args: String,
    pub payment: i64,
}

/*
type RemoteCallError = variant {
     UnableToParseArgs;
     UnableToSerializeArgs;
     RemoteCallReject : text;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum RemoteCallError {
    UnableToParseArgs,
    UnableToSerializeArgs,
    RemoteCallReject(String),
}

/*
type RemoteCallResult = variant {
     Ok : blob;
     Err : RemoteCallError;
};
*/
pub type RemoteCallResult = Result<Vec<u8>, RemoteCallError>;

/*
type Controlled_* = record {
     data : *;
     controller : opt principal;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Controlled<T> {
    pub data: T,
    pub controller: Option<Principal>,
}

impl<T> Controlled<T> {
    pub fn by(exact: Principal, data: T) -> Controlled<T> {
        Controlled {
            controller: Some(exact),
            data,
        }
    }

    pub fn by_no_one(data: T) -> Controlled<T> {
        Controlled {
            controller: None,
            data,
        }
    }

    pub fn is_controller(&self, principal: Principal) -> bool {
        if let Some(controller) = self.controller.clone() {
            return controller == principal;
        }

        true
    }
}

/*
type CanisterInfo = record {
     canister_id : principal;
     description : text;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CanisterInfo {
    pub canister_id: Principal,
    pub description: String,
}

/*
type VotingId = record {
     union_wallet : principal;
     idx : nat64;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingId {
    pub union_wallet: Principal,
    pub idx: usize,
}

/*
type Account = variant {
     None;
     Some : principal;
};
*/
pub type Account = Option<Principal>;

/*
type TokenMoveEvent = record {
     from : Account;
     to : Account;
     qty : nat64;
};
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenMoveEvent {
    pub from: Account,
    pub to: Account,
    pub qty: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenMoveEventAndListeners {
    pub event: TokenMoveEvent,
    pub listeners: Vec<OnMoveListener>,
}

/*
 type AccountFilter = variant {
   None;
   Some : Account;
 }
*/
pub type AccountFilter = Option<Account>;

/*
 type Filter = record {
   from : AccountFilter;
   to : AccountFilter;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Filter {
    pub from: AccountFilter,
    pub to: AccountFilter,
}

/*
 type OnMoveListener {
   filter : Filter;
   endpoint : RemoveCallEndpoint;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct OnMoveListener {
    pub filter: Filter,
    pub endpoint: RemoteCallEndpoint,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum OnMoveListenerError {
    AccessDenied,
    ListenerDoesNotExist,
    ListenerFatalError,
}

#[derive(Clone, Default, Debug, CandidType, Deserialize)]
pub struct OnMoveListenersInfo {
    pub id_counter: u64,
    pub enumeration: HashMap<u64, OnMoveListener>,
    pub index: HashMap<AccountFilter, Vec<u64>>,
}

impl OnMoveListenersInfo {
    pub fn add_listener(&mut self, listener: OnMoveListener) -> Result<u64, OnMoveListenerError> {
        let id = self.id_counter;
        self.enumeration.insert(id, listener.clone());

        // add only one listener if there are the same filters for both 'from' and 'to'
        if listener.filter.from == listener.filter.to {
            let index = self
                .index
                .entry(listener.filter.from)
                .or_insert_with(Vec::new);

            index.push(id);
        } else {
            let index_from = self
                .index
                .entry(listener.filter.from.clone())
                .or_insert_with(Vec::new);

            index_from.push(id);

            let index_to = self
                .index
                .entry(listener.filter.to)
                .or_insert_with(Vec::new);

            index_to.push(id);
        }

        self.id_counter += 1;

        Ok(id)
    }

    pub fn remove_listener(&mut self, id: u64) -> Result<OnMoveListener, OnMoveListenerError> {
        let listener = self
            .enumeration
            .remove(&id)
            .ok_or(OnMoveListenerError::ListenerDoesNotExist)?;

        // remove only one listener if there are the same filters for both 'from' and 'to'
        if listener.filter.from == listener.filter.to {
            let index = self
                .index
                .get_mut(&listener.filter.from)
                .ok_or(OnMoveListenerError::ListenerFatalError)?;

            let idx = index
                .binary_search(&id)
                .map_err(|_| OnMoveListenerError::ListenerFatalError)?;

            index.remove(idx);
        } else {
            // remove listener 'from'
            let index_from = self
                .index
                .get_mut(&listener.filter.from)
                .ok_or(OnMoveListenerError::ListenerFatalError)?;
            let idx_from = index_from
                .binary_search(&id)
                .map_err(|_| OnMoveListenerError::ListenerFatalError)?;
            index_from.remove(idx_from);

            // remove listener 'to'
            let index_to = self
                .index
                .get_mut(&listener.filter.to)
                .ok_or(OnMoveListenerError::ListenerFatalError)?;
            let idx_to = index_to
                .binary_search(&id)
                .map_err(|_| OnMoveListenerError::ListenerFatalError)?;
            index_to.remove(idx_to);
        }

        Ok(listener)
    }

    pub fn get_matching_listeners(&self, event: &TokenMoveEvent) -> Vec<OnMoveListener> {
        // collect listeners which listen for ANY sender
        let any_ids = self
            .index
            .get(&AccountFilter::None)
            .cloned()
            .unwrap_or_else(Vec::new);

        // collect listeners which listen for 'from'
        let from_ids = self
            .index
            .get(&AccountFilter::Some(event.from.clone()))
            .cloned()
            .unwrap_or_else(Vec::new);

        // collect listeners which listen for 'to'
        let to_ids = self
            .index
            .get(&AccountFilter::Some(event.to.clone()))
            .cloned()
            .unwrap_or_else(Vec::new);

        // dedup
        let unique_ids = any_ids
            .iter()
            .chain(from_ids.iter())
            .chain(to_ids.iter())
            .collect::<HashSet<_>>();

        // get listeners by id
        unique_ids
            .iter()
            .map(|&id| self.enumeration.get(id).unwrap().clone())
            .collect()
    }
}
