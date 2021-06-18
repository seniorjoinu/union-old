use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use union_utils::types::{
    Account, OnMoveListener, OnMoveListenerError, OnMoveListenersInfo, TokenMoveEvent,
    TokenMoveEventAndListeners,
};

/*
 type Controllers = record {
   issue_controller : Account;
   revoke_controller : Account;
   on_move_controller : Account;
   info_controller : Account;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Controllers {
    pub issue_controller: Account,
    pub revoke_controller: Account,
    pub on_move_controller: Account,
    pub info_controller: Account,
}

impl Controllers {
    pub fn single(controller: Account) -> Controllers {
        Controllers {
            issue_controller: controller,
            revoke_controller: controller,
            on_move_controller: controller,
            info_controller: controller,
        }
    }
}

/*
 type ClaimTokenInitPayload {
   info : ClaimTokenInfo;
   controllers : variant { None; Some : Controllers; };
   on_move_listeners : vec OnMoveListener;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClaimTokenInitPayload {
    pub info: ClaimTokenInfo,
    pub controllers: Option<Controllers>,
    pub on_move_listeners: Vec<OnMoveListener>,
}

/*
 type ClaimTokenInfo = record {
   name : Text;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClaimTokenInfo {
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClaimToken {
    pub claims: HashMap<Principal, bool>,
    pub total_supply: u64,
    pub info: ClaimTokenInfo,
    pub on_move_listeners: OnMoveListenersInfo,
    pub controllers: Controllers,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Error {
    AlreadyHasClaim,
    DoesNotHaveClaimYet,
    AccessDenied,
    ForbiddenOperation,
    ListenerError(OnMoveListenerError),
}

impl ClaimToken {
    pub fn issue(
        &mut self,
        to: Principal,
        caller: Principal,
    ) -> Result<TokenMoveEventAndListeners, Error> {
        check_controlled_op(self.controllers.issue_controller, caller)?;

        if self.has_claim(&to) {
            return Err(Error::AlreadyHasClaim);
        }

        self.claims.insert(to, true);
        self.total_supply += 1;

        Ok(self.create_event_and_find_listeners(Account::None, Account::Some(to), 1))
    }

    pub fn revoke(
        &mut self,
        from: Principal,
        caller: Principal,
    ) -> Result<TokenMoveEventAndListeners, Error> {
        check_controlled_op(self.controllers.revoke_controller, caller)?;

        if !self.has_claim(&from) {
            return Err(Error::DoesNotHaveClaimYet);
        }

        self.claims.insert(from, false);
        self.total_supply -= 1;

        Ok(self.create_event_and_find_listeners(Account::Some(from), Account::None, 1))
    }

    pub fn subscribe_on_move(
        &mut self,
        listener: OnMoveListener,
        caller: Principal,
    ) -> Result<u64, Error> {
        check_controlled_op(self.controllers.on_move_controller, caller)?;

        self.on_move_listeners
            .add_listener(listener)
            .map_err(Error::ListenerError)
    }

    pub fn unsubscribe_on_move(
        &mut self,
        id: u64,
        caller: Principal,
    ) -> Result<OnMoveListener, Error> {
        check_controlled_op(self.controllers.on_move_controller, caller)?;

        self.on_move_listeners
            .remove_listener(id)
            .map_err(Error::ListenerError)
    }

    pub fn update_info(
        &mut self,
        new_info: ClaimTokenInfo,
        caller: Principal,
    ) -> Result<ClaimTokenInfo, Error> {
        check_controlled_op(self.controllers.info_controller, caller)?;

        let old_info = self.info.clone();
        self.info = new_info;

        Ok(old_info)
    }

    pub fn update_issue_controller(
        &mut self,
        new_issue_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.issue_controller, caller)?;

        self.controllers.issue_controller = new_issue_controller;

        Ok(())
    }

    pub fn update_revoke_controller(
        &mut self,
        new_revoke_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.revoke_controller, caller)?;

        self.controllers.revoke_controller = new_revoke_controller;

        Ok(())
    }

    pub fn update_on_move_controller(
        &mut self,
        new_on_move_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.on_move_controller, caller)?;

        self.controllers.on_move_controller = new_on_move_controller;

        Ok(())
    }

    pub fn update_info_controller(
        &mut self,
        new_info_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.info_controller, caller)?;

        self.controllers.info_controller = new_info_controller;

        Ok(())
    }

    pub fn has_claim(&self, holder: &Principal) -> bool {
        match self.claims.get(&holder) {
            None => false,
            Some(b) => *b,
        }
    }

    fn create_event_and_find_listeners(
        &self,
        from: Account,
        to: Account,
        qty: u64,
    ) -> TokenMoveEventAndListeners {
        let event = TokenMoveEvent { from, to, qty };

        TokenMoveEventAndListeners {
            event: event.clone(),
            listeners: self.on_move_listeners.get_matching_listeners(&event),
        }
    }
}

fn check_controlled_op(controller: Account, caller: Principal) -> Result<(), Error> {
    if let Some(c) = controller {
        if caller != c {
            return Err(Error::AccessDenied);
        }
    } else {
        return Err(Error::ForbiddenOperation);
    }

    Ok(())
}
