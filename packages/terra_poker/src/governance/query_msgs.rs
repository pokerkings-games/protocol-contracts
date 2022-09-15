use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::common::OrderBy;
use super::enumerations::PollStatus;
use super::models::VoteInfoMsg;
use crate::common::ExecutionMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    PollConfig {},
    PollState {},
    Poll {
        poll_id: u64,
    },
    Polls {
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    Voters {
        poll_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    StakingConfig {},
    StakingState {},
    StakerState {
        address: String,
    },
    AllStaker {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    VotingPower {
        address: String,
    },
    SimulateStakeAmount {
        amount: Uint128,
    },
    SimulateUnstakeAmount {
        amount: Uint128,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub governance_token: String,
    pub staking_token: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StakingStateResponse {
    pub total_unstake_locked: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerStateResponse {
    pub votes: Vec<(u64, VoteInfoMsg)>,
    pub locked_balance: Uint128,
    pub unstake_locked_list: Vec<(u64, Uint128)>,
}

impl Default for StakerStateResponse {
    fn default() -> Self {
        StakerStateResponse {
            votes: vec![],
            locked_balance: Uint128::zero(),
            unstake_locked_list: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AllStakersResponse {
    pub stakers: Vec<StakerInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StakerInfoResponse {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PollConfigResponse {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub proposal_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PollStateResponse {
    pub poll_count: u64,
    pub total_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollResponse {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub executions: Vec<ExecutionMsg>,
    pub creator: String,
    pub deposit_amount: Uint128,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub abstain_votes: Uint128,
    pub start_height: u64,
    pub end_height: u64,
    pub status: PollStatus,
    pub total_balance_at_start_poll: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollCountResponse {
    pub poll_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotersResponse {
    pub voters: Vec<VoteInfoMsg>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotingPowerResponse {
    pub voting_power: Decimal,
}