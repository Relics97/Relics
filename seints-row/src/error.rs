use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Occurs when the token decimals are invalid (e.g., greater than 18).
    #[error("Invalid decimals: {decimals} (must be <= 18)")]
    InvalidDecimals { decimals: u8 },

    /// Occurs when a duplicate address is provided (e.g., team and pool addresses are the same).
    #[error("Duplicate addresses: {address}")]
    DuplicateAddresses { address: String },

    /// Occurs when an action is attempted by an unauthorized address.
    #[error("Unauthorized")]
    Unauthorized {},

    /// Occurs when a user tries to transfer more tokens than they have.
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: Uint128, available: Uint128 },

    /// Occurs when an invalid address is provided.
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    /// Occurs when an invalid token amount is specified.
    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: Uint128 },

    /// Occurs when the token decimals are invalid (e.g., greater than 18).
    #[error("Invalid decimals: {decimals} (must be <= 18)")]
    InvalidDecimals { decimals: u8 },

    /// Occurs when the initial supply is invalid (e.g., not exactly 1 billion).
    #[error("Invalid initial supply: {actual} (expected {expected})")]
    InvalidInitialSupply { expected: Uint128, actual: Uint128 },

    /// Occurs when an arithmetic operation overflows or underflows.
    #[error("Arithmetic overflow/underflow")]
    Overflow {},

    /// Occurs when the metadata URL is invalid.
    #[error("Invalid metadata URL: {url} (must be a valid URL)")]
    InvalidMetadata { url: String },

    /// Occurs when duplicate addresses are provided (e.g., team and pool addresses are the same).
    #[error("Duplicate addresses: {address}")]
    DuplicateAddresses { address: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unauthorized_error() {
        let err = ContractError::Unauthorized {};
        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn test_insufficient_balance_error() {
        let err = ContractError::InsufficientBalance {
            required: Uint128::new(100),
            available: Uint128::new(50),
        };
        assert_eq!(
            err.to_string(),
            "Insufficient balance: required 100, available 50"
        );
    }

    #[test]
    fn test_invalid_address_error() {
        let err = ContractError::InvalidAddress {
            address: "invalid_address".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid address: invalid_address");
    }

    #[test]
    fn test_invalid_amount_error() {
        let err = ContractError::InvalidAmount {
            amount: Uint128::new(0),
        };
        assert_eq!(err.to_string(), "Invalid amount: 0");
    }

    #[test]
    fn test_invalid_decimals_error() {
        let err = ContractError::InvalidDecimals { decimals: 19 };
        assert_eq!(err.to_string(), "Invalid decimals: 19 (must be <= 18)");
    }

    #[test]
    fn test_invalid_initial_supply_error() {
        let err = ContractError::InvalidInitialSupply {
            expected: Uint128::new(1_000_000_000),
            actual: Uint128::new(500_000_000),
        };
        assert_eq!(
            err.to_string(),
            "Invalid initial supply: 500000000 (expected 1000000000)"
        );
    }

    #[test]
    fn test_overflow_error() {
        let err = ContractError::Overflow {};
        assert_eq!(err.to_string(), "Arithmetic overflow/underflow");
    }

    #[test]
    fn test_invalid_metadata_error() {
        let err = ContractError::InvalidMetadata {
            url: "invalid_url".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid metadata URL: invalid_url (must be a valid URL)");
    }

    #[test]
    fn test_duplicate_addresses_error() {
        let err = ContractError::DuplicateAddresses {
            address: "team_address".to_string(),
        };
        assert_eq!(err.to_string(), "Duplicate addresses: team_address");
    }
}