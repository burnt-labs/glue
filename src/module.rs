//! Traits for reusable, composable CosmWasm modules.

use crate::response::Response;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdError, StdResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;

/// A well typed CosmWasm module
///
/// A module must implement instantiate, execute, and query handlers.
/// These handlers may, however, be no-ops.
///
/// Programmers looking to implement reusable CosmWasm modules should create
/// structs that implement Module.
pub trait Module {
    /// The message sent to the module to instantiate its state.
    type InstantiateMsg: for<'a> Deserialize<'a>;
    /// The type of transaction messages this module can handle. For modules
    /// that support multiple types of transaction, this will often times be
    /// a sum type.
    type ExecuteMsg: for<'a> Deserialize<'a>;
    /// The type of query messages this module can handle. For modules that
    /// support multiple queries, this will often times be a sum type.
    type QueryMsg: for<'a> Deserialize<'a>;
    /// The response to queries dispatched to the module.
    type QueryResp: Serialize;
    /// The type of errors this module can generate. This must implement
    /// Display for easy stringification.
    type Error: Display;

    /// The instantiate handler for the module. When a Manager with this
    /// module registered is instantiated, this method may be called.
    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Self::Error>;
    /// The transaction handler for this module. Messages to this contract
    /// will be dispatched by the Manager.
    fn execute(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error>;
    /// The query handler for this module. Messages to this contract will be
    /// dispatched by the Manager.
    fn query(
        &self,
        deps: &Deps,
        env: Env,
        msg: Self::QueryMsg,
    ) -> Result<Self::QueryResp, Self::Error>;
}

/// A dynamically typed module.
///
/// GenericModules accept JSON values as their messages and return them as
/// their results. Errors returned by GenericModules are strings. This trait
/// was created to enable a simple dynamic dispatch of messages sent to the
/// contract by the `Manager`.
pub trait GenericModule {
    /// A generic implementation of Module::instantiate
    fn instantiate_value(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: &Value,
    ) -> Result<Response, String>;
    /// A generic implementation of Module::execute
    fn execute_value(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: &Value,
    ) -> Result<Response, String>;
    /// A generic implementation of Module::query
    fn query_value(&self, deps: &Deps, env: Env, msg: &Value) -> StdResult<Binary>;
}

/// An implementation of GenericModule for all valid implementations of Module.
impl<T, A, B, C, D, E> GenericModule for T
where
    A: for<'de> Deserialize<'de>,
    B: for<'de> Deserialize<'de>,
    C: for<'de> Deserialize<'de>,
    D: Serialize,
    E: Display,
    T: Module<InstantiateMsg = A, ExecuteMsg = B, QueryMsg = C, QueryResp = D, Error = E>,
{
    fn instantiate_value(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: &Value,
    ) -> Result<Response, String> {
        let parsed_msg = serde_json::from_value(msg.clone()).map_err(|e| e.to_string())?;
        self.instantiate(deps, env, info, parsed_msg)
            .map_err(|e| e.to_string())
    }

    fn execute_value(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: &Value,
    ) -> Result<Response, String> {
        let parsed_msg = serde_json::from_value(msg.clone()).map_err(|e| e.to_string())?;
        self.execute(deps, env, info, parsed_msg)
            .map_err(|e| e.to_string())
    }

    fn query_value(&self, deps: &Deps, env: Env, msg: &Value) -> StdResult<Binary> {
        let parsed_msg = serde_json::from_value(msg.clone())
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        let res = self
            .query(deps, env, parsed_msg)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        cosmwasm_std::to_binary(&res)
    }
}
