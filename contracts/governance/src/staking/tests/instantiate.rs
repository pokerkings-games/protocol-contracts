use cosmwasm_std::{Env, MessageInfo, Response, Uint128};

use terrapoker::common::ContractResult;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::governance::governance_env;

use crate::staking::executions::instantiate;
use crate::staking::states::StakingState;
use terrapoker::governance::execute_msgs::StakingConfigInitMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    distributor: Option<String>,
) -> ContractResult<Response> {
    let msg = StakingConfigInitMsg {
        distributor,
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
        None,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    // Validate
    let staking_state = StakingState::load(&deps.storage).unwrap();
    assert_eq!(staking_state.total_share, Uint128::zero())
}
