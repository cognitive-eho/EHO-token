#![cfg(test)]

use cosmwasm_std::{coin, Addr, Coin, Empty, Uint128};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use cw_multi_test::AppBuilder;

use presale_eho::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Rate};
use presale_eho::state::{Config, SaleStatus, State};
use presale_eho::ContractError;

// --- IBC Denoms for Realistic Testing ---
const NOBLE_USDC: &str = "ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81";
const AXELAR_USDC: &str = "ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349";
const ATOM: &str = "ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9";
const OSMO: &str = "ibc/376222D6D9DAE23092E29740E56B758580935A6D77C24C2ABD57A6A78A1F3955";

// --- Helper Contracts ---
fn eho_cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_eho::contract::execute,
        cw20_eho::contract::instantiate,
        cw20_eho::contract::query,
    );
    Box::new(contract)
}

fn presale_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        presale_eho::contract::execute,
        presale_eho::contract::instantiate,
        presale_eho::contract::query,
    );
    Box::new(contract)
}

// --- Test Setup Helper ---
struct TestSetup {
    app: App,
    presale_addr: Addr,
    eho_addr: Addr,
    admin: Addr,
    alice: Addr,
    bob: Addr,
}
fn setup() -> TestSetup {
    // Define user addresses with the "cosmwasm" prefix
    let admin = Addr::unchecked("cosmwasm1qypqxpq9qcrsszgszyfpx9q4zct3sxfqx5vwjh");
    let alice = Addr::unchecked("cosmwasm1qypqxpq9qcrsszgszyfpx9q4zct3sy3q8mmchv");
    let bob = Addr::unchecked("cosmwasm1qypqxpq9qcrsszgszyfpx9q4zct3sy3p6d0d27");

    // --- FIX IS HERE: Use AppBuilder for correct prefix handling ---
    let mut app = AppBuilder::new().build(|_router, _, _storage| {
        // We can initialize balances here if needed, but for now we'll use sudo later
    });
    // --- END OF FIX ---

    // Store contract code
    let eho_code_id = app.store_code(eho_cw20_contract());
    let presale_code_id = app.store_code(presale_contract());

    // Instantiate EHO token
    let eho_instantiate_msg = cw20_eho::msg::InstantiateMsg {
        name: "Cognitive Echo".to_string(),
        symbol: "EHO".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: admin.to_string(),
            cap: None,
        }),
        marketing: None,
    };
    let eho_addr = app
        .instantiate_contract(
            eho_code_id,
            admin.clone(),
            &eho_instantiate_msg,
            &[],
            "EHO",
            None,
        )
        .unwrap();

    // Instantiate Presale contract with realistic parameters
    let presale_instantiate_msg = InstantiateMsg {
        admin: admin.to_string(),
        eho_token_address: eho_addr.to_string(),
        eho_price: Uint128::new(10_000), // $0.01 per EHO
        accepted_rates: vec![
            Rate {
                denom: NOBLE_USDC.to_string(),
                rate: Uint128::new(1_000_000),
            },
            Rate {
                denom: AXELAR_USDC.to_string(),
                rate: Uint128::new(1_000_000),
            },
            Rate {
                denom: ATOM.to_string(),
                rate: Uint128::new(7_000_000),
            },
            Rate {
                denom: OSMO.to_string(),
                rate: Uint128::new(550_000),
            },
        ],
        start_time: app.block_info().time.seconds() + 100,
        end_time: app.block_info().time.seconds() + 200,
        soft_cap: Uint128::new(100_000_000_000), // 100k USDC
        hard_cap: Uint128::new(500_000_000_000), // 500k USDC
        max_contribution_per_user: Uint128::new(200_000_000_000), // Increased for test
    };
    let presale_addr = app
        .instantiate_contract(
            presale_code_id,
            admin.clone(),
            &presale_instantiate_msg,
            &[],
            "Presale",
            None,
        )
        .unwrap();

    // Fund the presale contract with EHO for distribution
    app.execute_contract(
        admin.clone(),
        eho_addr.clone(),
        &Cw20ExecuteMsg::Mint {
            recipient: presale_addr.to_string(),
            amount: Uint128::new(400_000_000_000_000), // 400M EHO
        },
        &[],
    )
    .unwrap();

    // Mint native funds for users
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: alice.to_string(),
            amount: vec![coin(300_000_000_000, NOBLE_USDC), coin(15_000_000_000, ATOM)],
        },
    ))
    .unwrap();
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: bob.to_string(),
            amount: vec![coin(2_000_000_000, OSMO)],
        },
    ))
    .unwrap();

    TestSetup {
        app,
        presale_addr,
        eho_addr,
        admin,
        alice,
        bob,
    }
}

#[test]
fn test_instantiation_and_config() {
    let setup = setup();
    let app = setup.app;

    // Query the config and verify parameters
    let config: Config = app
        .wrap()
        .query_wasm_smart(setup.presale_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.admin, setup.admin);
    assert_eq!(config.eho_token_address, setup.eho_addr);
    assert_eq!(config.soft_cap, Uint128::new(100_000_000_000));
    assert_eq!(config.hard_cap, Uint128::new(500_000_000_000));
    assert_eq!(
        config.max_contribution_per_user,
        Uint128::new(200_000_000_000)
    );
    assert_eq!(config.accepted_payment_denoms.len(), 4);

    // Query the state and verify initial state
    let state: State = app
        .wrap()
        .query_wasm_smart(setup.presale_addr, &QueryMsg::State {})
        .unwrap();
    assert_eq!(state.sale_status, SaleStatus::Pending);
    assert_eq!(state.total_usdc_raised, Uint128::zero());
    assert!(!state.paused);
}

#[test]
fn test_buy_logic_and_failures() {
    let mut setup = setup();

    // Whitelist alice
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::AddToWhitelist {
                addresses: vec![setup.alice.to_string()],
            },
            &[],
        )
        .unwrap();

    // FAIL: Try to buy before sale starts
    let err = setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(1_000_000_000, NOBLE_USDC)],
        )
        .unwrap_err();
    assert_eq!(ContractError::SaleNotActive {}, err.downcast().unwrap());

    // Advance time to start the sale
    setup.app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // SUCCESS: Alice buys with USDC
    setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(5_000_000_000, NOBLE_USDC)], // 5k USDC
        )
        .unwrap();

    // Verify state
    let state: State = setup
        .app
        .wrap()
        .query_wasm_smart(setup.presale_addr.clone(), &QueryMsg::State {})
        .unwrap();
    assert_eq!(state.total_usdc_raised, Uint128::new(5_000_000_000));

    // FAIL: Bob (not whitelisted) tries to buy
    let err = setup
        .app
        .execute_contract(
            setup.bob.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(1_000_000_000, OSMO)],
        )
        .unwrap_err();
    assert_eq!(ContractError::NotInWhitelist {}, err.downcast().unwrap());

    // Mint unaccepted denom to Alice to avoid bank overflow
    setup.app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: setup.alice.to_string(),
            amount: vec![coin(1_000, "untrn")],
        },
    ))
    .unwrap();

    // FAIL: Alice tries to buy with an unaccepted coin
    let err = setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(1_000, "untrn")],
        )
        .unwrap_err();
    assert_eq!(ContractError::UnacceptedPaymentDenom { denom: "untrn".to_string() }, err.downcast().unwrap());

    // FAIL: Alice tries to exceed her individual cap
    let err = setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(196_000_000_000, NOBLE_USDC)], // To exceed 200k
        )
        .unwrap_err();
    assert_eq!(ContractError::UserCapExceeded {}, err.downcast().unwrap());
}

#[test]
fn test_successful_sale_flow() {
    let mut setup = setup();

    // Whitelist users
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::AddToWhitelist {
                addresses: vec![setup.alice.to_string(), setup.bob.to_string()],
            },
            &[],
        )
        .unwrap();

    // Advance time to start the sale
    setup.app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // Alice contributes ~100k USDC worth of ATOM (approx 14,285.71 ATOM)
    setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(14_285_714_286, ATOM)],
        )
        .unwrap();

    // Bob contributes 1.1k USDC worth of OSMO (2k OSMO)
    setup
        .app
        .execute_contract(
            setup.bob.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(2_000_000_000, OSMO)],
        )
        .unwrap();

    // Verify total raised is over soft cap
    let state: State = setup
        .app
        .wrap()
        .query_wasm_smart(setup.presale_addr.clone(), &QueryMsg::State {})
        .unwrap();
    assert_eq!(state.total_usdc_raised, Uint128::new(101_100_000_002));

    // Advance time to end the sale
    setup.app.update_block(|block| {
        block.time = block.time.plus_seconds(100);
    });

    // Bob claims his tokens
    setup
        .app
        .execute_contract(
            setup.bob.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::ClaimTokens {},
            &[],
        )
        .unwrap();

    // Verify Bob received EHO (1.1k USDC / $0.01 = 110k EHO)
    let bob_eho_balance: cw20::BalanceResponse = setup
        .app
        .wrap()
        .query_wasm_smart(
            setup.eho_addr.clone(),
            &cw20_eho::msg::QueryMsg::Balance {
                address: setup.bob.to_string(),
            },
        )
        .unwrap();
    assert_eq!(bob_eho_balance.balance, Uint128::new(110_000_000_000));

    // FAIL: Bob tries to claim again
    let err = setup
        .app
        .execute_contract(
            setup.bob.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::ClaimTokens {},
            &[],
        )
        .unwrap_err();
    assert_eq!(ContractError::NothingToClaim {}, err.downcast().unwrap());

    // Admin withdraws funds
    let admin_atom_before: Coin = setup
        .app
        .wrap()
        .query_balance(setup.admin.clone(), ATOM)
        .unwrap();
    let admin_osmo_before: Coin = setup
        .app
        .wrap()
        .query_balance(setup.admin.clone(), OSMO)
        .unwrap();

    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::WithdrawFunds {},
            &[],
        )
        .unwrap();

    // Verify funds arrived in admin wallet
    let admin_atom_after: Coin = setup
        .app
        .wrap()
        .query_balance(setup.admin.clone(), ATOM)
        .unwrap();
    let admin_osmo_after: Coin = setup
        .app
        .wrap()
        .query_balance(setup.admin.clone(), OSMO)
        .unwrap();
    assert_eq!(
        admin_atom_after.amount,
        admin_atom_before.amount + Uint128::new(14_285_714_286)
    );
    assert_eq!(
        admin_osmo_after.amount,
        admin_osmo_before.amount + Uint128::new(2_000_000_000)
    );
}

#[test]
fn test_failed_sale_flow() {
    let mut setup = setup();

    // Whitelist alice
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::AddToWhitelist {
                addresses: vec![setup.alice.to_string()],
            },
            &[],
        )
        .unwrap();

    // Advance time to start the sale
    setup.app.update_block(|block| {
        block.time = block.time.plus_seconds(101);
    });

    // Alice contributes 5k USDC (less than soft cap)
    setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(5_000_000_000, NOBLE_USDC)],
        )
        .unwrap();

    // Advance time to end the sale
    setup.app.update_block(|block| {
        block.time = block.time.plus_seconds(100);
    });

    // FAIL: Alice tries to claim tokens
    let err = setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::ClaimTokens {},
            &[],
        )
        .unwrap_err();
    assert_eq!(ContractError::SoftCapNotReached {}, err.downcast().unwrap());

    // SUCCESS: Alice requests a refund
    let alice_usdc_before = setup
        .app
        .wrap()
        .query_balance(setup.alice.clone(), NOBLE_USDC)
        .unwrap()
        .amount;
    setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::RequestRefund {},
            &[],
        )
        .unwrap();
    let alice_usdc_after = setup
        .app
        .wrap()
        .query_balance(setup.alice.clone(), NOBLE_USDC)
        .unwrap()
        .amount;

    // Verify her USDC was returned
    assert_eq!(
        alice_usdc_after,
        alice_usdc_before + Uint128::new(5_000_000_000)
    );

    // FAIL: Admin tries to withdraw funds
    let err = setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::WithdrawFunds {},
            &[],
        )
        .unwrap_err();
    assert_eq!(ContractError::SaleNotSucceeded {}, err.downcast().unwrap());
}

#[test]
fn test_admin_controls() {
    let mut setup = setup();
    let charlie = Addr::unchecked("cosmwasm1qypqxpq9qcrsszgszyfpx9q4zct3s9pp76qy8a");

    // PAUSE
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::UpdatePause { pause: true },
            &[],
        )
        .unwrap();

    // FAIL: Alice tries to buy while paused
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::AddToWhitelist {
                addresses: vec![setup.alice.to_string()],
            },
            &[],
        )
        .unwrap();
    setup
        .app
        .update_block(|b| b.time = b.time.plus_seconds(101));
    let err = setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(1_000, NOBLE_USDC)],
        )
        .unwrap_err();
    assert_eq!(ContractError::Paused {}, err.downcast().unwrap());

    // UPDATE ADMIN
    setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::UpdateAdmin {
                new_admin: charlie.to_string(),
            },
            &[],
        )
        .unwrap();

    // FAIL: Old admin tries to use admin powers
    let err = setup
        .app
        .execute_contract(
            setup.admin.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::UpdatePause { pause: false },
            &[],
        )
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());

    // SUCCESS: New admin (Charlie) unpauses the contract
    setup
        .app
        .execute_contract(
            charlie.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::UpdatePause { pause: false },
            &[],
        )
        .unwrap();

    // SUCCESS: Alice can now buy
    setup
        .app
        .execute_contract(
            setup.alice.clone(),
            setup.presale_addr.clone(),
            &ExecuteMsg::Buy {},
            &[coin(1_000, NOBLE_USDC)],
        )
        .unwrap();
}