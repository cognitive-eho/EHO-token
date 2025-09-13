use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

// Helper struct for instantiation
#[cw_serde]
pub struct Rate {
    pub denom: String,
    pub rate: Uint128, // The value of 1 full token in USDC, with 6 decimals
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub eho_token_address: String,
    pub accepted_rates: Vec<Rate>,
    pub start_time: u64,
    pub end_time: u64,
    pub soft_cap: Uint128,
    pub hard_cap: Uint128,
    pub max_contribution_per_user: Uint128,
    pub eho_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Allows a whitelisted user to buy tokens by sending accepted native tokens with the message.
    Buy {},
    /// Allows a user to claim their EHO tokens after a successful sale.
    ClaimTokens {},
    /// Allows a user to request a refund if the sale failed.
    RequestRefund {},

    // --- Admin Functions ---
    EndSale {},
    AddToWhitelist {
        addresses: Vec<String>,
    },
    RemoveFromWhitelist {
        addresses: Vec<String>,
    },
    ReclaimUnsoldTokens {},
    WithdrawFunds {},
    UpdateAdmin {
        new_admin: String,
    },
    UpdatePause {
        pause: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the contract's immutable configuration.
    #[returns(crate::state::Config)]
    Config {},
    /// Returns the contract's current dynamic state.
    #[returns(crate::state::State)]
    State {},
    /// Returns the exchange rates for all accepted tokens.
    #[returns(Vec<Rate>)]
    AcceptedRates {},
    /// Checks if a given address is whitelisted.
    #[returns(bool)]
    IsWhitelisted { address: String },
    /// Returns the total USDC-equivalent value a user has contributed.
    #[returns(Uint128)]
    TotalContributionOf { address: String },
    /// Returns the specific coins a user has contributed.
    #[returns(Vec<cosmwasm_std::Coin>)]
    ContributionsOf { address: String },
    /// Calculates and returns the amount of EHO a user is entitled to claim
    /// based on their current contribution. Returns 0 if they haven't contributed.
    #[returns(cosmwasm_std::Uint128)]
    EhoAllocationOf { address: String },
}
