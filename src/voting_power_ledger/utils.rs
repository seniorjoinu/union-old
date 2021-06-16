use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Error {
    EmitterAlreadyRegistered,
    EmitterNotRegistered,
    AccessDenied,
    HistoryLookupFatalError,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct VotingPowerEntry {
    pub timestamp: i64,
    pub balance: u64,
}

pub type VotingPowerHistory = Vec<VotingPowerEntry>;

// account -> history
#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct VotingPowerLedger {
    history: HashMap<Principal, VotingPowerHistory>,
    total_voting_power: VotingPowerHistory,
}

// token canister -> ledger
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GlobalVotingPowerLedger(pub HashMap<Principal, VotingPowerLedger>);

impl GlobalVotingPowerLedger {
    pub fn supply_total_voting_power_entry(
        &mut self,
        canister_id: Principal,
        new_total_voting_power: u64,
        timestamp: i64,
    ) -> Result<(), Error> {
        let ledger = self
            .0
            .entry(canister_id)
            .or_insert_with(VotingPowerLedger::default);

        let entry = VotingPowerEntry {
            timestamp,
            balance: new_total_voting_power,
        };

        ledger.total_voting_power.push(entry);

        Ok(())
    }

    // caller == canister_id
    pub fn supply_voting_power_entry(
        &mut self,
        canister_id: Principal,
        account_id: Principal,
        voting_power: u64,
        timestamp: i64,
    ) -> Result<(), Error> {
        let ledger = self
            .0
            .entry(canister_id)
            .or_insert_with(VotingPowerLedger::default);
        let history = ledger
            .history
            .entry(account_id)
            .or_insert_with(VotingPowerHistory::new);

        let entry = VotingPowerEntry {
            timestamp,
            balance: voting_power,
        };

        history.push(entry);

        Ok(())
    }

    pub fn get_voting_power_at(
        &self,
        canister_id: &Principal,
        account_id: &Principal,
        timestamp: i64,
    ) -> Result<u64, Error> {
        let ledger = self.0.get(canister_id).ok_or(Error::EmitterNotRegistered)?;

        match ledger.history.get(account_id) {
            None => Ok(0),
            Some(history) => match lookup_history_at(history, timestamp) {
                // if this executes - something really wrong with the code
                None => Err(Error::HistoryLookupFatalError),
                Some(vp) => Ok(vp),
            },
        }
    }

    pub fn get_total_voting_power_at(
        &self,
        canister_id: &Principal,
        timestamp: i64,
    ) -> Result<u64, Error> {
        let ledger = self.0.get(canister_id).ok_or(Error::EmitterNotRegistered)?;

        lookup_history_at(&ledger.total_voting_power, timestamp).ok_or(Error::HistoryLookupFatalError)
    }
}

fn lookup_history(
    history: &VotingPowerHistory,
    begin: usize,
    end: usize,
    timestamp: i64,
) -> Option<usize> {
    let mid_left = begin + (end - begin) / 2;
    let mid_right = mid_left + 1;

    let left_timestamp = history.get(mid_left).unwrap().timestamp;
    let right_timestamp = history.get(mid_right).unwrap().timestamp;

    // if we're in between (left is lower, right is higher) or if we found exact value - return its index
    if left_timestamp <= timestamp && right_timestamp > timestamp {
        return Some(mid_left);
    }

    if right_timestamp == timestamp {
        return Some(mid_right);
    }

    // if we're higher than both left and right, repeat for the left side
    if left_timestamp < timestamp && right_timestamp < timestamp {
        return lookup_history(history, mid_right, end, timestamp);
    }

    // if we're lower than both left and right, repeat for the right side
    if left_timestamp > timestamp && right_timestamp > timestamp {
        return lookup_history(history, begin, mid_left, timestamp);
    }

    // it is impossible, because we've checked boundaries before
    None
}

fn lookup_history_at(history: &VotingPowerHistory, timestamp: i64) -> Option<u64> {
    match history.first() {
        None => Some(0),
        Some(f) => {
            // if the timestamp is earlier than the first balance snapshot - it's balance is 0
            if timestamp < f.timestamp {
                return Some(0);
            }

            match history.last() {
                None => Some(0),
                Some(l) => {
                    // if the timestamp is more recent than the last balance snapshot - it's balance is the last balance
                    if timestamp >= l.timestamp {
                        return Some(l.balance);
                    }

                    // otherwise - binary search based lookup
                    let idx = lookup_history(history, 0, history.len(), timestamp);
                    idx.map(|i| history.get(i).unwrap().balance)
                }
            }
        }
    }
}
