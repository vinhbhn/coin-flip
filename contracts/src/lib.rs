use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::UnorderedMap,
    env,
    json_types::{Base58PublicKey, U128},
    near_bindgen, AccountId, Balance, Promise, PublicKey,
};
use serde::Serialize;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
const PROB: u8 = 128;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotMachine {
    pub owner_id: AccountId,
    pub credits: UnorderedMap<AccountId, Balance>,
}

impl Default for SlotMachine {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SlotMachine {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Invalid owner account"
        );
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            credits: UnorderedMap::new(b"credits".to_vec()),
        }
    }

    #[payable]
    pub fn deposit(&mut self) {
        let account_id = env::signer_account_id();
        let deposit = env::attached_deposit();
        let mut credits = self.credits.get(&account_id).unwrap_or(0);
        credits = credits + deposit;
        self.credits.insert(&account_id, &credits);
    }

    pub fn play(&mut self) -> u8 {
        let account_id = env::signer_account_id();
        let mut credits = self.credits.get(&account_id).unwrap_or(0);
        assert!(credits > 0, "no credits to play");
        credits = credits - ONE_NEAR;

        let rand: u8 = *env::random_seed().get(0).unwrap();
        if rand < PROB {
            credits = credits + 10 * ONE_NEAR;
        }

        self.credits.insert(&account_id, &credits);
        rand
    }

    pub fn get_credits(&self, account_id: AccountId) -> u128 {
        self.credits.get(&account_id).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{testing_env, MockedBlockchain, VMContext};

    fn get_context() -> VMContext {
        VMContext {
            predecessor_account_id: "alice.testnet".to_string(),
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "bob.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
            storage_usage: 0,
        }
    }

    #[test]
    fn deposit_works() {
        let mut context = get_context();
        testing_env!(context.clone());
        let mut contract = SlotMachine::new(context.current_account_id.clone());

        context.attached_deposit = 2 * ONE_NEAR;
        testing_env!(context.clone());

        contract.deposit();
        let credit = contract.get_credits(context.signer_account_id.clone());

        assert_eq!(credit, 2 * ONE_NEAR);
    }

    #[test]
    fn play_works() {
        let mut context = get_context();
        testing_env!(context.clone());
        let mut contract = SlotMachine::new(context.current_account_id.clone());

        context.attached_deposit = 2 * ONE_NEAR;
        testing_env!(context.clone());

        contract.deposit();
        let credit = contract.get_credits(context.signer_account_id.clone());

        assert_eq!(credit, 2 * ONE_NEAR);

        let rand = contract.play();
        let credit = contract.get_credits(context.signer_account_id.clone());
        if rand < PROB {
            assert_eq!(credit, 11 * ONE_NEAR);
        } else {
            assert_eq!(credit, ONE_NEAR);
        }
    }
}
