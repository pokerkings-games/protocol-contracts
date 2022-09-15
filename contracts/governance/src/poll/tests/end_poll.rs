use terrapoker::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, CosmosMsg, WasmMsg, Uint128, attr, to_binary, SubMsg};
use terrapoker::common::ContractResult;
use crate::poll::executions::end_poll;
use crate::tests::init_default;
use cw20::Cw20ExecuteMsg;
use cosmwasm_std::testing::mock_info;
use crate::poll::states::{Poll, PollResult};
use terrapoker::governance::enumerations::{PollStatus, VoteOption};
use crate::poll::tests::cast_vote::{VOTER1, VOTER2, VOTER3};
use terrapoker::message_matchers;
use crate::poll::tests::create_poll::PROPOSER1;
use terrapoker::test_utils::expect_generic_err;
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::governance::*;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, poll_id: u64) -> ContractResult<Response> {
    let response = end_poll(deps.as_mut(), env, info, poll_id)?;

    for msg in message_matchers::cw20_transfer(&response.messages) {
        deps.querier.minus_token_balances(&[(
            &msg.contract_addr,
            &[(GOVERNANCE, &msg.amount)],
        )]);
        deps.querier.plus_token_balances(&[(
            &msg.contract_addr,
            &[(&msg.recipient, &msg.amount)],
        )]);
    }

    Ok(response)
}

pub fn will_success(deps: &mut CustomDeps, poll_id: u64) -> (Env, MessageInfo, Response) {
    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let env = governance_env_height(poll.end_height + 1);

    let info = default_sender();

    let response = exec(deps, env.clone(), info.clone(), poll_id).unwrap();

    (env, info, response)
}

#[test]
fn succeed_passed() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128::new(100);
    let staker2_staked_amount = Uint128::new(100);
    let staker3_staked_amount = Uint128::new(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128::new(100));
    super::cast_vote::will_success(&mut deps, VOTER2, poll_id, VoteOption::No, Uint128::new(30));
    super::cast_vote::will_success(&mut deps, VOTER3, poll_id, VoteOption::Abstain, Uint128::new(100));

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: GOVERNANCE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: PROPOSER1.to_string(),
                amount: POLL_PROPOSAL_DEPOSIT,
            }).unwrap(),
        })),
    ]);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Passed);
}

#[test]
fn succeed_rejected_threshold_not_reached() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128::new(100);
    let staker2_staked_amount = Uint128::new(100);
    let staker3_staked_amount = Uint128::new(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128::new(30));
    super::cast_vote::will_success(&mut deps, VOTER2, poll_id, VoteOption::No, Uint128::new(100));
    super::cast_vote::will_success(&mut deps, VOTER3, poll_id, VoteOption::Abstain, Uint128::new(10));

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: GOVERNANCE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: PROPOSER1.to_string(),
                amount: POLL_PROPOSAL_DEPOSIT,
            }).unwrap(),
        })),
    ]);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::ThresholdNotReached.to_string()),
        attr("passed", "false"),
    ]);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
}

#[test]
fn succeed_rejected_quorum_not_reached() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128::new(100);
    let staker2_staked_amount = Uint128::new(100);
    let staker3_staked_amount = Uint128::new(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128::new(1));

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert!(response.messages.is_empty());

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn succeed_rejected_zero_quorum() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128::new(100);
    let staker2_staked_amount = Uint128::new(100);
    let staker3_staked_amount = Uint128::new(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake_token_hook::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert!(response.messages.is_empty());

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn succeed_end_poll_with_controlled_quorum() {
    //TODO:
}

#[test]
fn succeed_rejected_nothing_staked() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);

    let poll_id = 1u64;

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert!(response.messages.is_empty());

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn failed_before_end_height() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);

    let poll_id = 1u64;

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        poll_id,
    );

    expect_generic_err(&result, "Voting period has not expired");
}
