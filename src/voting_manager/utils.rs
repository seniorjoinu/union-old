use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use union_utils::fns::is_passing_threshold;
use union_utils::types::{RemoteCallError, RemoteCallPayload, RemoteCallResult};

#[derive(Clone, Debug, CandidType, Deserialize, PartialOrd, PartialEq)]
pub enum VotingStatus {
    Proposal,
    Approved,
    Rejected,
    Finished,
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
    VotingIsRejected,
    VotingThresholdError,
    VotingThresholdNotPassed,
    VotingAlreadyExecuted,
    CallerIsNotCreator,
    VotingExecutionError(RemoteCallError),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Voting {
    pub created_at: i64,
    pub updated_at: i64,

    pub approval: f64,
    pub rejection: f64,
    pub quorum: f64,
    pub consensus: f64,
    pub duration: Option<i64>,

    pub title: String,
    pub description: String,
    pub payload: Vec<RemoteCallPayload>,
    pub execute_result: Vec<RemoteCallResult>,

    pub union_wallet: Principal,
    pub proposer: Principal,
    pub status: VotingStatus,

    pub voters_for: HashMap<Principal, i64>,
    pub voting_power_for: u64,
    pub voters_against: HashMap<Principal, i64>,
    pub voting_power_against: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct NewVotingParams {
    pub union_wallet: Principal,

    pub approval: f64,
    pub rejection: f64,
    pub quorum: f64,
    pub consensus: f64,
    pub duration: Option<i64>,

    pub title: String,
    pub description: String,
    pub payload: Vec<RemoteCallPayload>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpdateVotingParams {
    pub union_wallet: Option<Principal>,

    pub approval: Option<f64>,
    pub rejection: Option<f64>,
    pub quorum: Option<f64>,
    pub consensus: Option<f64>,
    pub duration: Option<Option<i64>>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub payload: Option<Vec<RemoteCallPayload>>,
}

impl Voting {
    // TODO: duration_sec
    pub fn new(proposer: Principal, timestamp: i64, params: NewVotingParams) -> Voting {
        Voting {
            created_at: timestamp,
            updated_at: timestamp,

            approval: params.approval,
            rejection: params.rejection,
            quorum: params.quorum,
            consensus: params.consensus,
            duration: params.duration,

            title: params.title,
            description: params.description,
            payload: params.payload,
            execute_result: Vec::new(),

            union_wallet: params.union_wallet,
            proposer,
            status: VotingStatus::Proposal,

            voters_for: HashMap::new(),
            voting_power_for: 0,
            voters_against: HashMap::new(),
            voting_power_against: 0,
        }
    }

    pub fn vote(
        &mut self,
        voter: &Principal,
        vote_voting_power: u64,
        total_voting_power: u64,
        vote: Vote,
        timestamp: i64,
    ) -> Result<(), Error> {
        if let Some(duration) = self.duration {
            if self.updated_at + duration < timestamp {
                return Err(Error::VotingAlreadyFinished);
            }
        }

        if self.status == VotingStatus::Rejected {
            return Err(Error::VotingIsRejected);
        }

        if self.status == VotingStatus::Executed {
            return Err(Error::VotingAlreadyExecuted);
        }

        self.remove_prev_vote(voter, vote_voting_power);

        match vote {
            Vote::Abstain => (),
            Vote::For => {
                self.voting_power_for += vote_voting_power;
                self.voters_for.insert(voter.clone(), timestamp);
            }
            Vote::Against => {
                self.voting_power_against += vote_voting_power;
                self.voters_against.insert(voter.clone(), timestamp);
            }
        };

        if self.status == VotingStatus::Proposal {
            if is_passing_threshold(
                self.voting_power_against,
                total_voting_power,
                self.rejection,
            ) {
                self.status = VotingStatus::Rejected;
            }

            if is_passing_threshold(self.voting_power_for, total_voting_power, self.approval) {
                self.status = VotingStatus::Approved;
            }
        }

        Ok(())
    }

    pub fn execute(&mut self, timestamp: i64) -> Result<(), Error> {
        if let Some(duration) = self.duration {
            if self.updated_at + duration >= timestamp {
                return Err(Error::VotingIsNotYetFinished);
            }
        }

        if self.status == VotingStatus::Proposal {
            return Err(Error::VotingThresholdNotPassed);
        }

        if self.status == VotingStatus::Executed {
            return Err(Error::VotingAlreadyExecuted);
        }

        if self.status == VotingStatus::Rejected {
            return Err(Error::VotingIsRejected);
        }

        self.status = VotingStatus::Executed;

        Ok(())
    }

    pub fn update(&mut self, params: UpdateVotingParams, timestamp: i64) -> Result<(), Error> {
        if self.status != VotingStatus::Proposal {
            return Err(Error::VotingAlreadyStarted);
        }

        if let Some(uw) = params.union_wallet {
            self.union_wallet = uw;
        }

        if let Some(a) = params.approval {
            self.approval = a;
        }

        if let Some(r) = params.rejection {
            self.rejection = r;
        }

        if let Some(q) = params.quorum {
            self.quorum = q;
        }

        if let Some(c) = params.consensus {
            self.consensus = c;
        }

        if let Some(d) = params.duration {
            self.duration = d;
        }

        if let Some(t) = params.title {
            self.title = t;
        }

        if let Some(d) = params.description {
            self.description = d;
        }

        if let Some(p) = params.payload {
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
