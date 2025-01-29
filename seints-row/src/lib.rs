pub mod contract;      // The core contract logic (instantiating, executing, querying).
mod error;             // Error handling (your custom errors like ContractError).
pub mod helpers;       // Helper functions or types (like your `CwTemplateContract`).
{% unless minimal %}pub mod integration_tests; // Conditional inclusion of integration tests based on a feature.
pub mod msg;           // The messages for interacting with the contract (InstantiateMsg, ExecuteMsg, etc.).
pub mod state;         // The contract's storage state, often including global variables like balances.

pub use crate::error::ContractError; // Re-exporting the `ContractError` for easy access in other modules.
