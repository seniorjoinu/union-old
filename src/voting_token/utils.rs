use ic_cdk::export::candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::trap;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct VotingToken {
    pub name: String,
    pub balances: HashMap<Principal, VotingTokenHistory>,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct BalanceEntry {
    pub timestamp: i64,
    pub balance: Nat,
}

pub type VotingTokenHistory = Vec<BalanceEntry>;

fn lookup_history(
    history: &VotingTokenHistory,
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

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Error {
    InsufficientBalance,
    AccessDenied,
}

pub trait IVotingToken {
    fn mint(&mut self, to: &Principal, quantity: &Nat, timestamp: i64);
    fn send(
        &mut self,
        from: &Principal,
        to: &Principal,
        quantity: &Nat,
        timestamp: i64,
    ) -> Option<Error>;
    fn burn(&mut self, from: &Principal, quantity: &Nat, timestamp: i64) -> Option<Error>;
    fn balance_of(&self, token_holder: &Principal, timestamp: Option<i64>) -> Nat;
}

impl VotingToken {
    fn peek_balance(&self, token_holder: &Principal) -> Nat {
        match self.balances.get(token_holder) {
            None => Nat::from(0),
            Some(h) => match h.last() {
                None => Nat::from(0),
                Some(b) => b.balance.clone(),
            },
        }
    }

    fn push_balance(&mut self, token_holder: &Principal, entry: BalanceEntry) {
        match self.balances.get_mut(token_holder) {
            None => {
                let mut history = VotingTokenHistory::new();
                history.push(entry);

                self.balances.insert(token_holder.clone(), history);
            }
            Some(h) => h.push(entry),
        };
    }

    fn lookup_balance(&self, token_holder: &Principal, timestamp: i64) -> Option<Nat> {
        match self.balances.get(token_holder) {
            // if there is no history for account - it's balance is 0
            None => Some(Nat::from(0)),

            Some(history) => match history.first() {
                None => Some(Nat::from(0)),
                Some(f) => {
                    // if the timestamp is earlier than the first balance snapshot - it's balance is 0
                    if timestamp < f.timestamp {
                        return Some(Nat::from(0));
                    }

                    match history.last() {
                        None => Some(Nat::from(0)),
                        Some(l) => {
                            // if the timestamp is more recent than the last balance snapshot - it's balance is the last balance
                            if timestamp >= l.timestamp {
                                return Some(l.balance.clone());
                            }

                            // otherwise - binary search based lookup
                            let idx = lookup_history(history, 0, history.len(), timestamp);
                            idx.map(|i| history.get(i).unwrap().balance.clone())
                        }
                    }
                }
            },
        }
    }
}

impl IVotingToken for VotingToken {
    fn mint(&mut self, to: &Principal, quantity: &Nat, timestamp: i64) {
        let prev_balance = self.peek_balance(to);

        let new_balance = BalanceEntry {
            timestamp,
            balance: prev_balance + quantity.clone(),
        };

        self.push_balance(to, new_balance);
    }

    fn send(
        &mut self,
        from: &Principal,
        to: &Principal,
        quantity: &Nat,
        timestamp: i64,
    ) -> Option<Error> {
        let latest_balance_from = self.peek_balance(from);
        let latest_balance_to = self.peek_balance(to);

        if latest_balance_from < quantity.clone() {
            return Some(Error::InsufficientBalance);
        }

        let new_balance_from = BalanceEntry {
            timestamp,
            balance: latest_balance_from - quantity.clone(),
        };
        let new_balance_to = BalanceEntry {
            timestamp,
            balance: latest_balance_to + quantity.clone(),
        };

        self.push_balance(from, new_balance_from);
        self.push_balance(to, new_balance_to);

        None
    }

    fn burn(&mut self, from: &Principal, quantity: &Nat, timestamp: i64) -> Option<Error> {
        let latest_balance = self.peek_balance(from);

        if latest_balance < quantity.clone() {
            return Some(Error::InsufficientBalance);
        }

        let new_balance = BalanceEntry {
            timestamp,
            balance: latest_balance - quantity.clone(),
        };

        self.push_balance(from, new_balance);

        None
    }

    fn balance_of(&self, token_holder: &Principal, timestamp: Option<i64>) -> Nat {
        match timestamp {
            None => self.peek_balance(token_holder),
            Some(t) => match self.lookup_balance(token_holder, t) {
                None => trap("Balance history lookup failed due to internal error"),
                Some(b) => b,
            },
        }
    }
}
