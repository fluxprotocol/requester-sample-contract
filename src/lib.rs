use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde_json::json;
use near_sdk::json_types::{U64, ValidAccountId};
use near_sdk::collections::{UnorderedSet, LookupMap};
use fungible_token_handler::fungible_token_transfer_call;
use flux_sdk::{RequestStatus, Nonce, DataRequestDetails, WrappedBalance, NewDataRequestArgs,Outcome};

mod fungible_token_handler;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RequesterContract {
    pub oracle: AccountId,
    pub payment_token: AccountId,
    pub nonce: Nonce,
    pub data_requests: LookupMap<u64, DataRequestDetails>,
    pub whitelist: UnorderedSet<AccountId> // accounts allowed to call create_data_request(). if len() == 0, no whitelist (any account can make data request)
}

impl Default for RequesterContract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

// Private methods
impl RequesterContract {
    pub fn assert_oracle(&self) {
        assert_eq!(&env::predecessor_account_id(), &self.oracle, "ERR_INVALID_ORACLE_ADDRESS");
    }
    // if whitelist is populated, make sure caller's account is included in it
    pub fn assert_whitelisted(&self) {
        if self.whitelist.len() > 0 {
            assert!(
                self.whitelist.contains(&env::predecessor_account_id()),
                "ERR_NOT_WHITELISTED"
            )
        }
    }

}

#[near_bindgen]
impl RequesterContract {
    #[init]
    pub fn new(
        oracle: AccountId,
        payment_token: AccountId,
        whitelist: Option<Vec<ValidAccountId>>,
    ) -> Self {
        let mut requester_instance = Self {
            oracle,
            payment_token,
            nonce: Nonce::new(),
            data_requests: LookupMap::new(b"drq".to_vec()),
            whitelist: UnorderedSet::new(b"w".to_vec())
        };

        // populate whitelist
        if let Some(whitelist) = whitelist {
            for acct in whitelist {
                requester_instance.whitelist.insert(acct.as_ref());
            }
        }

        requester_instance
    }

    #[payable]
    pub fn create_data_request(
        &mut self,
        amount: WrappedBalance,
        payload: NewDataRequestArgs
    ) -> Promise {
        self.assert_whitelisted();
        let request_id = self.nonce.get_and_incr();

        // insert request_id into tags
        let mut payload = payload;
        let mut tags = payload.tags;
        tags.push(request_id.to_string());
        payload.tags = tags.to_vec();

        let dr = DataRequestDetails {
            amount,
            payload: payload.clone(),
            tags: tags,
            status: RequestStatus::Pending
        };
        self.data_requests.insert(&request_id, &dr);
        log!("storing data request under {}", request_id);
        fungible_token_transfer_call(
            self.payment_token.clone(),
            self.oracle.clone(),
            amount.into(),
            json!({"NewDataRequest": payload}).to_string() 
        )
    }

    #[payable]
    pub fn set_outcome(
        &mut self,
        requestor: AccountId,
        outcome: Outcome,
        tags: Vec<String>,
    ) {
        self.assert_oracle();
        assert_eq!(env::current_account_id(), requestor, "can only set outcomes for requests that are initiated by this requester");
        assert_eq!(env::attached_deposit(), 1);

        let request_id = tags.last().unwrap().parse::<u64>().unwrap();
        let mut request = self.data_requests.get(&request_id).unwrap();
        request.status = RequestStatus::Finalized(outcome);
        self.data_requests.insert(&request_id, &request);
    }

    pub fn get_data_request(&self, request_id: U64) -> Option<DataRequestDetails> {
        self.data_requests.get(&u64::from(request_id))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::serde_json;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::json_types::U128;
    use flux_sdk::DataRequestDataType;

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn oracle() -> AccountId {
        "oracle.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: alice(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: alice(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 10000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_ORACLE_ADDRESS")]
    fn ri_not_oracle() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let contract = RequesterContract::new(
            oracle(),
            token(),
            None,
        );
        contract.request_ft_transfer(
            token(),
            100,
            alice()
        );
    }

    #[test]
    fn ri_create_dr_success() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            None,
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String,
        });
    }


    #[test]
    fn ri_whitelisted_success() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            Some(vec![serde_json::from_str("\"alice.near\"").unwrap()])
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String,
        });
    }

    #[test]
    #[should_panic(expected = "ERR_NOT_WHITELISTED")]
    fn ri_unwhitelisted_fail() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            Some(vec![serde_json::from_str("\"bob.near\"").unwrap()])
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String
        });
    }

    #[test]
    fn ri_empty_tags_nonce_works() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            Some(vec![serde_json::from_str("\"alice.near\"").unwrap()])
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String
        });

        assert!(contract.data_requests.get(&0).is_some());
    }

    #[test]
    fn ri_some_tags_nonce_works() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            Some(vec![serde_json::from_str("\"alice.near\"").unwrap()])
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: vec!["butt".to_owned(),"on".to_owned()],
            data_type: DataRequestDataType::String
        });

        assert!(contract.data_requests.get(&0).is_some());
    }

    #[test]
    fn ri_nonce_iterates_properly() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = RequesterContract::new(
            oracle(),
            token(),
            Some(vec![serde_json::from_str("\"alice.near\"").unwrap()])
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String
        });

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Some(Vec::new()),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            description: Some("a".to_string()),
            tags: Vec::new(),
            data_type: DataRequestDataType::String
        });

        assert!(contract.data_requests.get(&1).is_some());
    }
}
