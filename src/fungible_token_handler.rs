use crate::*;

use flux_sdk::consts::{DR_NEW_GAS, GAS_BASE_TRANSFER};
use near_sdk::{ext_contract, json_types::U128, serde_json, AccountId, Promise, PromiseOrValue};

#[ext_contract(fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> Promise;
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> Promise;
    fn ft_balance_of(&self, account_id: AccountId) -> Promise;
}

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

pub fn fungible_token_transfer(
    token_account_id: AccountId,
    receiver_id: AccountId,
    value: u128,
) -> Promise {
    fungible_token::ft_transfer(
        receiver_id,
        U128(value),
        None,
        // Near params
        &token_account_id,
        1,
        GAS_BASE_TRANSFER,
    )
}

pub fn fungible_token_transfer_call(
    token_account_id: AccountId,
    receiver_id: AccountId,
    value: u128,
    msg: String,
) -> Promise {
    fungible_token::ft_transfer_call(
        receiver_id,
        U128(value),
        None,
        msg,
        // Near params
        &token_account_id,
        1,
        DR_NEW_GAS,
    )
}

#[near_bindgen]
impl RequesterContract {
    pub fn request_ft_transfer(
        &self,
        token_id: AccountId,
        amount: Balance,
        receiver_id: AccountId,
    ) -> Promise {
        self.assert_caller(&self.oracle);
        assert_eq!(
            self.payment_token.clone(),
            token_id,
            "ERR_INVALID_PAYMENT_TOKEN"
        );
        fungible_token_transfer(self.payment_token.clone(), receiver_id.clone(), amount)
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for RequesterContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<WrappedBalance> {
        let payload: NewDataRequestArgs =
            serde_json::from_str(&msg).expect("Failed to parse the payload, invalid `msg` format");
        self.assert_whitelisted(&sender_id.clone().into()); // if whitelist is set, make sure sender can call create_data_request()
        PromiseOrValue::Promise(self.create_data_request(amount.into(), sender_id.into(), payload))
    }
}
