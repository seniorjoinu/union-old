use ic_cdk::export::candid::{CandidType, Deserialize, Nat, Principal};
use std::collections::{HashMap, LinkedList};

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

pub type VotingTokenHistory = LinkedList<BalanceEntry>;

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
            Some(h) => match h.front() {
                None => Nat::from(0),
                Some(b) => b.balance.clone(),
            },
        }
    }

    fn push_balance(&mut self, token_holder: &Principal, entry: BalanceEntry) {
        match self.balances.get_mut(token_holder) {
            None => {
                let mut history = VotingTokenHistory::new();
                history.push_front(entry);

                self.balances.insert(token_holder.clone(), history);
            }
            Some(h) => h.push_front(entry),
        };
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
        // TODO: binary search

        self.peek_balance(token_holder)
    }
}
