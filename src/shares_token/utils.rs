use std::borrow::BorrowMut;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::trap;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct SharesToken {
    pub name: String,
    pub balances: HashMap<Principal, SharesTokenHistory>,
    pub total_supplies: SharesTokenHistory,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct BalanceEntry {
    pub timestamp: i64,
    pub balance: u64,
}

pub type SharesTokenHistory = Vec<BalanceEntry>;

fn lookup_history(
    history: &SharesTokenHistory,
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

impl SharesToken {
    pub fn mint(&mut self, to: &Principal, quantity: u64, timestamp: i64) {
        // changing balance
        let history = match self.balances.get_mut(to) {
            None => {
                self.balances.insert(to.clone(), SharesTokenHistory::new());
                self.balances.get_mut(to).unwrap()
            }
            Some(h) => h,
        };
        let prev_balance = peek_history(history);
        let new_balance = BalanceEntry {
            timestamp,
            balance: prev_balance + quantity,
        };
        push_entry(history, new_balance);

        // adding total supply entry
        let total_supply_history = &mut self.total_supplies;
        let prev_total_supply = peek_history(total_supply_history);
        let new_total_supply = BalanceEntry {
            timestamp,
            balance: prev_total_supply + quantity,
        };
        push_entry(total_supply_history, new_total_supply);
    }

    pub fn send(
        &mut self,
        from: &Principal,
        to: &Principal,
        quantity: u64,
        timestamp: i64,
    ) -> Result<(), Error> {
        // changing balance from
        let history_from_opt = self.balances.get_mut(from);
        if history_from_opt.is_none() {
            return Err(Error::InsufficientBalance);
        }
        let mut history_from = history_from_opt.unwrap();
        let latest_balance_from = peek_history(history_from);
        if latest_balance_from < quantity {
            return Err(Error::InsufficientBalance);
        }
        let new_balance_from = BalanceEntry {
            timestamp,
            balance: latest_balance_from - quantity,
        };
        push_entry(history_from, new_balance_from);

        // changing balance to
        let history_to_opt = self.balances.get_mut(to);
        let mut history_to = match history_to_opt {
            None => {
                self.balances.insert(to.clone(), SharesTokenHistory::new());
                self.balances.get_mut(to).unwrap()
            }
            Some(h) => h,
        };
        let latest_balance_to = peek_history(history_to);
        let new_balance_to = BalanceEntry {
            timestamp,
            balance: latest_balance_to + quantity,
        };
        push_entry(history_to, new_balance_to);

        Ok(())
    }

    pub fn burn(&mut self, from: &Principal, quantity: u64, timestamp: i64) -> Result<(), Error> {
        let history_opt = self.balances.get_mut(from);
        if history_opt.is_none() {
            return Err(Error::InsufficientBalance);
        }

        // changing balance
        let history = history_opt.unwrap();
        let latest_balance = peek_history(history);
        if latest_balance < quantity {
            return Err(Error::InsufficientBalance);
        }
        let new_balance = BalanceEntry {
            timestamp,
            balance: latest_balance - quantity,
        };
        push_entry(history, new_balance);

        // adding total supply entry
        let total_supply_history = &mut self.total_supplies;
        let prev_total_supply = peek_history(total_supply_history);
        let new_total_supply = BalanceEntry {
            timestamp,
            balance: prev_total_supply - quantity,
        };
        push_entry(total_supply_history, new_total_supply);

        Ok(())
    }

    pub fn balance_of_at(&self, token_holder: &Principal, timestamp: Option<i64>) -> u64 {
        let history = self.balances.get(token_holder);
        if history.is_none() {
            return 0;
        }

        match timestamp {
            None => peek_history(history.unwrap()),
            Some(t) => match lookup_history_at(history.unwrap(), t) {
                None => trap("Balance history lookup failed due to internal error"),
                Some(b) => b,
            },
        }
    }

    pub fn total_supply_at(&self, timestamp: Option<i64>) -> u64 {
        if self.total_supplies.is_empty() {
            return 0;
        }

        match timestamp {
            None => peek_history(&self.total_supplies),
            Some(t) => match lookup_history_at(&self.total_supplies, t) {
                None => trap("Total supply history lookup failed due to internal error"),
                Some(b) => b,
            },
        }
    }
}

fn peek_history(history: &SharesTokenHistory) -> u64 {
    history.last().map_or(0, |b| b.balance)
}

fn push_entry(history: &mut SharesTokenHistory, entry: BalanceEntry) {
    history.push(entry);
}

fn lookup_history_at(history: &SharesTokenHistory, timestamp: i64) -> Option<u64> {
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
