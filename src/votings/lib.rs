use ic_cdk::export::candid::ser::ArgumentEncoder;
use ic_cdk::export::candid::{CandidType, Deserialize, Nat, Principal};
use std::collections::HashMap;

#[derive(Clone, Debug, CandidType, Deserialize, PartialOrd, PartialEq)]
pub enum VotingStatus {
    Created,
    Started,
    Executed,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialOrd, PartialEq)]
pub enum Vote {
    For,
    Against,
    Abstain,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingPayloadEntry<T: ArgumentEncoder> {
    pub canister_id: Principal,
    pub method_name: String,
    pub args: HashMap<String, T>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Error {
    VotingAlreadyFinished,
    VotingIsNotYetFinished,
    VotingAlreadyStarted,
    VotingThresholdError,
    VotingThresholdNotPassed,
    VotingAlreadyExecuted,
    CallerIsNotCreator,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Voting<T: ArgumentEncoder> {
    pub used_token_id: Nat,
    pub used_token_total_supply: Nat,
    pub creator: Principal,
    pub status: VotingStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub duration: i64,
    pub title: String,
    pub description: String,
    pub payload: Vec<VotingPayloadEntry<T>>,
    pub voters_for: HashMap<Principal, i64>,
    pub votes_for: Nat,
    pub voters_against: HashMap<Principal, i64>,
    pub votes_against: Nat,
}

impl<T: ArgumentEncoder> Voting<T> {
    // TODO: duration_sec
    pub fn new(
        used_token_id: Nat,
        used_token_total_supply: Nat,
        creator: Principal,
        duration: i64,
        title: String,
        description: String,
        payload: Vec<VotingPayloadEntry<T>>,
        timestamp: i64,
    ) -> Voting<T> {
        Voting {
            used_token_id,
            used_token_total_supply,
            creator,
            status: VotingStatus::Created,
            created_at: timestamp,
            updated_at: timestamp,
            duration,
            title,
            description,
            payload,
            voters_for: HashMap::new(),
            votes_for: Nat::from(0),
            voters_against: HashMap::new(),
            votes_against: Nat::from(0),
        }
    }

    // TODO: access control for a voting_token
    pub fn vote(
        &mut self,
        voter: &Principal,
        voting_power: Nat,
        vote: Vote,
        reverse_threshold: u64,
        timestamp: i64,
    ) -> Option<Error> {
        if self.updated_at + self.duration < timestamp {
            return Some(Error::VotingAlreadyFinished);
        }

        self.remove_prev_vote(voter, &voting_power);

        match vote {
            Vote::Abstain => (),
            Vote::For => {
                self.votes_for += voting_power;
                self.voters_for.insert(voter.clone(), timestamp);
            }
            Vote::Against => {
                self.votes_against += voting_power;
                self.voters_against.insert(voter.clone(), timestamp);
            }
        };

        if self.status == VotingStatus::Created
            && is_passing_threshold(
                self.used_token_total_supply.clone(),
                self.votes_for.clone() + self.votes_against.clone(),
                reverse_threshold,
            )
        {
            self.status = VotingStatus::Started;
        }

        None
    }

    pub fn execute(&mut self, timestamp: i64) -> Option<Error> {
        if self.updated_at + self.duration >= timestamp {
            return Some(Error::VotingIsNotYetFinished);
        }

        if self.status == VotingStatus::Created {
            return Some(Error::VotingThresholdNotPassed);
        }

        if self.status == VotingStatus::Executed {
            return Some(Error::VotingAlreadyExecuted);
        }

        // TODO: do execute in canister code itself

        self.status = VotingStatus::Executed;
        None
    }

    pub fn update(
        &mut self,
        duration: Option<i64>,
        title: Option<String>,
        description: Option<String>,
        payload: Option<Vec<VotingPayloadEntry<T>>>,
        timestamp: i64,
        caller: Principal,
    ) -> Option<Error> {
        if self.status != VotingStatus::Created {
            return Some(Error::VotingAlreadyStarted);
        }

        if caller != self.creator {
            return Some(Error::CallerIsNotCreator);
        }

        if duration.is_some() {
            self.duration = duration.unwrap();
        }

        if title.is_some() {
            self.title = title.unwrap();
        }

        if description.is_some() {
            self.description = description.unwrap();
        }

        if payload.is_some() {
            self.payload = payload.unwrap();
        }

        self.updated_at = timestamp;

        None
    }

    fn remove_prev_vote(&mut self, voter: &Principal, voting_power: &Nat) {
        let vote_for = self.voters_for.get(voter);
        if vote_for.is_some() {
            self.voters_for.remove(voter);
            self.votes_for -= voting_power.clone();

            return;
        }

        let vote_against = self.voters_against.get(voter);
        if vote_against.is_some() {
            self.voters_against.remove(voter);
            self.votes_against -= voting_power.clone();
        }
    }
}

// TODO: threshold is always > 0 and < 0.5

fn is_passing_threshold(big: Nat, small: Nat, reverse_threshold: u64) -> bool {
    small * reverse_threshold >= big
}

// TODO: add `fraction` rust library for threshold
// (1.0 / threshold).to_u64()
