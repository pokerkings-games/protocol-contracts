#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError,
};
use cw20::Cw20ReceiveMsg;
use terrapoker::common::ContractResult;
use terrapoker::errors::ContractError;
use terrapoker::governance::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use terrapoker::governance::query_msgs::QueryMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    crate::common::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.contract_config,
    )?;
    crate::staking::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.staking_config,
    )?;
    crate::poll::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.poll_config,
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateStakingConfig {
            distributor,
            unstake_lock_period,
        } => crate::staking::executions::update_staking_config(
            deps,
            env,
            info,
            distributor,
            unstake_lock_period,
        ),
        ExecuteMsg::UpdatePollConfig {
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            proposal_deposit,
        } => crate::poll::executions::update_poll_config(
            deps,
            env,
            info,
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            proposal_deposit,
        ),
        ExecuteMsg::StakeGovernanceTokenHook {
            staker,
            amount,
        } => crate::staking::executions::stake_token_hook(
            deps, env, info, staker, amount,
        ),
        ExecuteMsg::UnstakeGovernanceTokenHook {
            staker,
            amount,
        } => crate::staking::executions::unstake_token_hook(
            deps, env, info, staker, amount,
        ),
        ExecuteMsg::ClaimUnstakedToken {} => crate::staking::executions::claim_unstaked_token(deps, env, info),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => crate::poll::executions::cast_vote(deps, env, info, poll_id, vote, amount),
        ExecuteMsg::EndPoll {
            poll_id,
        } => crate::poll::executions::end_poll(deps, env, info, poll_id),
        ExecuteMsg::ExecutePoll {
            poll_id,
        } => crate::poll::executions::execute_poll(deps, env, info, poll_id),
        ExecuteMsg::RunExecution {
            executions,
        } => crate::poll::executions::run_execution(deps, env, info, executions),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::StakeToken {} => crate::staking::executions::stake_token(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
        ),
        Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            executions,
        } => crate::poll::executions::create_poll(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
            title,
            description,
            link,
            executions,
        ),
        Cw20HookMsg::UnstakeGovernanceToken {} => crate::staking::executions::unstake_token(deps, env, info, Addr::unchecked(cw20_msg.sender), cw20_msg.amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        crate::poll::executions::REPLY_EXECUTION => {
            crate::poll::executions::reply_execution(deps, env, msg)
        }
        _ => Err(ContractError::Std(StdError::not_found("reply_id"))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::ContractConfig {} => to_binary(&crate::common::queries::get_contract_config(deps, env)?),
        QueryMsg::PollConfig {} => to_binary(&crate::poll::queries::get_poll_config(deps, env)?),
        QueryMsg::PollState {} => to_binary(&crate::poll::queries::get_poll_state(deps, env)?),
        QueryMsg::Poll { poll_id } => to_binary(&crate::poll::queries::get_poll(deps, env, poll_id)?),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::poll::queries::query_polls(
            deps,
            env,
            filter,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::poll::queries::query_voters(
            deps,
            env,
            poll_id,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::StakingConfig {} => to_binary(&crate::staking::queries::get_staking_config(deps, env)?),
        QueryMsg::StakingState {} => to_binary(&crate::staking::queries::get_staking_state(deps, env)?),
        QueryMsg::StakerState { address } => to_binary(&crate::staking::queries::get_staker_state(
            deps, env, address,
        )?),
        QueryMsg::AllStaker {
            start_after,
            limit,
        } => to_binary(&crate::staking::queries::get_all_stakers(deps, env, start_after, limit)?),
        QueryMsg::VotingPower { address } => to_binary(&crate::staking::queries::get_voting_power(
            deps, env, address,
        )?),
        QueryMsg::SimulateStakeAmount { amount } => to_binary(&crate::staking::queries::get_simulate_stake_amount(
            deps, env, amount,
        )?),
        QueryMsg::SimulateUnstakeAmount { amount } => to_binary(&crate::staking::queries::get_simulate_unstake_amount(
            deps, env, amount,
        )?),
    }?;

    Ok(result)
}