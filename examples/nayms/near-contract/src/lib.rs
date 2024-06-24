use aurora_sdk::{
    ethabi, near_sdk, Address, CallArgsWrapper, FixedBytes, FunctionCallArgsV1, SubmitResult,
    TransactionStatus, U256,
};
use nayms_on_near_types::Entity;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

const GET_ENTITY_SELECTOR: [u8; 4] = [0x63, 0x5e, 0xe5, 0x6d];

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct NaymsProxy {
    aurora: AccountId,
    nayms: Address,
}

#[near_bindgen]
impl NaymsProxy {
    #[init]
    pub fn new(aurora: AccountId, nayms_address: String) -> Self {
        Self {
            aurora,
            nayms: aurora_sdk::parse_address(&nayms_address).unwrap(),
        }
    }

    pub fn get_entity(&self, entity_id: String) -> Promise {
        let evm_input =
            ethabi::encode(&[ethabi::Token::FixedBytes(hex::decode(entity_id).unwrap())]);
        let aurora_call_args = CallArgsWrapper::V1(FunctionCallArgsV1 {
            contract: self.nayms,
            input: [GET_ENTITY_SELECTOR.as_slice(), evm_input.as_slice()].concat(),
        });
        aurora_sdk::aurora_contract::ext(self.aurora.clone())
            .with_unused_gas_weight(3)
            .call(aurora_call_args)
            .then(Self::ext(env::current_account_id()).parse_entity_result())
    }

    #[private]
    pub fn parse_entity_result(&self, #[callback_unwrap] result: SubmitResult) -> Entity {
        match result.status {
            TransactionStatus::Succeed(bytes) => {
                let entity_id = FixedBytes::from_slice(&bytes);
                Entity {
                    asset_id: entity_id,
                    collateral_ratio: U256::zero(), // You may need to adjust these default values
                    max_capacity: U256::zero(),
                    utilized_capacity: U256::zero(),
                    simple_policy_enabled: false,
                }
            }
            TransactionStatus::Revert(bytes) => {
                let error_message =
                    format!("Revert: {}", aurora_sdk::parse_evm_revert_message(&bytes));
                env::panic_str(&error_message)
            }
            other => env::panic_str(&format!("Aurora Error: {other:?}")),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub asset_id: FixedBytes,
    pub collateral_ratio: U256,
    pub max_capacity: U256,
    pub utilized_capacity: U256,
    pub simple_policy_enabled: bool,
}
