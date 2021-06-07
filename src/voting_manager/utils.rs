use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use union_utils::fns::is_passing_threshold;
use union_utils::types::{
    Controlled, RemoteCallEndpoint, RemoteCallError, RemoteCallPayload, RemoteCallResult, VotingId,
};

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
    VotingDoesNotExist,
    VotingThresholdError,
    VotingThresholdNotPassed,
    VotingAlreadyExecuted,
    CallerIsNotCreator,
    VotingExecutionError(RemoteCallError),
    VotingConfigDoesNotExist,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Voting {
    pub created_at: i64,
    pub updated_at: i64,

    pub can_vote: WhoCanVote,

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
pub enum WhoCanVote {
    Member,
    ExactMember(HashSet<Principal>),
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

    pub can_vote: WhoCanVote,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpdateVotingParams {
    pub approval: Option<f64>,
    pub rejection: Option<f64>,
    pub quorum: Option<f64>,
    pub consensus: Option<f64>,
    pub duration: Option<Option<i64>>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub payload: Option<Vec<RemoteCallPayload>>,

    pub can_vote: Option<WhoCanVote>,
}

impl Voting {
    // TODO: duration_sec
    pub fn new(proposer: Principal, timestamp: i64, params: NewVotingParams) -> Voting {
        Voting {
            created_at: timestamp,
            updated_at: timestamp,

            can_vote: params.can_vote,

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

        if let Some(c) = params.can_vote {
            self.can_vote = c;
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

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum VotingConfigType {
    None,
    Whitelist(HashSet<RemoteCallEndpoint>),
    Blacklist(HashSet<RemoteCallEndpoint>),
}

impl VotingConfigType {
    pub fn is_allowed_to_create(&self, params: &NewVotingParams) -> bool {
        match self {
            VotingConfigType::None => true,
            VotingConfigType::Whitelist(wl) => {
                // TODO: make it faster
                let endpoints = HashSet::from_iter(params.payload.iter().map(|it| it.endpoint));

                endpoints.is_subset(wl)
            }
            VotingConfigType::Blacklist(bl) => {
                // TODO: make it faster
                let endpoints = HashSet::from_iter(params.payload.iter().map(|it| it.endpoint));

                endpoints.is_disjoint(bl)
            }
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Interval<T> {
    pub min: T,
    pub max: T,
}

impl<T: PartialOrd> Interval<T> {
    pub fn contains(&self, a: T) -> bool {
        a >= self.min && a <= self.max
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum VotingCharacter {
    Member,
    All,
    Exact(HashSet<Principal>),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PossibleVoter {
    Any,
    CreatorList,
    Exact(HashSet<Principal>),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallVotingParams {
    pub approval: Interval<f64>,
    pub rejection: Interval<f64>,
    pub quorum: Interval<f64>,
    pub consensus: Interval<f64>,
    pub duration: Option<Interval<i64>>,
    pub can_vote: PossibleVoter,
    pub can_create: VotingCharacter,
    pub can_delete: VotingCharacter,
    pub can_execute: VotingCharacter,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingConfig {
    pub default: RemoteCallVotingParams,
    pub custom: HashMap<RemoteCallEndpoint, RemoteCallVotingParams>,
}

impl VotingConfig {
    pub fn is_allowed_to_create(
        &self,
        params: &NewVotingParams,
        proposer: &Principal,
        proposer_is_a_member: bool,
    ) -> bool {
        let mut result = true;

        for payload in params.payload.iter() {
            let endpoint_config = self.custom.get(&payload.endpoint).unwrap_or(&self.default);

            if !endpoint_config.approval.contains(params.approval) {
                result = false;
                break;
            }

            if !endpoint_config.rejection.contains(params.rejection) {
                result = false;
                break;
            }

            if !endpoint_config.consensus.contains(params.consensus) {
                result = false;
                break;
            }

            if !endpoint_config.quorum.contains(params.quorum) {
                result = false;
                break;
            }

            if let Some(d) = &endpoint_config.duration {
                if params.duration.is_none() {
                    result = false;
                    break;
                }

                if !d.contains(params.duration.unwrap()) {
                    result = false;
                    break;
                }
            }

            match &endpoint_config.can_create {
                VotingCharacter::All => (),
                VotingCharacter::Exact(p) => {
                    if p.contains(proposer) {
                        result = false;
                        break;
                    }
                }
                VotingCharacter::Member => {
                    if !proposer_is_a_member {
                        result = false;
                        break;
                    }
                }
            };
        }

        result
    }

    pub fn is_allowed_to_update(
        &self,
        params: &UpdateVotingParams,
        updater: &Principal,
        voting: &Voting,
    ) -> bool {
        if voting.proposer != updater.clone() {
            return false;
        }

        let endpoints = params.payload.unwrap_or(voting.payload.clone());

        let mut result = true;

        for payload in endpoints.iter() {
            let endpoint_config = self.custom.get(&payload.endpoint).unwrap_or(&self.default);

            if let Some(approval) = params.approval {
                if !endpoint_config.approval.contains(approval) {
                    result = false;
                    break;
                }
            }

            if let Some(rejection) = params.rejection {
                if !endpoint_config.rejection.contains(rejection) {
                    result = false;
                    break;
                }
            }

            if let Some(consensus) = params.consensus {
                if !endpoint_config.consensus.contains(consensus) {
                    result = false;
                    break;
                }
            }

            if let Some(quorum) = params.quorum {
                if !endpoint_config.quorum.contains(quorum) {
                    result = false;
                    break;
                }
            }

            if let Some(duration) = params.duration {
                if let Some(d) = &endpoint_config.duration {
                    if duration.is_none() {
                        result = false;
                        break;
                    }

                    if !d.contains(duration.unwrap()) {
                        result = false;
                        break;
                    }
                }
            }
        }

        result
    }

    pub fn is_allowed_to_vote(&self, voter: &Principal, voting: &Voting) -> bool {
        voting.payload.iter().all(|p| {
            let config = self.custom.get(&p.endpoint).unwrap_or(&self.default);

            match &config.can_vote {
                PossibleVoter::Any => true,
                PossibleVoter::Exact(voters) => voters.contains(voter),
                PossibleVoter::CreatorList => match &voting.can_vote {
                    WhoCanVote::Member => true,
                    WhoCanVote::ExactMember(voters) => voters.contains(voter),
                },
            }
        })
    }

    pub fn is_allowed_to_delete(
        &self,
        deleter: &Principal,
        voting: &Voting,
        is_deleter_a_member: bool,
    ) -> bool {
        voting.payload.iter().all(|p| {
            let config = self.custom.get(&p.endpoint).unwrap_or(&self.default);

            match &config.can_delete {
                VotingCharacter::All => true,
                VotingCharacter::Member => is_deleter_a_member,
                VotingCharacter::Exact(members) => members.contains(deleter),
            }
        })
    }

    pub fn is_allowed_to_execute(
        &self,
        executer: &Principal,
        voting: &Voting,
        is_executer_a_member: bool,
    ) -> bool {
        voting.payload.iter().all(|p| {
            let config = self.custom.get(&p.endpoint).unwrap_or(&self.default);

            match &config.can_execute {
                VotingCharacter::All => true,
                VotingCharacter::Member => is_executer_a_member,
                VotingCharacter::Exact(members) => members.contains(executer),
            }
        })
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingManager {
    pub votings: HashMap<Principal, Vec<Voting>>,
    pub membership_guards: HashMap<Principal, Controlled<Option<Principal>>>,
    pub voting_config_types: HashMap<Principal, Controlled<VotingConfigType>>,
    pub voting_configs: HashMap<Principal, Controlled<VotingConfig>>,

    pub event_listeners: HashMap<VotingEventType, HashSet<RemoteCallEndpoint>>,
}

impl VotingManager {
    pub fn create_voting(
        &mut self,
        proposer: Principal,
        timestamp: i64,
        params: NewVotingParams,
        is_proposer_a_member: bool,
    ) -> Result<VotingId, Error> {
        let config = self
            .voting_configs
            .get(&params.union_wallet)
            .ok_or(Error::VotingConfigDoesNotExist)?;

        if !config
            .data
            .is_allowed_to_create(&params, &proposer, is_proposer_a_member)
        {
            return Err(Error::VotingIsRejected); // TODO: another error here please
        }

        let voting = Voting::new(proposer, timestamp, params);

        let votings = match self.votings.get_mut(&params.union_wallet) {
            None => {
                self.votings.insert(params.union_wallet.clone(), Vec::new());
                self.votings.get_mut(&params.union_wallet).unwrap()
            }
            Some(v) => v,
        };

        let idx = votings.len();
        votings.push(voting);

        Ok(VotingId {
            union_wallet: params.union_wallet.clone(),
            idx,
        })
    }

    pub fn delete_voting(
        &mut self,
        voting_id: VotingId,
        caller: Principal,
        is_caller_a_member: bool,
    ) -> Result<Voting, Error> {
        let votings = self
            .votings
            .get_mut(&voting_id.union_wallet)
            .ok_or(Error::VotingDoesNotExist)?;

        if votings.len() <= voting_id.idx {
            Err(Error::VotingDoesNotExist)
        } else {
            let voting = votings.get(voting_id.idx).unwrap();

            let config = self
                .voting_configs
                .get(&voting.union_wallet)
                .ok_or(Error::VotingConfigDoesNotExist)?;

            if config
                .data
                .is_allowed_to_delete(&caller, voting, is_caller_a_member)
            {
                Ok(votings.remove(voting_id.idx))
            } else {
                Err(Error::VotingConfigDoesNotExist) // TODO: we need another error here
            }
        }
    }

    pub fn update_voting(
        &mut self,
        voting_id: VotingId,
        params: UpdateVotingParams,
        timestamp: i64,
        caller: Principal,
    ) -> Result<Voting, Error> {
        let voting = self.get_voting_mut(voting_id)?;

        let config = self
            .voting_configs
            .get(&voting.union_wallet)
            .ok_or(Error::VotingConfigDoesNotExist)?;

        if config.data.is_allowed_to_update(&params, &caller, voting) {
            voting.update(params, timestamp)?;

            Ok(voting.clone())
        } else {
            Err(Error::VotingConfigDoesNotExist) // TODO: we need another error here
        }
    }

    pub fn vote(
        &mut self,
        voting_id: VotingId,
        voter: &Principal,
        vote_voting_power: u64,
        total_voting_power: u64,
        vote: Vote,
        timestamp: i64,
    ) -> Result<(), Error> {
        let voting = self.get_voting_mut(voting_id)?;

        let config = self
            .voting_configs
            .get(&voting.union_wallet)
            .ok_or(Error::VotingConfigDoesNotExist)?;

        if config.data.is_allowed_to_vote(voter, voting) {
            voting.vote(
                voter,
                vote_voting_power,
                total_voting_power,
                vote,
                timestamp,
            );

            Ok(())
        } else {
            Err(Error::VotingIsRejected) // TODO: another error here
        }
    }

    pub fn execute(
        &mut self,
        voting_id: VotingId,
        timestamp: i64,
        caller: Principal,
        is_caller_a_member: bool,
    ) -> Result<(), Error> {
        let voting = self.get_voting_mut(voting_id)?;

        let config = self
            .voting_configs
            .get(&voting.union_wallet)
            .ok_or(Error::VotingConfigDoesNotExist)?;

        if config
            .data
            .is_allowed_to_execute(&caller, voting, is_caller_a_member)
        {
            voting.execute(timestamp);
            Ok(())
        } else {
            Err(Error::VotingIsRejected) // TODO: another error please
        }
    }

    pub fn get_voting_mut(&mut self, id: VotingId) -> Result<&mut Voting, Error> {
        let votings = self
            .votings
            .get_mut(&id.union_wallet)
            .map_or(Err(Error::VotingDoesNotExist), |v| Ok(v))?;

        votings
            .get_mut(id.idx)
            .map_or(Err(Error::VotingDoesNotExist), |v| Ok(v))
    }

    pub fn get_listeners(&self, event_type: VotingEventType) -> Vec<RemoteCallEndpoint> {
        self.event_listeners
            .get(&event_type)
            .map_or(Vec::new(), |s| s.clone().into_iter().collect())
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingCreatedEventPayload {
    pub id: VotingId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotingUpdatedEventPayload {
    pub id: VotingId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StatusChangedEventPayload {
    pub id: VotingId,
    pub status: VotingStatus,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VotePlacedEventPayload {
    pub id: VotingId,
    pub vote: Vote,
    pub voting_power: u64,
    pub voter: Principal,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug, CandidType, Deserialize)]
pub enum VotingEventType {
    VotingCreated,
    VotingUpdated,
    StatusChanged,
    VotePlaced,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum VotingEvent {
    VotingCreated(VotingCreatedEventPayload),
    VotingUpdated(VotingUpdatedEventPayload),
    StateChanged(StatusChangedEventPayload),
    VotePlaced(VotePlacedEventPayload),
}
