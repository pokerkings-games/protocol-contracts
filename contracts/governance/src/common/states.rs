use cosmwasm_std::{Addr, StdResult, Storage, Deps, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terrapoker::cw20::query_cw20_balance;
use terrapoker::xtpt::query_msgs::QueryMsg as XtptQueryMsg;
use crate::staking::states::StakingState;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub address: Addr, // contract address
    pub governance_token: Addr,  //xTPT
    pub staking_token: Addr, //TPT
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_governance_token(&self, address: &Addr) -> bool {
        self.governance_token.eq(address)
    }

    pub fn is_staking_token(&self, address: &Addr) -> bool {
        self.staking_token.eq(address)
    }
}

pub fn load_gov_token_total_supply(deps: Deps, height: Option<u64>) -> StdResult<Uint128> {
    let contract_config = ContractConfig::load(deps.storage)?;

    if let Some(height) = height {
        let total_supply_at: Uint128 = deps.querier.query_wasm_smart(
            contract_config.governance_token,
            &XtptQueryMsg::TotalSupplyAt {
                block: height,
            },
        )?;

        Ok(total_supply_at)
    } else {
        let response: cw20::TokenInfoResponse = deps.querier.query_wasm_smart(
            contract_config.governance_token,
            &Cw20QueryMsg::TokenInfo {},
        )?;

        Ok(response.total_supply)
    }
}

pub fn load_gov_token_balance(deps: Deps, address: &Addr, height: Option<u64>) -> StdResult<Uint128> {
    let contract_config = ContractConfig::load(deps.storage)?;

    if let Some(height) = height {
        let gov_token_amount: BalanceResponse = deps.querier.query_wasm_smart(
            contract_config.governance_token,
            &XtptQueryMsg::BalanceAt {
                address: address.to_string(),
                block: height,
            },
        )?;

        Ok(gov_token_amount.balance)
    } else {
        let contract_balance = query_cw20_balance(
            &deps.querier,
            &contract_config.governance_token,
            address,
        )?;

        Ok(contract_balance)
    }
}

pub fn load_contract_staking_token_balance(deps: Deps) -> StdResult<Uint128> {
    let contract_config = ContractConfig::load(deps.storage)?;
    let contract_balance = query_cw20_balance(
        &deps.querier,
        &contract_config.staking_token,
        &contract_config.address,
    )?;

    let total_locked = StakingState::load(deps.storage)?.total_unstake_locked;

    Ok(contract_balance.checked_sub(total_locked)?)
}