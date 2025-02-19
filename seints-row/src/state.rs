use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


/// Represents the global information about the token, including its name, symbol, decimals, total supply, and owner.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub owner: Addr,
}

/// Represents vesting information for the owner, including the total amount, start time, and release schedule.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    pub amount: Uint128,
    pub start_time: Timestamp,
    pub release_schedule: Vec<(Timestamp, Uint128)>,
}

/// Represents gradual release information for the pool, including the total amount and release schedule.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolReleaseInfo {
    pub amount: Uint128,
    pub release_schedule: Vec<(Timestamp, Uint128)>,
}

// Token information
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");

// Balances of token holders
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balances");

// Vesting information for the owner, with optional pagination support
pub const VESTING: Map<(&Addr, Option<u64>, Option<u32>), VestingInfo> = Map::new("vesting");

// Gradual release schedule for the pool, with optional pagination support
pub const POOL_RELEASE_SCHEDULE: Map<(&Addr, Option<u64>, Option<u32>), PoolReleaseInfo> =
    Map::new("pool_release_schedule");

// Metadata URL for the token
pub const METADATA_URL: Item<String> = Item::new("metadata_url");
pub struct VestingInfo {
    pub amount: Uint128,
    pub start_time: Timestamp,
    pub release_schedule: Vec<(Timestamp, Uint128)>,
}

pub struct PoolReleaseInfo {
    pub amount: Uint128,
    pub release_schedule: Vec<(Timestamp, Uint128)>,
}