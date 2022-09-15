use std::cmp::min;
use cosmwasm_std::{Decimal, Deps, Env, Uint128};

use terrapoker::common::ContractResult;
use terrapoker::governance::models::VoteInfoMsg;
use terrapoker::governance::query_msgs::{AllStakersResponse, StakerInfoResponse, StakerStateResponse, StakingStateResponse, VotingPowerResponse};

use crate::common::states::{load_contract_staking_token_balance, load_gov_token_balance, load_gov_token_total_supply};

use super::states::{StakerState, StakingState};
use crate::staking::states::StakingConfig;


pub fn get_staking_config(deps: Deps, _env: Env) -> ContractResult<StakingConfig> {
    Ok(StakingConfig::load(deps.storage)?)
}

pub fn get_staking_state(deps: Deps, _env: Env) -> ContractResult<StakingStateResponse> {
    let staking_state = StakingState::load(deps.storage)?;
    Ok(StakingStateResponse {
        total_unstake_locked: staking_state.total_unstake_locked,
    })
}

pub fn get_staker_state(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<StakerStateResponse> {
    let address = deps.api.addr_validate(&address)?;
    let staker_state = StakerState::may_load(deps.storage, &address)?;

    if staker_state.is_none() {
        return Ok(StakerStateResponse::default())
    }

    let mut staker_state = staker_state.unwrap();
    staker_state.clean_votes(deps.storage);

    let votes = staker_state
        .votes
        .iter()
        .map(|(poll_id, vote)| {
            let msg = VoteInfoMsg {
                voter: vote.voter.to_string(),
                option: vote.option.clone(),
                amount: vote.amount,
            };

            (*poll_id, msg)
        })
        .collect();

    Ok(StakerStateResponse {
        votes,
        locked_balance: staker_state.get_vote_locked_balance(),
        unstake_locked_list: staker_state.unstake_locked_list,
    })
}

pub fn get_voting_power(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<VotingPowerResponse> {
    let address = deps.api.addr_validate(&address)?;

    Ok(VotingPowerResponse {
        voting_power: Decimal::from_ratio(
            load_gov_token_balance(deps, &address, None)?,
            load_gov_token_total_supply(deps, None)?
        ),
    })
}

pub fn get_all_stakers(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> ContractResult<AllStakersResponse> {
    Ok(AllStakersResponse {
        stakers: StakerState::load_all(deps, start_after, limit)?.iter()
            .map(|s| StakerInfoResponse {
                address: s.address.to_string(),
            })
            .collect(),
    })
}

pub fn get_simulate_stake_amount(
    deps: Deps,
    _env: Env,
    amount: Uint128,
) -> ContractResult<Uint128> {
    let staked_amount = load_contract_staking_token_balance(deps)?;
    simulate_stake_amount(deps, staked_amount, amount)
}

pub fn simulate_stake_amount(
    deps: Deps,
    staked_amount: Uint128,
    staking_amount: Uint128,
) -> ContractResult<Uint128> {
    // TPT => xTPT

    let xtpt_total_supply = load_gov_token_total_supply(deps, None)?;
    if xtpt_total_supply.is_zero() {
        Ok(staking_amount)
    } else {
        let r1 = Decimal::from_ratio(xtpt_total_supply, staked_amount);
        let amount = staking_amount * r1;

        Ok(amount)
    }
}

pub fn get_simulate_unstake_amount(
    deps: Deps,
    _env: Env,
    amount: Uint128,
) -> ContractResult<Uint128> {
    simulate_unstake_amount(deps, amount)
}

pub fn simulate_unstake_amount(
    deps: Deps,
    amount: Uint128,
) -> ContractResult<Uint128> {
    // xTPT => TPT

    let xtpt_total_supply = load_gov_token_total_supply(deps, None)?;

    let share = min(Decimal::from_ratio(amount, xtpt_total_supply), Decimal::one());

    let staked_amount = load_contract_staking_token_balance(deps)?;
    let amount = staked_amount * share;

    Ok(amount)
}