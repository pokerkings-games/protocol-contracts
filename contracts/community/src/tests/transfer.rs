use cosmwasm_std::{Addr, CosmosMsg, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use terrapoker::common::ContractResult;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::{default_sender, DEFAULT_SENDER};
use terrapoker::test_constants::community::{ALLOWED_ADDRESS, community_env, MANAGING_TOKEN, COMMUNITY};
use terrapoker::test_constants::governance::{GOVERNANCE, governance_sender};
use terrapoker::test_utils::{expect_exceed_limit_err, expect_generic_err, expect_unauthorized_err};

use crate::executions::transfer;
use crate::states::Allowance;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    transfer(deps.as_mut(), env, info, recipient, amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    sender: &str,
    recipient: String,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = community_env();
    let info = mock_info(sender, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        recipient,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_allowed() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128::new(100),
    );

    let (_, _, response) = will_success(
        &mut deps,
        ALLOWED_ADDRESS,
        DEFAULT_SENDER.to_string(),
        Uint128::new(1),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DEFAULT_SENDER.to_string(),
                amount: Uint128::new(1),
            }).unwrap(),
            funds: vec![],
        })),
    ]);
}

#[test]
fn succeed_governance() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(100))],
    )]);

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        GOVERNANCE,
        DEFAULT_SENDER.to_string(),
        Uint128::new(100),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DEFAULT_SENDER.to_string(),
                amount: Uint128::new(100),
            }).unwrap(),
            funds: vec![],
        })),
    ]);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        community_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        Uint128::new(1),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_exceed_limit() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128::new(100),
    );

    will_success(
        &mut deps,
        ALLOWED_ADDRESS,
        DEFAULT_SENDER.to_string(),
        Uint128::new(99),
    );

    let result = exec(
        &mut deps,
        community_env(),
        mock_info(ALLOWED_ADDRESS, &[]),
        DEFAULT_SENDER.to_string(),
        Uint128::new(2),
    );

    expect_exceed_limit_err(&result);
}

#[test]
fn delete_after_exceed_limit() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128::new(100),
    );

    will_success(
        &mut deps,
        ALLOWED_ADDRESS,
        DEFAULT_SENDER.to_string(),
        Uint128::new(100),
    );

    let campaign = Allowance::may_load(
        &deps.storage,
        &Addr::unchecked(ALLOWED_ADDRESS),
    ).unwrap();
    assert!(campaign.is_none());
}

#[test]
fn failed_insufficient_free_balance() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128::new(100),
    );

    let result = exec(
        &mut deps,
        community_env(),
        governance_sender(),
        DEFAULT_SENDER.to_string(),
        Uint128::new(1),
    );

    expect_generic_err(&result, "Insufficient balance");
}