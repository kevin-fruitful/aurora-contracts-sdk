use near_sdk::json_types::{Base58CryptoHash, U128};
use near_sdk::serde::{Deserialize, Serialize};

/// Input parameters for Entity struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Entity {
    pub asset_id: Base58CryptoHash,
    pub collateral_ratio: U128,
    pub max_capacity: U128,
    pub utilized_capacity: U128,
    pub simple_policy_enabled: bool,
}
