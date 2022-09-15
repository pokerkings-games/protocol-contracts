use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use terrapoker::common::ContractResult;
use terrapoker::governance::execute_msgs::ContractConfigInitMsg;

use super::states::ContractConfig;
use terrapoker::utils::make_response;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    ContractConfig {
        address: env.contract.address,
        governance_token: deps.api.addr_validate(&msg.governance_token)?,
        staking_token: deps.api.addr_validate(&msg.staking_token)?,
    }.save(deps.storage)?;

    Ok(response)
}
