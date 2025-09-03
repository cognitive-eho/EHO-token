use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    // The admin of the contract, who can start the sale, add to whitelist, etc.
    pub admin: Addr,
    // The address of the EHO token contract.
    pub eho_token_address: Addr,
    // The address of the token being accepted for payment (USDC).
    pub usdc_token_address: Addr,
    // The price of 1 EHO in terms of the smallest unit of USDC (e.g., if 1 USDC = 10 EHO, this could be 100_000).
    pub exchange_rate: Uint128,
    // The timestamp (in seconds) when the sale starts.
    pub start_time: u64,
    // The timestamp (in seconds) when the sale ends.
    pub end_time: u64,
    // The minimum amount of USDC to be raised for the sale to be a success.
    pub soft_cap: Uint128,
    // The maximum amount of USDC the contract will accept.
    pub hard_cap: Uint128,
}
pub const CONFIG: Item<Config> = Item::new("config");

// --- STATE (Changes during the sale) ---
#[cw_serde]
pub struct State {
    // The total amount of USDC raised so far.
    pub total_usdc_raised: Uint128,
    // The current status of the sale.
    pub sale_status: SaleStatus,
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
// A map from a user's address to the amount of USDC they have contributed.
pub const CONTRIBUTIONS: Map<&Addr, Uint128> = Map::new("contributions");

// A map of whitelisted addresses.
pub const WHITELIST: Map<&Addr, bool> = Map::new("whitelist");