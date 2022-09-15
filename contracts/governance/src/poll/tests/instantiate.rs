use cosmwasm_std::{Decimal, Env, MessageInfo, Response, Uint128};

use terrapoker::common::ContractResult;
use terrapoker::governance::execute_msgs::PollConfigInitMsg;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::governance::*;
use terrapoker::test_utils::expect_generic_err;

use crate::poll::executions::instantiate;
use crate::poll::states::{PollConfig, PollState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    quorum: Decimal,
    threshold: Decimal,
    voting_period: u64,
    execution_delay_period: u64,
    proposal_deposit: Uint128,
) -> ContractResult<Response> {
    let msg = PollConfigInitMsg {
        quorum,
        threshold,
        voting_period,
        execution_delay_period,
        proposal_deposit,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        Decimal::percent(POLL_QUORUM_PERCENT),
        Decimal::percent(POLL_THRESHOLD_PERCENT),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
    ).unwrap();

    (env, info, response)
}


#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    let poll_config = PollConfig::load(&deps.storage).unwrap();
    assert_eq!(poll_config.quorum, Decimal::percent(POLL_QUORUM_PERCENT));
    assert_eq!(poll_config.threshold, Decimal::percent(POLL_THRESHOLD_PERCENT));
    assert_eq!(poll_config.voting_period, POLL_VOTING_PERIOD);
    assert_eq!(poll_config.execution_delay_period, POLL_EXECUTION_DELAY_PERIOD);
    assert_eq!(poll_config.proposal_deposit, POLL_PROPOSAL_DEPOSIT);

    let poll_state = PollState::load(&deps.storage).unwrap();
    assert_eq!(poll_state.poll_count, 0);
    assert_eq!(poll_state.total_deposit, Uint128::zero());
}

#[test]
fn failed_invalid_threshold() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        Decimal::percent(POLL_QUORUM_PERCENT),
        Decimal::percent(101),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
    );

    expect_generic_err(&result, "threshold must be 0 to 1");
}

#[test]
fn failed_invalid_quorum() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        Decimal::percent(101),
        Decimal::percent(POLL_THRESHOLD_PERCENT),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
    );

    expect_generic_err(&result, "quorum must be 0 to 1");
}

#[test]
fn failed_invalid_execution_delay_period() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        Decimal::percent(POLL_QUORUM_PERCENT),
        Decimal::percent(POLL_THRESHOLD_PERCENT),
        POLL_VOTING_PERIOD,
        999,
        POLL_PROPOSAL_DEPOSIT,
    );

    expect_generic_err(&result, "execution_delay_period must be greater than 1000");
}