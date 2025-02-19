use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdError,
    StdResult, WasmMsg, WasmQuery,
};
use crate::msg::{ExecuteMsg, GetCountResponse, QueryMsg};

/// A wrapper around a contract address that provides helper functions
/// for interacting with the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    /// Returns the contract address.
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    /// Creates a `CosmosMsg` to execute a message on this contract.
    ///
    /// # Arguments
    /// * `msg` - The message to execute, which can be converted into `ExecuteMsg`.
    /// * `funds` - Optional coins to send along with the message (default is empty).
    ///
    /// # Returns
    /// A `StdResult<CosmosMsg>` containing the message to execute.
    ///
    /// # Errors
    /// Returns an error if serialization of the message fails.
    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into()).map_err(|e| {
            StdError::generic_err(format!("Failed to serialize message: {}", e))
        })?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }

    /// Queries the contract to get the current count.
    ///
    /// # Arguments
    /// * `querier` - A reference to a querier implementing the `Querier` trait.
    ///
    /// # Returns
    /// A `StdResult<GetCountResponse>` containing the current count.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Serialization of the query message fails.
    /// - The query execution fails.
    pub fn count<Q, CQ>(&self, querier: &Q) -> StdResult<GetCountResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetCount {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg).map_err(|e| {
                StdError::generic_err(format!("Failed to serialize query message: {}", e))
            })?,
        }
        .into();
        let res: GetCountResponse = QuerierWrapper::<CQ>::new(querier)
            .query(&query)
            .map_err(|e| StdError::generic_err(format!("Query failed: {}", e)))?;
        Ok(res)
    }
}