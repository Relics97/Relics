/// Core contract logic, including instantiation, execution, and query handlers.
pub mod contract;

/// Error handling for the contract, defining custom errors like `ContractError`.
mod error;

/// Helper functions and types to simplify interactions with the contract.
pub mod helpers;

/// Integration tests to validate the contract's functionality in a simulated blockchain environment.
#[cfg(not(feature = "minimal"))]
pub mod integration_tests;

/// Messages used to interact with the contract, such as `InstantiateMsg`, `ExecuteMsg`, and `QueryMsg`.
pub mod msg;

/// The contract's storage state, including global variables like balances and metadata.
pub mod state;

/// Re-export `ContractError` for easy access in other modules.
pub use crate::error::ContractError;