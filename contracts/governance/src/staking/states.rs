use cosmwasm_std::{Addr, Deps, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terrapoker::governance::enumerations::PollStatus;

use crate::poll::states::{Poll, VoteInfo};

const STAKING_CONFIG: Item<StakingConfig> = Item::new("staking-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingConfig {
    pub distributor: Option<Addr>,
    pub unstake_lock_period: u64,
}

impl StakingConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKING_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingConfig> {
        STAKING_CONFIG.load(storage)
    }
}

const STAKING_STATE: Item<StakingState> = Item::new("staking-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingState {
    pub total_unstake_locked: Uint128,
}

impl StakingState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKING_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingState> {
        STAKING_STATE.load(storage)
    }
}


const STAKER_STATES: Map<&Addr, StakerState> = Map::new("staker-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerState {
    pub address: Addr,
    // total staked balance
    pub votes: Vec<(u64, VoteInfo)>, // maps poll_id to weight voted
    pub unstake_locked_list: Vec<(u64, Uint128)>,
}

impl StakerState {
    pub fn default(address: &Addr) -> StakerState {
        StakerState {
            address: address.clone(),
            votes: vec![],
            unstake_locked_list: vec![],
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKER_STATES.save(storage, &self.address, self)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<StakerState>> {
        STAKER_STATES.may_load(storage, address)
    }

    pub fn load_safe(storage: &dyn Storage, address: &Addr) -> StdResult<StakerState> {
        Ok(STAKER_STATES.may_load(storage, address)?.unwrap_or(StakerState::default(address)))
    }

    pub fn load_all(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<Vec<StakerState>> {
        let limit = limit.unwrap_or(10).min(100) as usize;

        let start_after = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));

        STAKER_STATES.range(deps.storage, start_after, None, Order::Ascending)
            .map(|d| Ok(d?.1))
            .take(limit)
            .collect::<StdResult<Vec<StakerState>>>()
    }

    pub fn clean_votes(&mut self, storage: &dyn Storage) -> () {
        self.votes.retain(|(poll_id, _)| {
            Poll::load(storage, &poll_id).ok()
                .map(|p| p.status == PollStatus::InProgress)
                .unwrap_or(false)
        });
    }

    // removes not in-progress poll voter info & unlock tokens
    // and returns the largest locked amount in participated polls.
    pub fn get_vote_locked_balance(&self) -> Uint128 {
        self.votes.iter()
            .map(|(_, v)| v.amount)
            .max()
            .unwrap_or_default()
    }

    pub fn clean_unstake_locked_list(&mut self, height: u64) {
        self.unstake_locked_list.retain(|(h, _a)| {
            h.clone() >= height
        });
    }

    pub fn get_unstake_claimable_amount(&self, height: u64) -> Uint128 {
        let mut amount = Uint128::zero();
        for (h, a) in self.unstake_locked_list.iter() {
            if h.clone() < height {
                amount += a;
            }
        }

        amount
    }

    // pub fn can_vote(&self, storage: &dyn Storage, contract_available_balance: Uint128, amount: Uint128) -> StdResult<bool> {
    //     let balance = self.load_balance(storage, contract_available_balance)?;
    //
    //     Ok(balance >= amount)
    // }

    pub fn vote(&mut self, poll_id: u64, vote: VoteInfo) {
        self.votes.push((poll_id, vote));
    }
}