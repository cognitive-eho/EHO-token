use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: Caller is not the contract admin")]
    Unauthorized {},

    #[error("Sale is not active")]
    SaleNotActive {},

    #[error("Sale has not started yet")]
    SaleNotStarted {},

    #[error("Sale has already ended")]
    SaleHasEnded {},

    #[error("Sale is still active, cannot claim or refund yet")]
    SaleIsStillActive {},
    
    #[error("Sale is not in a state that can be ended (must be Active)")]
    SaleCannotBeEnded {},

    #[error("Hard cap has been reached")]
    HardCapReached {},

    #[error("Soft cap was not reached, sale failed. Cannot claim tokens.")]
    SoftCapNotReached {},

    #[error("Sale did not fail, refunds are not available.")]
    SaleDidNotFail {},

    #[error("Sale was not successful, cannot withdraw funds.")]
    SaleNotSucceeded {},

    #[error("Address is not on the whitelist")]
    NotInWhitelist {},

    #[error("Caller has nothing to claim")]
    NothingToClaim {},

    #[error("Caller has no funds to refund")]
    NothingToRefund {},

    #[error("Invalid amount: Cannot process a transaction with zero tokens")]
    InvalidZeroAmount {},

    #[error("Contribution exceeds maximum limit per user")]
    UserCapExceeded {},

    #[error("Invalid payment: Must be a single, accepted coin type")]
    InvalidPayment {},

    #[error("Payment denom '{denom}' is not an accepted payment type")]
    UnacceptedPaymentDenom { denom: String },
    
    #[error("Configuration error: {details}")]
    ConfigError { details: String },
    
    #[error("Contract is currently paused")]
    Paused {},

    #[error("No funds available for withdrawal")]
    NoFundsToWithdraw {},

    #[error("No tokens available to reclaim")]
    NoTokensToReclaim {},
}