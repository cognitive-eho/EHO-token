#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128, WasmMsg,
};
use cw2::{ensure_from_older_version, set_contract_version};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Rate};
use crate::state::{
    Config, SaleStatus, State, CONFIG, CONTRIBUTIONS, EXCHANGE_RATES, STATE, WHITELIST,
};

const CONTRACT_NAME: &str = "crates.io:eho-presale-multi-asset";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate parameters
    if msg.start_time >= msg.end_time {
        return Err(ContractError::ConfigError {
            details: "Start time must be before end time".to_string(),
        });
    }
    if msg.soft_cap > msg.hard_cap {
        return Err(ContractError::ConfigError {
            details: "Soft cap cannot be greater than hard cap".to_string(),
        });
    }
    if msg.hard_cap.is_zero() || msg.max_contribution_per_user.is_zero() || msg.eho_price.is_zero()
    {
        return Err(ContractError::InvalidZeroAmount {});
    }
    if msg.accepted_rates.is_empty() {
        return Err(ContractError::ConfigError {
            details: "At least one accepted rate must be provided".to_string(),
        });
    }

    let mut accepted_denoms = vec![];
    for rate in msg.accepted_rates {
        if rate.rate.is_zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }
        EXCHANGE_RATES.save(deps.storage, &rate.denom, &rate.rate)?;
        accepted_denoms.push(rate.denom);
    }

    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?,
        eho_token_address: deps.api.addr_validate(&msg.eho_token_address)?,
        accepted_payment_denoms: accepted_denoms,
        start_time: msg.start_time,
        end_time: msg.end_time,
        soft_cap: msg.soft_cap,
        hard_cap: msg.hard_cap,
        max_contribution_per_user: msg.max_contribution_per_user,
        eho_price: msg.eho_price,
    };
    CONFIG.save(deps.storage, &config)?;

    let state = State {
        total_usdc_raised: Uint128::zero(),
        sale_status: SaleStatus::Pending,
        paused: false,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy {} => execute_buy(deps, env, info),
        ExecuteMsg::ClaimTokens {} => execute_claim_tokens(deps, env, info),
        ExecuteMsg::RequestRefund {} => execute_request_refund(deps, env, info),
        ExecuteMsg::EndSale {} => execute_end_sale(deps, env, info),
        ExecuteMsg::AddToWhitelist { addresses } => execute_add_to_whitelist(deps, info, addresses),
        ExecuteMsg::RemoveFromWhitelist { addresses } => {
            execute_remove_from_whitelist(deps, info, addresses)
        }
        ExecuteMsg::ReclaimUnsoldTokens {} => execute_reclaim_unsold_tokens(deps, env, info),
        ExecuteMsg::WithdrawFunds {} => execute_withdraw_funds(deps, env, info),
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
        ExecuteMsg::UpdatePause { pause } => execute_update_pause(deps, info, pause),
    }
}

pub fn execute_buy(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if state.paused {
        return Err(ContractError::Paused {});
    }
    if state.sale_status == SaleStatus::Pending && env.block.time.seconds() >= config.start_time {
        state.sale_status = SaleStatus::Active;
    }
    if state.sale_status != SaleStatus::Active {
        return Err(ContractError::SaleNotActive {});
    }
    if env.block.time.seconds() >= config.end_time {
        return Err(ContractError::SaleHasEnded {});
    }
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidPayment {});
    }

    let payment = info.funds[0].clone();

    if payment.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let rate = EXCHANGE_RATES
        .may_load(deps.storage, &payment.denom)?
        .ok_or(ContractError::UnacceptedPaymentDenom {
            denom: payment.denom.clone(),
        })?;

    let usdc_value = payment.amount.multiply_ratio(rate, Uint128::new(1_000_000));

    if state.total_usdc_raised + usdc_value > config.hard_cap {
        return Err(ContractError::HardCapReached {});
    }

    let user_addr = &info.sender;
    // if !WHITELIST.load(deps.storage, user_addr).unwrap_or(false) {
    //     return Err(ContractError::NotInWhitelist {});
    // }

    let total_user_usdc_value = get_total_usdc_value(deps.as_ref(), user_addr)?;
    if total_user_usdc_value + usdc_value > config.max_contribution_per_user {
        return Err(ContractError::UserCapExceeded {});
    }

    state.total_usdc_raised += usdc_value;

    CONTRIBUTIONS.update(deps.storage, user_addr, |contributions| -> StdResult<_> {
        let mut user_contributions = contributions.unwrap_or_default();
        if let Some(existing_coin) = user_contributions
            .iter_mut()
            .find(|c| c.denom == payment.denom)
        {
            existing_coin.amount += payment.amount;
        } else {
            user_contributions.push(payment.clone());
        }
        Ok(user_contributions)
    })?;

    if state.total_usdc_raised == config.hard_cap {
        state.sale_status = SaleStatus::Succeeded;
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "buy")
        .add_attribute("buyer", user_addr.to_string())
        .add_attribute("paid_denom", payment.denom)
        .add_attribute("paid_amount", payment.amount)
        .add_attribute("usdc_value_added", usdc_value))
}

fn _end_sale_if_over(deps: DepsMut, env: Env) -> Result<State, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    if state.sale_status == SaleStatus::Active && env.block.time.seconds() >= config.end_time {
        state.sale_status = if state.total_usdc_raised >= config.soft_cap {
            SaleStatus::Succeeded
        } else {
            SaleStatus::Failed
        };
        STATE.save(deps.storage, &state)?;
    }
    Ok(state)
}

pub fn execute_claim_tokens(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = _end_sale_if_over(deps.branch(), env)?;
    if state.sale_status != SaleStatus::Succeeded {
        return Err(ContractError::SoftCapNotReached {});
    }
    let total_usdc_value = get_total_usdc_value(deps.as_ref(), &info.sender)?;
    if total_usdc_value.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }
    let config = CONFIG.load(deps.storage)?;
    let eho_to_send = total_usdc_value.multiply_ratio(Uint128::new(1_000_000), config.eho_price);
    CONTRIBUTIONS.remove(deps.storage, &info.sender);
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string(),
        amount: eho_to_send,
    };
    let wasm_msg = WasmMsg::Execute {
        contract_addr: config.eho_token_address.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    };
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(wasm_msg))
        .add_attribute("action", "claim_tokens"))
}

pub fn execute_request_refund(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = _end_sale_if_over(deps.branch(), env)?;
    if state.sale_status != SaleStatus::Failed {
        return Err(ContractError::SaleNotSucceeded {});
    }
    let user_contributions = CONTRIBUTIONS.load(deps.storage, &info.sender)?;
    if user_contributions.is_empty() {
        return Err(ContractError::NothingToRefund {});
    }
    CONTRIBUTIONS.remove(deps.storage, &info.sender);
    let refund_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: user_contributions,
    };
    Ok(Response::new()
        .add_message(CosmosMsg::Bank(refund_msg))
        .add_attribute("action", "request_refund"))
}

pub fn execute_end_sale(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let final_state = _end_sale_if_over(deps.branch(), env)?;
    Ok(Response::new()
        .add_attribute("action", "admin_end_sale")
        .add_attribute("final_status", format!("{:?}", final_state.sale_status)))
}

pub fn execute_reclaim_unsold_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    // --- Validation ---
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if (state.sale_status != SaleStatus::Succeeded) && (state.sale_status != SaleStatus::Failed) {
        return Err(ContractError::SaleIsStillActive {});
    }

    // Query the EHO token contract to see how many tokens this presale contract currently holds.
    let balance_query = cw20::Cw20QueryMsg::Balance {
        address: env.contract.address.to_string(),
    };
    let balance_response: cw20::BalanceResponse = deps
        .querier
        .query_wasm_smart(config.eho_token_address.clone(), &balance_query)?;

    let remaining_balance = balance_response.balance;

    if remaining_balance.is_zero() {
        return Err(ContractError::NoTokensToReclaim {});
    }

    // Create the CW20 Transfer message to send the remaining EHO tokens back to the admin.
    let reclaim_msg = Cw20ExecuteMsg::Transfer {
        recipient: config.admin.to_string(),
        amount: remaining_balance,
    };
    let wasm_msg = WasmMsg::Execute {
        contract_addr: config.eho_token_address.to_string(),
        msg: to_json_binary(&reclaim_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(wasm_msg))
        .add_attribute("action", "reclaim_unsold_tokens")
        .add_attribute("amount_reclaimed", remaining_balance))
}

pub fn execute_withdraw_funds(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let state = _end_sale_if_over(deps.branch(), env.clone())?;
    if state.sale_status != SaleStatus::Succeeded {
        return Err(ContractError::SaleNotSucceeded {});
    }
    let contract_addr = env.contract.address;
    let mut funds_to_withdraw: Vec<Coin> = vec![];
    for denom in config.accepted_payment_denoms {
        let balance = deps.querier.query_balance(contract_addr.clone(), denom)?;
        if !balance.amount.is_zero() {
            funds_to_withdraw.push(balance);
        }
    }
    if funds_to_withdraw.is_empty() {
        return Err(ContractError::NoFundsToWithdraw {});
    }
    let withdraw_msg = BankMsg::Send {
        to_address: config.admin.to_string(),
        amount: funds_to_withdraw,
    };
    Ok(Response::new()
        .add_message(CosmosMsg::Bank(withdraw_msg))
        .add_attribute("action", "withdraw_funds"))
}

pub fn execute_add_to_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    for addr_str in addresses {
        WHITELIST.save(deps.storage, &deps.api.addr_validate(&addr_str)?, &true)?;
    }
    Ok(Response::new().add_attribute("action", "add_to_whitelist"))
}

pub fn execute_remove_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    for addr_str in addresses {
        WHITELIST.remove(deps.storage, &deps.api.addr_validate(&addr_str)?);
    }
    Ok(Response::new().add_attribute("action", "remove_from_whitelist"))
}

pub fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized {});
        }
        config.admin = deps.api.addr_validate(&new_admin)?;
        Ok(config)
    })?;
    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", new_admin))
}

pub fn execute_update_pause(
    deps: DepsMut,
    info: MessageInfo,
    pause: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.paused = pause;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_attribute("action", "update_pause")
        .add_attribute("paused", pause.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::AcceptedRates {} => {
            let rates: StdResult<Vec<Rate>> = EXCHANGE_RATES
                .range(deps.storage, None, None, Order::Ascending)
                .map(|item| {
                    let (denom, rate) = item?;
                    Ok(Rate { denom, rate })
                })
                .collect();
            to_json_binary(&rates?)
        }
        QueryMsg::IsWhitelisted { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let is_whitelisted = WHITELIST.load(deps.storage, &addr).unwrap_or(false);
            to_json_binary(&is_whitelisted)
        }
        QueryMsg::TotalContributionOf { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let total_value = get_total_usdc_value(deps, &addr)?;
            to_json_binary(&total_value)
        }
        QueryMsg::ContributionsOf { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let contributions = CONTRIBUTIONS
                .may_load(deps.storage, &addr)?
                .unwrap_or_default();
            to_json_binary(&contributions)
        }
        QueryMsg::EhoAllocationOf { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let config = CONFIG.load(deps.storage)?;

            // Get the user's total contributed value in USDC
            let total_usdc_value = get_total_usdc_value(deps, &addr)?;

            // Calculate the EHO allocation using the same logic as the claim function
            let eho_allocation =
                total_usdc_value.multiply_ratio(Uint128::new(1_000_000), config.eho_price);

            to_json_binary(&eho_allocation)
        }
    }
}

fn get_total_usdc_value(deps: Deps, user: &cosmwasm_std::Addr) -> StdResult<Uint128> {
    let contributions = CONTRIBUTIONS
        .may_load(deps.storage, user)?
        .unwrap_or_default();
    let mut total_value = Uint128::zero();
    for coin in contributions {
        let rate = EXCHANGE_RATES.load(deps.storage, &coin.denom)?;
        let value = coin.amount.multiply_ratio(rate, Uint128::new(1_000_000));
        total_value += value;
    }
    Ok(total_value)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match msg {
        MigrateMsg::MinimalUpgrade {} => {}
    }
    Ok(Response::new()
        .add_attribute("action", "migrate_to_open_access")
        .add_attribute("new_version", CONTRACT_VERSION))
}
