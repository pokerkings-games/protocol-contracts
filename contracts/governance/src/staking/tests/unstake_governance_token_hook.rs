use cosmwasm_std::{Addr, CosmosMsg, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use terrapoker::common::ContractResult;
use terrapoker::message_matchers;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::governance::{GOVERNANCE, governance_env, GOVERNANCE_TOKEN, governance_sender};
use terrapoker::test_utils::expect_generic_err;
use terrapoker::utils::parse_uint128;

use crate::staking::executions::unstake_token_hook;
use crate::staking::states::{StakerState, StakingState};
use crate::staking::tests::stake_token_hook::{STAKER1, STAKER1_STAKE_AMOUNT, STAKER2, STAKER2_STAKE_AMOUNT};
use crate::tests::init_default;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, staker: String, amount: Option<Uint128>) -> ContractResult<Response> {
    let response = unstake_token_hook(
        deps.as_mut(),
        env,
        info,
        staker,
        amount,
    )?;

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

pub fn will_success(deps: &mut CustomDeps, staker: &str, amount: Option<Uint128>) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = mock_info(GOVERNANCE, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        staker.to_string(),
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::stake_token_hook::will_success(&mut deps, STAKER1, STAKER1_STAKE_AMOUNT);
    super::stake_token_hook::will_success(&mut deps, STAKER2, STAKER2_STAKE_AMOUNT);

    let increased_balance = (STAKER1_STAKE_AMOUNT + STAKER2_STAKE_AMOUNT)
        .checked_mul(Uint128::new(2))
        .unwrap();

    deps.querier.with_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(GOVERNANCE, &increased_balance)]
    )]);

    let (_, _, response) = will_success(&mut deps, STAKER1, None);

    let unstake_amount = response.attributes.iter()
        .find_map(|v| if v.key == "unstake_amount" {
            Some(parse_uint128(&v.value).unwrap())
        } else {
            None
        })
        .unwrap();

    let unstake_share = response.attributes.iter()
        .find_map(|v| if v.key == "unstake_share" {
            Some(parse_uint128(&v.value).unwrap())
        } else {
            None
        })
        .unwrap();

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: GOVERNANCE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: STAKER1.to_string(),
                amount: STAKER1_STAKE_AMOUNT.checked_mul(Uint128::new(2)).unwrap(),
            }).unwrap(),
        })),
    ]);

    let staking_state = StakingState::load(&deps.storage).unwrap();
    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();

    assert_eq!(unstake_amount, STAKER1_STAKE_AMOUNT.checked_mul(Uint128::new(2)).unwrap());
    assert_eq!(unstake_share, STAKER1_STAKE_AMOUNT);
    assert_eq!(staking_state.total_share, STAKER2_STAKE_AMOUNT);
    assert_eq!(staker_state.share, Uint128::zero());
}

#[test]
fn remove_completed_vote() {
    //TODO: Implement after poll test codes.
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::stake_token_hook::will_success(&mut deps, STAKER1, STAKER1_STAKE_AMOUNT);

    let result = exec(
        &mut deps,
        governance_env(),
        governance_sender(),
        STAKER1.to_string(),
        Some(STAKER1_STAKE_AMOUNT + Uint128::new(1)),
    );

    expect_generic_err(&result, "User is trying to unstake too many tokens.")
}

#[test]
fn failed_no_staked() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        governance_sender(),
        default_sender().sender.to_string(),
        None,
    );

    expect_generic_err(&result, "Nothing staked");
}