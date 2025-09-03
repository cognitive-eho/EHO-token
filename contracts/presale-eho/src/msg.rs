use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub eho_token_address: String,
    pub usdc_token_address: String,
    pub exchange_rate: Uint128,
    pub start_time: u64,
    pub end_time: u64,
    pub soft_cap: Uint128,
    pub hard_cap: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Starts the sale. Can only be called by the admin.
    StartSale {},
    // Allows a whitelisted user to buy tokens. This would be called via a CW20 Send message from the USDC contract.
    Receive(cw20::Cw20ReceiveMsg),
    // Allows a user to claim their EHO tokens after a successful sale.
    ClaimTokens {},
    // Allows a user to request a refund if the sale failed.
    RequestRefund {},
    
    // --- Admin Functions ---
    // Adds a list of addresses to the whitelist.
    AddToWhitelist { addresses: Vec<String> },
    // Withdraws the raised USDC to the treasury after a successful sale.
    WithdrawFunds {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Returns the contract's configuration.
    #[returns(crate::state::Config)]
    Config {},
    // Returns the contract's current state (amount raised, status).
    #[returns(crate::state::State)]
    State {},
    // Checks if a given address is whitelisted.
    #[returns(bool)]
    IsWhitelisted { address: String },
    // Returns the amount of USDC a specific user has contributed.
    #[returns(Uint128)]
    ContributionOf { address: String },
}