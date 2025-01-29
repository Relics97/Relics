#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse};
use crate::state::{TokenInfo, TOKEN_INFO, BALANCES};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:seints-row";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Validate the token info
    if msg.decimals > 18 {
        return Err(ContractError::InvalidDecimals {});
    }

    if msg.initial_supply.is_zero() {
        return Err(ContractError::InvalidInitialSupply {});
    }

    let specified_address = deps.api.addr_validate(&msg.specified_address)?;

    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: msg.initial_supply,
        owner: info.sender.clone(),
    };

    // Save token info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // Calculate 20% of the initial supply
    let twenty_percent = msg
        .initial_supply
        .multiply_ratio(20u128, 100u128);

    // Calculate 80% of the initial supply
    let creator_amount = msg
        .initial_supply
        .checked_sub(twenty_percent)
        .ok_or(ContractError::Overflow {})?;

    // Mint 80% to the creator
    BALANCES.save(deps.storage, &info.sender, &creator_amount)?;

    // Mint 20% to the specified address
    BALANCES.save(deps.storage, &specified_address, &twenty_percent)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("total_supply", msg.initial_supply))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => execute::transfer(deps, info, recipient, amount),
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

        BALANCES.update(deps.storage, &info.sender, |balance| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            if balance < amount {
                return Err(ContractError::InsufficientBalance {});
            }
            Ok(balance - amount)
        })?;

        BALANCES.update(deps.storage, &recipient_addr, |balance| -> Result<_, ContractError> {
            Ok(balance.unwrap_or_default() + amount)
        })?;

        Ok(Response::new()
            .add_attribute("method", "transfer")
            .add_attribute("from", info.sender)
            .add_attribute("to", recipient)
            .add_attribute("amount", amount))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTokenInfo {} => to_json_binary(&query::token_info(deps)?),
        QueryMsg::GetBalance { address } => to_json_binary(&query::balance(deps, address)?),
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
            owner: token_info.owner,
        })
    }

    pub fn balance(deps: Deps, address: String) -> StdResult<Uint128> {
        let addr = deps.api.addr_validate(&address)?;
        let balance = BALANCES.load(deps.storage, &addr).unwrap_or_default();
        Ok(balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "SEINT".to_string(),
            symbol: "SEINT".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000_000_000),
            specified_address: "0x45d2B456361f5D2D3e473018E56482059075eceB".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenInfo {}).unwrap();
        let token_info: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!("SEINT", token_info.name);
        assert_eq!("SEINT", token_info.symbol);
        assert_eq!(6, token_info.decimals);
        assert_eq!(Uint128::new(1_000_000_000_000_000), token_info.total_supply);
    }

    #[test]
    fn transfer_works() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "SEINT".to_string(),
            symbol: "SEINT".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000_000_000),
            specified_address: "0x45d2B456361f5D2D3e473018E56482059075eceB".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let transfer_msg = ExecuteMsg::Transfer {
            recipient: "recipient".to_string(),
            amount: Uint128::new(100),
        };
        let res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();
        assert_eq!(res.attributes.len(), 4);
    }
}
