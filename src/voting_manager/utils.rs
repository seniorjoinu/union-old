use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use std::collections::HashMap;
use union_utils::types::{RemoteCallError, RemoteCallPayload};

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
pub enum Error {
    VotingAlreadyFinished,
    VotingIsNotYetFinished,
    VotingAlreadyStarted,
    VotingThresholdError,
    VotingThresholdNotPassed,
    VotingAlreadyExecuted,
    CallerIsNotCreator,
    VotingExecutionError(RemoteCallError),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Voting {
    pub creator: Principal,
    pub status: VotingStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub duration: i64,
    pub title: String,
    pub description: String,
    pub payload: Option<RemoteCallPayload>,
    pub voters_for: HashMap<Principal, i64>,
    pub voting_power_for: u64,
    pub voters_against: HashMap<Principal, i64>,
    pub voting_power_against: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct NewVotingParams {
    pub creator: Principal,
    pub duration: i64,
    pub title: String,
    pub description: String,
    pub payload: Option<RemoteCallPayload>,
    pub timestamp: i64,
}

impl Voting {
    // TODO: duration_sec
    pub fn new(params: NewVotingParams) -> Voting {
        Voting {
            creator: params.creator,
            status: VotingStatus::Created,
            created_at: params.timestamp,
            updated_at: params.timestamp,
            duration: params.duration,
            title: params.title,
            description: params.description,
            payload: params.payload,
            voters_for: HashMap::new(),
            voting_power_for: 0,
            voters_against: HashMap::new(),
            voting_power_against: 0,
        }
    }

    // TODO: access control for a voting_token
    pub fn vote(
        &mut self,
        voter: &Principal,
        voting_power: u64,
        vote: Vote,
        threshold: f64,
        timestamp: i64,
    ) -> Result<(), Error> {
        if self.updated_at + self.duration < timestamp {
            return Err(Error::VotingAlreadyFinished);
        }

        self.remove_prev_vote(voter, voting_power);

        match vote {
            Vote::Abstain => (),
            Vote::For => {
                self.voting_power_for += voting_power;
                self.voters_for.insert(voter.clone(), timestamp);
            }
            Vote::Against => {
                self.voting_power_against += voting_power;
                self.voters_against.insert(voter.clone(), timestamp);
            }
        };

        if self.status == VotingStatus::Created && is_passing_threshold(self, threshold) {
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
        payload: Option<Option<RemoteCallPayload>>,
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

    fn remove_prev_vote(&mut self, voter: &Principal, voting_power: u64) {
        let vote_for = self.voters_for.get(voter);
        if vote_for.is_some() {
            self.voters_for.remove(voter);
            self.voting_power_for -= voting_power;

            return;
        }

        let vote_against = self.voters_against.get(voter);
        if vote_against.is_some() {
            self.voters_against.remove(voter);
            self.voting_power_against -= voting_power;
        }
    }
}

fn is_passing_threshold(voting: &Voting, threshold: f64) -> bool {
    true
}
