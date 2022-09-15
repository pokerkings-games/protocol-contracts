use cw_storage_plus::{Bound, Item, Map};
use cosmwasm_std::{Addr, Storage, StdResult, Uint128, QuerierWrapper, Env};
use terrapoker::common::OrderBy;
use terrapoker::community::query_msgs::{AllowancesResponse, AllowanceResponse, BalanceResponse};
use terrapoker::cw20::query_cw20_balance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");
const ADMIN_NOMINEE: Item<Addr> = Item::new("admin_nominee");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Addr,
    pub managing_token: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn may_load_admin_nominee(storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ADMIN_NOMINEE.may_load(storage)
    }

    pub fn save_admin_nominee(storage: &mut dyn Storage, address: &Addr) -> StdResult<()> {
        ADMIN_NOMINEE.save(storage, address)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin == *address
    }
}


const CONTRACT_STATE: Item<ContractState> = Item::new("contract-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractState {
    pub remain_allowance_amount: Uint128,
}

impl ContractState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractState> {
        CONTRACT_STATE.load(storage)
    }

    pub fn load_balance(
        &self,
        querier: &QuerierWrapper,
        env: &Env,
        token_address: &Addr,
    ) -> StdResult<BalanceResponse> {
        let total_balance = query_cw20_balance(
            querier,
            token_address,
            &env.contract.address,
        )?;

        Ok(BalanceResponse {
            total_balance,
            allowance_amount: self.remain_allowance_amount,
            free_balance: total_balance.checked_sub(self.remain_allowance_amount)?,
        })
    }
}


const ALLOWANCE: Map<&Addr, Allowance> = Map::new("allowance");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allowance {
    pub address: Addr,
    pub allowed_amount: Uint128,
    pub remain_amount: Uint128,
}

impl Allowance {
    pub fn default(address: &Addr) -> Allowance {
        Allowance {
            address: address.clone(),
            allowed_amount: Uint128::zero(),
            remain_amount: Uint128::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ALLOWANCE.save(storage, &self.address, self)
    }

    pub fn delete(&self, storage: &mut dyn Storage) {
        ALLOWANCE.remove(storage, &self.address)
    }

    pub fn save_or_delete(&self, storage: &mut dyn Storage) -> StdResult<()> {
        if self.remain_amount.is_zero() {
            self.delete(storage);
            Ok(())
        } else {
            self.save(storage)
        }
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Allowance> {
        ALLOWANCE.load(storage, address)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<Allowance>> {
        ALLOWANCE.may_load(storage, address)
    }

    pub fn load_or_default(storage: &dyn Storage, address: &Addr) -> StdResult<Allowance> {
        Ok(Self::may_load(storage, address)?.unwrap_or_else(|| Self::default(address)))
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<AllowancesResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

        let start_after = start_after.map(|s| Bound::ExclusiveRaw(s.into_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        let allowances = ALLOWANCE
            .range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (_, allowance) = item?;

                Ok(AllowanceResponse {
                    address: allowance.address.to_string(),
                    allowed_amount: allowance.allowed_amount,
                    remain_amount: allowance.remain_amount,
                })
            })
            .collect::<StdResult<Vec<AllowanceResponse>>>()?;

        Ok(AllowancesResponse {
            allowances,
        })
    }

    pub fn increase(&mut self, amount: Uint128) {
        self.allowed_amount += amount;
        self.remain_amount += amount;
    }

    pub fn decrease(&mut self, amount: Uint128) -> StdResult<()> {
        self.allowed_amount = self.allowed_amount.checked_sub(amount.clone())?;
        self.remain_amount = self.remain_amount.checked_sub(amount)?;

        Ok(())
    }
}
