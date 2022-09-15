use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo};

use terrapoker::governance::execute_msgs::{ContractConfigInitMsg, InstantiateMsg, PollConfigInitMsg, StakingConfigInitMsg};
use terrapoker::test_constants::{contract_creator};
use terrapoker::test_constants::governance::*;

use crate::entrypoints;

pub fn init_default(deps: DepsMut) -> (Env, MessageInfo) {
    let env = governance_env();
    let info = contract_creator();

    let msg = InstantiateMsg {
        contract_config: ContractConfigInitMsg {
            governance_token: GOVERNANCE_TOKEN.to_string(),
            staking_token: STAKING_TOKEN.to_string(),
        },
        poll_config: PollConfigInitMsg {
            quorum: Decimal::percent(POLL_QUORUM_PERCENT),
            threshold: Decimal::percent(POLL_THRESHOLD_PERCENT),
            voting_period: POLL_VOTING_PERIOD,
            execution_delay_period: POLL_EXECUTION_DELAY_PERIOD,
            proposal_deposit: POLL_PROPOSAL_DEPOSIT,
        },
        staking_config: StakingConfigInitMsg {
            distributor: None,
        },
    };

    entrypoints::instantiate(deps, env.clone(), info.clone(), msg).unwrap();

    (env, info)
}