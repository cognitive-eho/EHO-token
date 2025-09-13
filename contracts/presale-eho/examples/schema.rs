use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::Uint128;

// Import your contract's message and state types
use presale_eho::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use presale_eho::state::{Config, State};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // Generate and export the schema for all primary message types
    export_schema_with_title(&schema_for!(InstantiateMsg), &out_dir, "InstantiateMsg");
    export_schema_with_title(&schema_for!(ExecuteMsg), &out_dir, "ExecuteMsg");
    export_schema_with_title(&schema_for!(QueryMsg), &out_dir, "QueryMsg");

    // Export schemas for any custom response types defined in QueryMsg
    // Note: We use schema_for!(<ResponseType>) here
    export_schema_with_title(&schema_for!(Config), &out_dir, "ConfigResponse");
    export_schema_with_title(&schema_for!(State), &out_dir, "StateResponse");
    export_schema_with_title(&schema_for!(bool), &out_dir, "IsWhitelistedResponse");
    export_schema_with_title(&schema_for!(Uint128), &out_dir, "ContributionOfResponse");
}