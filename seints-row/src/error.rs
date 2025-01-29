#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Occurs when an action is attempted by an unauthorized address.
    #[error("Unauthorized")]
    Unauthorized {},

     /// Occurs when a user tries to transfer more tokens than they have.
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: Uint128, available: Uint128 },

     /// Occurs when an invalid address is provided.
    #[error("Invalid address")]
    InvalidAddress {},

     /// Occurs when an invalid token amount is specified.
    #[error("Invalid amount")]
    InvalidAmount {},
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
}