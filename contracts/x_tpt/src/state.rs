use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Deps, Env, Order, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, IntKeyOld, Item, Map, SnapshotMap, Strategy};

use terrapoker::governance::query_msgs::{QueryMsg as GovQueryMsg, StakerStateResponse};
use crate::queries::query_balance;

pub const BALANCES: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "balance",
    "balance__checkpoints",
    "balance__changelog",
    Strategy::EveryBlock,
);

type U64Key = IntKeyOld<u64>;

pub const TOTAL_SUPPLY_HISTORY: Map<U64Key, Uint128> = Map::new("total_supply_history");

pub fn capture_total_supply_history(
    storage: &mut dyn Storage,
    env: &Env,
    total_supply: Uint128,
) -> StdResult<()> {
    TOTAL_SUPPLY_HISTORY.save(storage, U64Key::new(env.block.height), &total_supply)
}

/// ## Description
/// Returns a [`cosmwasm_std::StdError`] on failure, otherwise returns the total token supply at the given block.
/// ## Params
/// * **storage** is an object of type [`Storage`].
pub fn get_total_supply_at(storage: &dyn Storage, block: u64) -> StdResult<Uint128> {
    // Look for the last value recorded before the current block (if none then value is zero)
    let end = Bound::inclusive(U64Key::new(block));
    let last_value_up_to_block = TOTAL_SUPPLY_HISTORY
        .range(storage, None, Some(end), Order::Descending)
        .next();

    if let Some(value) = last_value_up_to_block {
        let (_, v) = value?;
        return Ok(v);
    }

    Ok(Uint128::zero())
}

const CONFIG: Item<Config> = Item::new("config");
const ADMIN_NOMINEE: Item<Addr> = Item::new("admin_nominee");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub governance: Addr,
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        &self.admin == address
    }

    pub fn is_governance(&self, address: &Addr) -> bool {
        &self.governance == address
    }

    pub fn may_load_admin_nominee(storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ADMIN_NOMINEE.may_load(storage)
    }

    pub fn save_admin_nominee(storage: &mut dyn Storage, address: &Addr) -> StdResult<()> {
        ADMIN_NOMINEE.save(storage, address)
    }
}

pub fn query_locked_balance(deps: Deps, address:String) -> StdResult<Uint128> {
    let governance = Config::load(deps.storage)?.governance;

    let res: StakerStateResponse = deps.querier.query_wasm_smart(
        governance.to_string(),
        &GovQueryMsg::StakerState {
            address,
        },
    )?;

    Ok(res.locked_balance)
}

pub fn check_available_balance(deps: Deps, address: Addr, amount: Uint128) -> StdResult<()> {
    let balance = query_balance(deps, address.to_string())?.balance;
    let locked = query_locked_balance(deps, address.to_string())?;
    let available = balance.checked_sub(locked)?;

    if amount > balance {
        // return 'OK'. this error will be processed next step.
        Ok(())
    } else if available >= amount {
        Ok(())
    } else {
        Err(StdError::generic_err(
            format!("your balance locked in governance contract. (total balance: {}, locked: {}, available: {})", balance, locked, available).as_str()))
    }
}