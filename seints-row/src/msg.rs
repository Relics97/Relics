use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

/// Message to instantiate the contract.
/// Defines the initial configuration, including token details and distribution addresses.
#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: Uint128,
    pub team_address: String,
    pub pool_address: String,
    pub metadata_url: String,
}

/// Messages that can modify the contract's state.
#[cw_serde]
pub enum ExecuteMsg {
    /// Transfers tokens from the sender to a recipient.
    Transfer { recipient: String, amount: Uint128 },
    /// Burns tokens from the sender's balance.
    Burn { amount: Uint128 },
    /// Releases vested tokens for the sender.
    ReleaseVested {},
    /// Releases pool tokens for the sender.
    ReleasePool {},
    /// Updates the metadata URL (only callable by the owner).
    UpdateMetadata { metadata_url: String },
}

/// Queries that can read the contract's state.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns information about the token (name, symbol, decimals, total supply, owner).
    #[returns(TokenInfoResponse)]
    GetTokenInfo {},
    /// Returns the balance of a specific address.
    #[returns(Uint128)]
    GetBalance { address: String },
    /// Returns vesting information for a specific address.
    #[returns(VestingInfoResponse)]
    GetVestingInfo {
        address: String,
        start_after: Option<u64>, // Optional timestamp to start after
        limit: Option<u32>,      // Optional limit on the number of results
    },
    /// Returns pool release information for a specific address.
    #[returns(PoolReleaseInfoResponse)]
    GetPoolReleaseInfo {
        address: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

/// Response for the `GetTokenInfo` query.
#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub owner: String,
}

/// Response for the `GetVestingInfo` query.
#[cw_serde]
pub struct VestingInfoResponse {
    pub amount: Uint128,
    pub start_time: u64,
    pub release_schedule: Vec<(u64, Uint128)>,
}

/// Response for the `GetPoolReleaseInfo` query.
#[cw_serde]
pub struct PoolReleaseInfoResponse {
    pub amount: Uint128,
    pub release_schedule: Vec<(u64, Uint128)>,
}
pub struct MetadataResponse {
    pub metadata_url: String,
}

pub struct GetCountResponse {
    pub count: u64,
}