use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

// --- CONFIGURATION (Set once at instantiation) ---
#[cw_serde]
pub struct Config {
    /// The admin of the contract, who can manage the whitelist, pause, and withdraw funds.
    pub admin: Addr,
    /// The address of the EHO token contract to be distributed.
    pub eho_token_address: Addr,
    /// The list of accepted native token denoms for payment (e.g., Noble USDC, Axelar USDC, ATOM, OSMO).
    pub accepted_payment_denoms: Vec<String>,
    /// The timestamp (in seconds) when the sale starts.
    pub start_time: u64,
    /// The timestamp (in seconds) when the sale ends.
    pub end_time: u64,
    /// The minimum amount of USDC-equivalent value to be raised for the sale to be a success.
    pub soft_cap: Uint128,
    /// The maximum amount of USDC-equivalent value the contract will accept.
    pub hard_cap: Uint128,
    /// The maximum USDC-equivalent value any single user is allowed to contribute.
    pub max_contribution_per_user: Uint128,
    /// The price of 1 EHO in USDC-equivalent value (with 6 decimals). E.g., $0.01 = 10000
    pub eho_price: Uint128,
}
pub const CONFIG: Item<Config> = Item::new("config");

/// A map from an accepted payment denom to its value in USDC (with 6 decimals).
/// e.g., "ibc/..." -> "1000000" for USDC, "ibc/..." -> "7000000" for ATOM at $7.00
pub const EXCHANGE_RATES: Map<&str, Uint128> = Map::new("exchange_rates");

// --- STATE (Changes during the sale) ---
#[cw_serde]
pub struct State {
    /// The total USDC-equivalent value raised so far.
    pub total_usdc_raised: Uint128,
    /// The current status of the sale.
    pub sale_status: SaleStatus,
    /// A flag to halt buy functionality in case of emergencies.
    pub paused: bool,
}
pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub enum SaleStatus {
    Pending,
    Active,
    Succeeded,
    Failed, // Refunds enabled
}

// --- USER DATA ---
/// A map from a user's address to a vector of the actual coins they have contributed.
/// This is crucial for accurate refunds of multiple asset types.
pub const CONTRIBUTIONS: Map<&Addr, Vec<Coin>> = Map::new("contributions");

/// A map of whitelisted addresses. The bool value must be `true`.
pub const WHITELIST: Map<&Addr, bool> = Map::new("whitelist");
