use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128, SubMsg, CosmosMsg, WasmMsg, to_binary, Decimal, Deps};
use cw20::Cw20ExecuteMsg;

use terrapoker::common::ContractResult;
use terrapoker::errors::ContractError;

use crate::common::states::{ContractConfig, load_contract_staking_token_balance};

use super::states::{StakerState, StakingState};
use terrapoker::utils::{make_response};
use terrapoker::message_factories;
use terrapoker::governance::execute_msgs::{StakingConfigInitMsg, ExecuteMsg};
use crate::staking::queries::{simulate_unstake_amount, simulate_stake_amount};
use crate::staking::states::StakingConfig;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: StakingConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    StakingConfig {
        distributor: msg.distributor.map(|d| deps.api.addr_validate(d.as_str())).transpose()?,
        unstake_lock_period: msg.unstake_lock_period,
    }.save(deps.storage)?;

    StakingState {
        total_unstake_locked: Uint128::zero()
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_staking_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    distributor: Option<String>,
    unstake_lock_period: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_staking_config");

    let mut config = StakingConfig::load(deps.storage)?;

    if let Some(distributor) = distributor {
        config.distributor = Some(deps.api.addr_validate(distributor.as_str())?);
        response = response.add_attribute("is_updated_distributor", "true");
    }

    if let Some(unstake_lock_period) = unstake_lock_period {
        config.unstake_lock_period = unstake_lock_period;
        response = response.add_attribute("is_updated_unstake_lock_period", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn stake_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> ContractResult<Response> {
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_staking_token(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    let config = StakingConfig::load(deps.storage)?;

    let mut response = make_response("stake_token");

    if let Some(distributor) = config.distributor {
        response.messages.push(SubMsg::new(message_factories::wasm_execute(
            &distributor,
            &terrapoker::distributor::execute_msgs::ExecuteMsg::Distribute {
                id: None,
            },
        )));
    }

    response.messages.push(SubMsg::new(message_factories::wasm_execute(
        &env.contract.address,
        &ExecuteMsg::StakeGovernanceTokenHook {
            staker: sender.to_string(),
            amount,
        },
    )));

    Ok(response)
}

pub fn stake_token_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    // Execute
    let mut response = make_response("stake_token_hook");

    let sender = deps.api.addr_validate(staker.as_str())?;
    let contract_config = ContractConfig::load(deps.storage)?;


    let staked_amount = load_contract_staking_token_balance(deps.as_ref())?
        .checked_sub(amount)?;
    let mint_amount = simulate_stake_amount(deps.as_ref(), staked_amount, amount)?;

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_config.governance_token.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: sender.to_string(),
            amount: mint_amount,
        })?,
    }));

    response = response.add_attribute("sender", sender.as_str());
    response = response.add_attribute("amount", amount.to_string());
    response = response.add_attribute("mint_amount", mint_amount.to_string());

    Ok(response)
}

pub fn unstake_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: Addr,
    amount: Uint128,
) -> ContractResult<Response> {
    //xtpt -> tpt
    let contract_config = ContractConfig::load(deps.storage)?;
    if !contract_config.is_governance_token(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let config = StakingConfig::load(deps.storage)?;

    let mut response = make_response("stake_token");

    if let Some(distributor) = config.distributor {
        response.messages.push(SubMsg::new(message_factories::wasm_execute(
            &distributor,
            &terrapoker::distributor::execute_msgs::ExecuteMsg::Distribute {
                id: None,
            },
        )));
    }

    response.messages.push(SubMsg::new(message_factories::wasm_execute(
        &env.contract.address,
        &ExecuteMsg::UnstakeGovernanceTokenHook {
            staker: staker.to_string(),
            amount,
        },
    )));

    Ok(response)
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn unstake_token_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    amount: Uint128, //xtpt
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let sender = deps.api.addr_validate(staker.as_str())?;

    let mut staker_state = StakerState::load_safe(deps.storage, &sender)?;

    // Execute
    let mut response = make_response("unstake_token_hook");

    let mut staking_state = StakingState::load(deps.storage)?;

    staker_state.clean_votes(deps.storage);

    let withdraw_amount = simulate_unstake_amount(deps.as_ref(), amount)?;

    let config = StakingConfig::load(deps.storage)?;
    staker_state.unstake_locked_list.push((env.block.height + config.unstake_lock_period, withdraw_amount));
    staker_state.save(deps.storage)?;

    staking_state.total_unstake_locked += withdraw_amount;
    staking_state.save(deps.storage)?;


    let contract_config = ContractConfig::load(deps.storage)?;
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_config.governance_token.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount,
        }).unwrap(),
    }));

    response = response.add_attribute("unstake_amount", withdraw_amount);
    Ok(response)
}

pub fn claim_unstaked_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {

    let mut staker_state = StakerState::load_safe(deps.storage, &info.sender)?;
    let claimable_amount = staker_state.get_unstake_claimable_amount(env.block.height);

    let mut staking_state = StakingState::load(deps.storage)?;
    staking_state.total_unstake_locked -= claimable_amount;
    staking_state.save(deps.storage)?;

    staker_state.clean_unstake_locked_list(env.block.height);
    staker_state.save(deps.storage)?;

    let contract_config = ContractConfig::load(deps.storage)?;
    let mut response = make_response("claim_unstaked_token");
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_config.staking_token.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: claimable_amount,
        })
            .unwrap(),
    }));

    Ok(response)
}



