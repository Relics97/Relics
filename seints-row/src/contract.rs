#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Timestamp, Addr, Map,
};
use cw2::set_contract_version;
use url::Url;
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse, VestingInfoResponse, PoolReleaseInfoResponse,
    MetadataResponse,
};
use crate::state::{TokenInfo, TOKEN_INFO, BALANCES, VESTING, POOL_RELEASE_SCHEDULE, METADATA_URL, VestingInfo, PoolReleaseInfo};
// Version info for migration
const CONTRACT_NAME: &str = "crates.io:seints-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const VESTING: Map<Addr, VestingInfo> = Map::new("vesting");
const POOL_RELEASE_SCHEDULE: Map<Addr, PoolReleaseInfo> = Map::new("pool_release_schedule");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Validate the token info
    if msg.decimals > 18 {
        return Err(ContractError::InvalidDecimals {});
    }

    // Ensure the initial supply is exactly 1 billion
    let one_billion = Uint128::new(1_000_000_000);
    if msg.initial_supply != one_billion {
        return Err(ContractError::InvalidInitialSupply {
            expected: one_billion,
            actual: msg.initial_supply,
        });
    }

    // Validate addresses
    let team_address = deps.api.addr_validate(&msg.team_address)?;
    let pool_address = deps.api.addr_validate(&msg.pool_address)?;

    if team_address == pool_address {
        return Err(ContractError::DuplicateAddresses {});
    }

    // Calculate distribution amounts
    let team_amount = msg.initial_supply.multiply_ratio(20u128, 100u128); // 20%
    let pool_amount = msg.initial_supply.multiply_ratio(50u128, 100u128); // 50%
    let owner_amount = msg.initial_supply.multiply_ratio(30u128, 100u128); // 30%

    // Save token info
    let token_info = TokenInfo {
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        decimals: msg.decimals,
        total_supply: msg.initial_supply,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // Save the metadata URL
    if !is_valid_url(&msg.metadata_url) {
        return Err(ContractError::InvalidMetadataUrl {});
    }
    METADATA_URL.save(deps.storage, &msg.metadata_url)?;

    // Mint 20% to the team
    BALANCES.update(deps.storage, &team_address, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + team_amount)
    })?;

    // Mint 40% of the pool's tokens upfront
    let pool_upfront_amount = pool_amount.multiply_ratio(40u128, 50u128); // 40% of 50%
    BALANCES.update(deps.storage, &pool_address, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + pool_upfront_amount)
    })?;

    // Lock 30% for the owner (vesting)
    let start_time = env.block.time;
    let release_schedule = vec![
        (start_time.plus_years(1), owner_amount.multiply_ratio(10u128, 100u128)), // 10% after 1 year
        (start_time.plus_years(2), owner_amount.multiply_ratio(10u128, 100u128)), // 10% after 2 years
        (start_time.plus_years(3), owner_amount.multiply_ratio(10u128, 100u128)), // 10% after 3 years
    ];
    let vesting_info = VestingInfo {
        amount: owner_amount,
        start_time,
        release_schedule,
    };
    VESTING.save(deps.storage, &info.sender, &vesting_info)?;

    // Set up gradual release for the remaining 10% of the pool
    let pool_gradual_amount = pool_amount.multiply_ratio(10u128, 50u128); // 10% of 50%
    let pool_release_schedule = vec![
        (start_time.plus_months(6), pool_gradual_amount.multiply_ratio(5u128, 10u128)), // 5% after 6 months
        (start_time.plus_months(12), pool_gradual_amount.multiply_ratio(25u128, 100u128)), // 2.5% after 12 months
        (start_time.plus_months(18), pool_gradual_amount.multiply_ratio(25u128, 100u128)), // 2.5% after 18 months
    ];
    let pool_release_info = PoolReleaseInfo {
        amount: pool_gradual_amount,
        release_schedule: pool_release_schedule,
    };
    POOL_RELEASE_SCHEDULE.save(deps.storage, &pool_address, &pool_release_info)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("total_supply", msg.initial_supply)
        .add_attribute("metadata_url", msg.metadata_url))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => execute::transfer(deps, info, recipient, amount),
        ExecuteMsg::Burn { amount } => execute::burn(deps, info, amount),
        ExecuteMsg::ReleaseVested {} => execute::release_vested(deps, env, info),
        ExecuteMsg::ReleasePool {} => execute::release_pool(deps, env, info),
        ExecuteMsg::UpdateMetadata { metadata_url } => execute::update_metadata(deps, info, metadata_url),
    }
}

pub mod execute {
    use super::*;

    pub fn transfer(
        deps: DepsMut,
        info: MessageInfo,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let recipient_addr = deps.api.addr_validate(&recipient)?;

        // Deduct tokens from sender first
        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            let balance = balance.unwrap_or_default();
            if balance < amount {
                return Err(StdError::generic_err("Insuficient balance"));
            }
            Ok(balance - amount)
        })?;

        // Add tokens to recipient afterward
        BALANCES.update(deps.storage, &recipient_addr, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + amount)
        })?;

        Ok(Response::new()
            .add_attribute("method", "transfer")
            .add_attribute("from", info.sender)
            .add_attribute("to", recipient)
            .add_attribute("amount", amount))
    }

    pub fn burn(
        deps: DepsMut,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // Deduct the tokens from the sender's balance
        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            let balance = balance.unwrap_or_default();
            if balance < amount {
                return Err(ContractError::InsufficientBalance {});
            }
            Ok(balance - amount)
        })?;

        // Reduce the total supply
        TOKEN_INFO.update(deps.storage, |mut token_info| -> StdResult<_> {
            token_info.total_supply = token_info
            .total_supply
            .checked_sub(amount)
            .map_err(|_| ContractError::Overflow {})?;
            Ok(token_info)  
        })?;

        Ok(Response::new()
            .add_attribute("method", "burn")
            .add_attribute("from", info.sender)
            .add_attribute("amount", amount))
    }

    pub fn release_vested(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut vesting_info = VESTING.load(deps.storage, &info.sender)?;
        let mut total_released = Uint128::zero();

        // Iterate through the release schedule
        let mut last_processed_time = vesting_info.last_processed_time.unwrap_or(vesting_info.start_time);
        for (timestamp, amount) in vesting_info.release_schedule.iter() {
            if *timestamp > last_processed_time && env.block.time >= *timestamp {
                total_released += *amount;
                last_processed_time = *timestamp;
            }
        }
        vesting_info.last_processed_time = Some(last_processed_time);

        // Remove released amounts from the schedule
        vesting_info.release_schedule.retain(|(timestamp, _)| env.block.time < *timestamp);

        // Update vesting info
        vesting_info.amount -= total_released;
        VESTING.save(deps.storage, &info.sender, &vesting_info)?;

        // Transfer released tokens to the owner
        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + total_released)
        })?;

        Ok(Response::new()
            .add_attribute("method", "release_vested")
            .add_attribute("amount", total_released))
    }

    pub fn release_pool(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut pool_release_info = POOL_RELEASE_SCHEDULE.load(deps.storage, &info.sender)?;
        let mut total_released = Uint128::zero();

        // Iterate through the release schedule
        let mut last_processed_time = pool_release_info.last_processed_time.unwrap_or(pool_release_info.start_time);
        for (timestamp, amount) in pool_release_info.release_schedule.iter() {
            if *timestamp > last_processed_time && env.block.time >= *timestamp {
                total_released += *amount;
                last_processed_time = *timestamp;
            }
        }
        pool_release_info.last_processed_time = Some(last_processed_time);

        // Remove released amounts from the schedule
        pool_release_info.release_schedule.retain(|(timestamp, _)| env.block.time < *timestamp);

        // Update pool release info
        pool_release_info.amount -= total_released;
        POOL_RELEASE_SCHEDULE.save(deps.storage, &info.sender, &pool_release_info)?;

        // Transfer released tokens to the pool
        BALANCES.update(deps.storage, &info.sender, |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + total_released)
        })?;

        Ok(Response::new()
            .add_attribute("method", "release_pool")
            .add_attribute("amount", total_released))
    }

    pub fn update_metadata(
        deps: DepsMut,
        info: MessageInfo,
        metadata_url: String,
    ) -> Result<Response, ContractError> {
        // Ensure only the owner can update the metadata
        let token_info = TOKEN_INFO.load(deps.storage)?;
        if info.sender != token_info.owner {
            return Err(ContractError::Unauthorized {});
        }

        // Validate the metadata URL format
        if !is_valid_url(&metadata_url) {
            return Err(ContractError::InvalidMetadataUrl {});
        }

        // Update the metadata URL
        METADATA_URL.save(deps.storage, &metadata_url)?;

        Ok(Response::new()
            .add_attribute("method", "update_metadata")
            .add_attribute("metadata_url", metadata_url))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTokenInfo {} => to_json_binary(&query::token_info(deps)?),
        QueryMsg::GetBalance { address } => to_json_binary(&query::balance(deps, address)?),
        QueryMsg::GetVestingInfo { address } => to_json_binary(&query::vesting_info(deps, address)?),
        QueryMsg::GetPoolReleaseInfo { address } => to_json_binary(&query::pool_release_info(deps, address)?),
        QueryMsg::GetMetadata {} => to_json_binary(&query::metadata(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        Ok(TokenInfoResponse {
            name: token_info.name,
            symbol: token_info.symbol,
            decimals: token_info.decimals,
            total_supply: token_info.total_supply,
            owner: token_info.owner.to_string(),
        })
    }

    pub fn balance(deps: Deps, address: String) -> StdResult<Uint128> {
        let addr = deps.api.addr_validate(&address)?;
        let balance = BALANCES.load(deps.storage, &addr).unwrap_or_default();
        Ok(balance)
    }

    pub fn vesting_info(deps: Deps, address: String) -> StdResult<VestingInfoResponse> {
        let addr = deps.api.addr_validate(&address)?;
        let vesting_info = VESTING.load(deps.storage, &addr)?;
        Ok(VestingInfoResponse {
            amount: vesting_info.amount,
            start_time: vesting_info.start_time.seconds(),
            release_schedule: vesting_info
                .release_schedule
                .iter()
                .map(|(t, a)| (t.seconds(), *a))
                .collect(),
        })
    }

    pub fn pool_release_info(deps: Deps, address: String) -> StdResult<PoolReleaseInfoResponse> {
        let addr = deps.api.addr_validate(&address)?;
        let pool_release_info = POOL_RELEASE_SCHEDULE.load(deps.storage, &addr)?;
        Ok(PoolReleaseInfoResponse {
            amount: pool_release_info.amount,
            release_schedule: pool_release_info
                .release_schedule
                .iter()
                .map(|(t, a)| (t.seconds(), *a))
                .collect(),
        })
    }

    pub fn metadata(deps: Deps) -> StdResult<MetadataResponse> {
        let metadata_url = METADATA_URL.load(deps.storage)?;
        Ok(MetadataResponse { metadata_url })
    }
}
fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, Addr, MessageInfo};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Verify token info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!("$SEINTS", token_info.name);
        assert_eq!("SEINTS", token_info.symbol);
        assert_eq!(6, token_info.decimals);
        assert_eq!(Uint128::new(1_000_000_000), token_info.total_supply);

        // Verify metadata URL
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMetadata {}).unwrap();
        let metadata: MetadataResponse = from_binary(&res).unwrap();
        assert_eq!(
            "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp",
            metadata.metadata_url
        );

        // Verify balances
        let team_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("team")).unwrap();
        assert_eq!(Uint128::new(200_000_000), team_balance); // 20% of 1 billion

        let pool_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("pool")).unwrap();
        assert_eq!(Uint128::new(400_000_000), pool_balance); // 40% of 1 billion
    }

    // Additional tests for `transfer`, `burn`, `release_vested`, `release_pool`, and `update_metadata`...
}

    #[test]
    fn transfer_works() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Transfer tokens
        let transfer_msg = ExecuteMsg::Transfer {
            recipient: "recipient".to_string(),
            amount: Uint128::new(100),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };

        let res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();
        assert_eq!(res.attributes.len(), 4);

        // Verify balances
        let creator_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("creator")).unwrap();
        assert_eq!(Uint128::new(299_999_900), creator_balance); // 300M - 100

        let recipient_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("recipient")).unwrap();
        assert_eq!(Uint128::new(100), recipient_balance);
    }

    #[test]
    fn burn_works() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Burn tokens
        let burn_msg = ExecuteMsg::Burn {
            amount: Uint128::new(100),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };

        let res = execute(deps.as_mut(), mock_env(), info, burn_msg).unwrap();
        assert_eq!(res.attributes.len(), 3);

        // Verify balances and total supply
        let creator_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("creator")).unwrap();
        assert_eq!(Uint128::new(299_999_900), creator_balance); // 300M - 100

        let token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        assert_eq!(Uint128::new(999_999_900), token_info.total_supply); // 1B - 100
    }

    #[test]
    fn release_vested_works() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Advance time to the first vesting release
        let mut env = mock_env();
        env.block.time = env.block.time.plus_years(1);

        // Release vested tokens
        let release_msg = ExecuteMsg::ReleaseVested {};
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };

        let res = execute(deps.as_mut(), env.clone(), info, release_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        // Verify balances
        let creator_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("creator")).unwrap();
        assert_eq!(Uint128::new(300_000_000), creator_balance); // 300M (initial) + 10% of 300M

        // Verify vesting schedule
        let vesting_info = VESTING.load(deps.as_ref().storage, &Addr::unchecked("creator")).unwrap();
        assert_eq!(vesting_info.release_schedule.len(), 2); // 2 releases remaining
    }

    #[test]
    fn release_pool_works() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Advance time to the first pool release
        let mut env = mock_env();
        env.block.time = env.block.time.plus_months(6);

        // Release pool tokens
        let release_msg = ExecuteMsg::ReleasePool {};
        let info = MessageInfo {
            sender: Addr::unchecked("pool"),
            funds: vec![],
        };

        let res = execute(deps.as_mut(), env.clone(), info, release_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        // Verify balances
        let pool_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("pool")).unwrap();
        assert_eq!(Uint128::new(450_000_000), pool_balance); // 400M (initial) + 50M (10% of 500M)

        // Verify pool release schedule
        let pool_release_info = POOL_RELEASE_SCHEDULE.load(deps.as_ref().storage, &Addr::unchecked("pool")).unwrap();
        assert_eq!(pool_release_info.release_schedule.len(), 2); // 2 releases remaining
    }

    #[test]
    fn update_metadata_works() {
        let mut deps = mock_dependencies();

        // Instantiate the contract
        let msg = InstantiateMsg {
            name: "$SEINTS".to_string(),
            symbol: "SEINTS".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000),
            metadata_url: "https://bafybeie6fkezbdf3pkioodnvuhjjhjrllcvxovhtam2z7d3qhnur4n4oy4.ipfs.w3s.link/logo.webp".to_string(),
            team_address: "team".to_string(),
            pool_address: "pool".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: coins(1000, "earth"),
        };

        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Update metadata
        let update_msg = ExecuteMsg::UpdateMetadata {
            metadata_url: "https://new-metadata-url.ipfs.w3s.link/logo.webp".to_string(),
        };

        // Use MessageInfo instead of mock_info
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };

        let res = execute(deps.as_mut(), mock_env(), info, update_msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        // Verify metadata URL
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMetadata {}).unwrap();
        let metadata: MetadataResponse = from_binary(&res).unwrap();
        assert_eq!("https://new-metadata-url.ipfs.w3s.link/logo.webp", metadata.metadata_url);
    }
