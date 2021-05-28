use fraction::DynaDecimal;
use ic_cdk::api::call::RejectionCode;
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
pub struct VotingPayloadEntry {
    pub canister_id: Principal,
    pub method_name: String,
    pub args: String,
    pub payment: i64,
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
    ArgsAreNotValid,
    PayloadEntryFailed(String),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Voting {
    pub used_token_id: Nat,
    pub used_token_total_supply: Nat,
    pub creator: Principal,
    pub status: VotingStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub duration: i64,
    pub title: String,
    pub description: String,
    pub payload: Option<VotingPayloadEntry>,
    pub voters_for: HashMap<Principal, i64>,
    pub votes_for: Nat,
    pub voters_against: HashMap<Principal, i64>,
    pub votes_against: Nat,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct NewVotingParams {
    pub used_token_id: Nat,
    pub used_token_total_supply: Nat,
    pub creator: Principal,
    pub duration: i64,
    pub title: String,
    pub description: String,
    pub payload: Option<VotingPayloadEntry>,
    pub timestamp: i64,
}

impl Voting {
    // TODO: duration_sec
    pub fn new(params: NewVotingParams) -> Voting {
        Voting {
            used_token_id: params.used_token_id,
            used_token_total_supply: params.used_token_total_supply,
            creator: params.creator,
            status: VotingStatus::Created,
            created_at: params.timestamp,
            updated_at: params.timestamp,
            duration: params.duration,
            title: params.title,
            description: params.description,
            payload: params.payload,
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
        threshold: f32,
        timestamp: i64,
    ) -> Result<(), Error> {
        if self.updated_at + self.duration < timestamp {
            return Err(Error::VotingAlreadyFinished);
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
                self.votes_for.clone() + self.votes_against.clone(),
                self.used_token_total_supply.clone(),
                threshold,
            )
        {
            self.status = VotingStatus::Started;
        }

        Ok(())
    }

    pub fn execute(&mut self, timestamp: i64) -> Result<(), Error> {
        //if self.updated_at + self.duration >= timestamp {
        //    return Some(Error::VotingIsNotYetFinished);
        //}

        if self.status == VotingStatus::Created {
            return Err(Error::VotingThresholdNotPassed);
        }

        if self.status == VotingStatus::Executed {
            return Err(Error::VotingAlreadyExecuted);
        }

        self.status = VotingStatus::Executed;

        Ok(())
    }

    pub fn update(
        &mut self,
        duration: Option<i64>,
        title: Option<String>,
        description: Option<String>,
        payload: Option<Option<VotingPayloadEntry>>,
        timestamp: i64,
        caller: Principal,
    ) -> Result<(), Error> {
        if self.status != VotingStatus::Created {
            return Err(Error::VotingAlreadyStarted);
        }

        if caller != self.creator {
            return Err(Error::CallerIsNotCreator);
        }

        if let Some(d) = duration {
            self.duration = d;
        }

        if let Some(t) = title {
            self.title = t;
        }

        if let Some(d) = description {
            self.description = d;
        }

        if let Some(p) = payload {
            self.payload = p;
        }

        self.updated_at = timestamp;

        Ok(())
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

fn is_passing_threshold(small: Nat, big: Nat, threshold: f32) -> bool {
    type D = DynaDecimal<usize, u8>;

    let num = D::from(small.0.to_string().as_str());
    let deno = D::from(big.0.to_string().as_str());
    let thresh = D::from(threshold.to_string().as_str());

    num / deno >= thresh
}
