use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use union_utils::types::{
    Account, OnMoveListener, OnMoveListenerError, OnMoveListenersInfo, TokenMoveEvent,
    TokenMoveEventAndListeners,
};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Controllers {
    pub mint_controller: Account,
    pub on_move_controller: Account,
    pub info_controller: Account,
}

impl Controllers {
    pub fn single(controller: Account) -> Controllers {
        Controllers {
            mint_controller: controller,
            on_move_controller: controller.clone(),
            info_controller: controller.clone(),
        }
    }
}

/*
 type FungibleTokenInfo = record {
   name : Text;
   symbol : Text;
   decimals : nat8;
 }
*/
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct FungibleTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct FungibleToken {
    pub balances: HashMap<Principal, u64>,
    pub total_supply: u64,
    pub info: FungibleTokenInfo,
    pub on_move_listeners: OnMoveListenersInfo,
    pub controllers: Controllers,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Error {
    InsufficientBalance,
    AccessDenied,
    ForbiddenOperation,
    ListenerError(OnMoveListenerError),
}

impl FungibleToken {
    pub fn mint(
        &mut self,
        to: Principal,
        qty: u64,
        caller: Principal,
    ) -> Result<TokenMoveEventAndListeners, Error> {
        check_controlled_op(self.controllers.mint_controller.clone(), caller);

        let prev_balance = self.balance_of(&to);

        self.total_supply += qty;
        self.balances.insert(to, prev_balance + qty);

        Ok(self.create_event_and_find_listeners(Account::None, Account::Some(to.clone()), qty))
    }

    pub fn send(
        &mut self,
        from: Principal,
        to: Principal,
        qty: u64,
    ) -> Result<TokenMoveEventAndListeners, Error> {
        let from_prev_balance = self.balance_of(&from);
        let to_prev_balance = self.balance_of(&to);

        if from_prev_balance < qty {
            return Err(Error::InsufficientBalance);
        }

        self.balances.insert(from, from_prev_balance - qty);
        self.balances.insert(to, to_prev_balance + qty);

        Ok(self.create_event_and_find_listeners(
            Account::Some(from.clone()),
            Account::Some(to.clone()),
            qty,
        ))
    }

    pub fn burn(&mut self, from: Principal, qty: u64) -> Result<TokenMoveEventAndListeners, Error> {
        let prev_balance = self.balance_of(&from);

        if prev_balance < qty {
            return Err(Error::InsufficientBalance);
        }

        self.total_supply -= qty;
        self.balances.insert(from, prev_balance - qty);

        Ok(self.create_event_and_find_listeners(Account::Some(from.clone()), Account::None, qty))
    }

    pub fn subscribe_on_move(
        &mut self,
        listener: OnMoveListener,
        caller: Principal,
    ) -> Result<u64, Error> {
        check_controlled_op(self.controllers.on_move_controller.clone(), caller);

        self.on_move_listeners
            .add_listener(listener)
            .map_err(|e| Error::ListenerError(e))
    }

    pub fn unsubscribe_on_move(
        &mut self,
        id: u64,
        caller: Principal,
    ) -> Result<OnMoveListener, Error> {
        check_controlled_op(self.controllers.on_move_controller.clone(), caller);

        self.on_move_listeners
            .remove_listener(id)
            .map_err(|e| Error::ListenerError(e))
    }

    pub fn update_info(
        &mut self,
        new_info: FungibleTokenInfo,
        caller: Principal,
    ) -> Result<FungibleTokenInfo, Error> {
        check_controlled_op(self.controllers.info_controller.clone(), caller)?;

        let old_info = self.info.clone();
        self.info = new_info;

        Ok(old_info)
    }

    pub fn update_mint_controller(
        &mut self,
        new_mint_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.mint_controller.clone(), caller);

        self.controllers.mint_controller = new_mint_controller;

        Ok(())
    }

    pub fn update_on_move_controller(
        &mut self,
        new_on_move_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.on_move_controller.clone(), caller);

        self.controllers.on_move_controller = new_on_move_controller;

        Ok(())
    }

    pub fn update_info_controller(
        &mut self,
        new_info_controller: Account,
        caller: Principal,
    ) -> Result<(), Error> {
        check_controlled_op(self.controllers.info_controller.clone(), caller)?;

        self.controllers.info_controller = new_info_controller;

        Ok(())
    }

    pub fn balance_of(&self, token_holder: &Principal) -> u64 {
        match self.balances.get(&token_holder) {
            None => 0,
            Some(b) => b.clone(),
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
