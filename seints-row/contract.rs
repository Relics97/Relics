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
    if msg.name.is_empty() || msg.symbol.is_empty() {
    return Err(ContractError::InvalidTokenInfo {});
    }

    // Save token info
    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: msg.initial_supply,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // Mint the initial supply to the owner
    BALANCES.save(deps.storage, &info.sender, &msg.initial_supply)?;

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
        ExecuteMsg::Burn { amount } => execute::burn(deps, info, amount),
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

    pub fn burn(
        deps: DepsMut,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // Deduct the tokens from the sender's balance
        BALANCES.update(deps.storage, &info.sender, |balance| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            if balance < amount {
                return Err(ContractError::InsufficientBalance {});
            }
            Ok(balance - amount)
        })?;

        // Reduce the total supply
        TOKEN_INFO.update(deps.storage, |mut token_info| -> Result<_, ContractError> {
            token_info.total_supply = token_info.total_supply.checked_sub(amount).ok_or(ContractError::Overflow {})?;
            Ok(token_info)
        })?;

        Ok(Response::new()
            .add_attribute("method", "burn")
            .add_attribute("from", info.sender)
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
            owner: token_info.owner.to_string(),
        })
    }

    pub fn balance(deps: Deps, address: String) -> StdResult<Uint128> {
        let addr = deps.api.addr_validate(&address)?;
        let balance = BALANCES.load(deps.storage, &addr).unwrap_or_default();
        Ok(balance)
    }
}
