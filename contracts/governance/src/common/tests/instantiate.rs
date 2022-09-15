use cosmwasm_std::{Env, MessageInfo, Response};

use terrapoker::common::ContractResult;
use terrapoker::governance::execute_msgs::ContractConfigInitMsg;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::governance::{governance_env, GOVERNANCE_TOKEN, STAKING_TOKEN};

use crate::common::executions;
use crate::common::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance_token: String,
    staking_token: String,
) -> ContractResult<Response> {
    let msg = ContractConfigInitMsg {
        governance_token,
        staking_token,
    };

    // Execute
    executions::instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE_TOKEN.to_string(),
        STAKING_TOKEN.to_string(),

    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    // Initialize
    let mut deps = custom_deps();

    let (env, _, _) = default(&mut deps);

    // Validate
    let contract_config = ContractConfig::load(&deps.storage).unwrap();

    assert_eq!(GOVERNANCE_TOKEN, contract_config.governance_token.as_str());
    assert_eq!(STAKING_TOKEN, contract_config.staking_token.as_str());
    assert_eq!(env.contract.address, contract_config.address);
}