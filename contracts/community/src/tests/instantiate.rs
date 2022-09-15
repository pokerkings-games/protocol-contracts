use cosmwasm_std::{Addr, Api, Env, MessageInfo, Response, Uint128};

use terrapoker::common::ContractResult;
use terrapoker::community::execute_msgs::InstantiateMsg;
use terrapoker::mock_querier::{custom_deps, CustomDeps};
use terrapoker::test_constants::default_sender;
use terrapoker::test_constants::community::{ADMIN, community_env, MANAGING_TOKEN};

use crate::executions::instantiate;
use crate::states::{ContractConfig, ContractState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admin: String,
    managing_token: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        admin,
        managing_token,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = community_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        ADMIN.to_string(),
        MANAGING_TOKEN.to_string(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        admin: deps.api.addr_validate(ADMIN).unwrap(),
        managing_token: Addr::unchecked(MANAGING_TOKEN),
    });

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state, ContractState {
        remain_allowance_amount: Uint128::zero(),
    });
}
