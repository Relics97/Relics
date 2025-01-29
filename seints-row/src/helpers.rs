use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg,
    WasmQuery,
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
    ///
    /// # Returns
    /// A `StdResult<CosmosMsg>` containing the message to execute.
    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
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
    pub fn count<Q, CQ>(&self, querier: &Q) -> StdResult<GetCountResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetCount {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: GetCountResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }
}