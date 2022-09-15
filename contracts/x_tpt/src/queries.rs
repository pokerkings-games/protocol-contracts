use cosmwasm_std::{Deps, Order, StdResult};
use cw20::{AllAccountsResponse, BalanceResponse};
use cw_storage_plus::Bound;
use terrapoker::utils::addr_opt_validate;
use terrapoker::xtpt::query_msgs::AvailableBalanceMsg;
use crate::state::{BALANCES, Config, query_locked_balance};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_config(deps: Deps) -> StdResult<Config> {
    Config::load(deps.storage)
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(address.as_str())?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_balance_at(deps: Deps, address: String, block: u64) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(address.as_str())?;
    let balance = BALANCES
        .may_load_at_height(deps.storage, &address, block)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_all_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAccountsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = addr_opt_validate(deps.api, &start_after)?;
    let start = start.as_ref().map(Bound::exclusive);

    let accounts = BALANCES
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|addr| addr.map(Into::into))
        .collect::<StdResult<_>>()?;

    Ok(AllAccountsResponse { accounts })
}

pub fn query_available_balance(
    deps: Deps,
    address: String,
) -> StdResult<AvailableBalanceMsg> {
    let total = query_balance(deps, address.clone())?.balance;
    let locked = query_locked_balance(deps, address)?;

    Ok(AvailableBalanceMsg {
        total,
        locked,
        available: total.checked_sub(locked)?,
    })
}