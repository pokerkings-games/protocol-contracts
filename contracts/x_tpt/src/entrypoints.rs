#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, };
use cw20::{MinterResponse};

use crate::queries::{query_all_accounts, query_available_balance, query_balance, query_balance_at, query_config};
use crate::state::{check_available_balance, Config, get_total_supply_at};
use cw20_base::ContractError;
use cw2::set_contract_version;
use terrapoker::xtpt::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use terrapoker::xtpt::query_msgs::QueryMsg;
use crate::executions::{execute_burn, execute_burn_from, execute_mint, execute_send, execute_send_from, execute_transfer, execute_transfer_from, mint};

const CONTRACT_NAME: &str = "xtpt-cw20-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut response = Response::new();
    response = response.add_attribute("action", "instantiate");

    Config {
        admin: deps.api.addr_validate(info.sender.as_str())?,
        governance: deps.api.addr_validate(msg.governance.as_str())?,
    }.save(deps.storage)?;

    cw20_base::contract::instantiate(
        deps,
        env.clone(),
        info,
        cw20_base::msg::InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            marketing: msg.marketing,
            mint: Some(MinterResponse {
                minter: env.contract.address.to_string(),
                cap: None,
            }),
    })?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Burn { amount } => {
            check_available_balance(deps.as_ref(), info.sender.clone(), amount)?;
            execute_burn(deps, env, info, amount)
        },
        ExecuteMsg::Transfer { recipient, amount } => {
            check_available_balance(deps.as_ref(), info.sender.clone(), amount)?;
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Send { contract, amount, msg } => {
            check_available_balance(deps.as_ref(), info.sender.clone(), amount)?;
            execute_send(deps, env, info, contract, amount, msg)
        }
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            check_available_balance(deps.as_ref(), deps.api.addr_validate(owner.as_str())?, amount)?;
            execute_transfer_from(deps, env, info, owner, recipient, amount)
        },
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => {
            check_available_balance(deps.as_ref(), deps.api.addr_validate(owner.as_str())?, amount)?;
            execute_send_from(deps, env, info, owner, contract, amount, msg)
        },
        ExecuteMsg::BurnFrom { owner, amount } => {
            check_available_balance(deps.as_ref(), deps.api.addr_validate(owner.as_str())?, amount)?;
            execute_burn_from(deps, env, info, owner, amount)
        },
        ExecuteMsg::Mint { recipient, amount } => {
            let config = Config::load(deps.storage)?;
            if config.is_governance(&info.sender) {
                mint(deps, env, info, recipient, amount)
            } else {
                execute_mint(deps, env, info, recipient, amount)
            }
        },
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => {
            cw20_base::contract::execute_update_marketing(deps, env, info, project, description, marketing)
        },
        ExecuteMsg::UploadLogo(logo) => {
            cw20_base::contract::execute_upload_logo(deps, env, info, logo)
        },
        ExecuteMsg::UpdateConfig {
            admin,
            governance,
        } => crate::executions::update_config(deps, env, info, admin, governance),
        ExecuteMsg::ApproveAdminNominee {} => crate::executions::approve_admin_nominee(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::BalanceAt { address, block } => {
            to_binary(&query_balance_at(deps, address, block)?)
        }
        QueryMsg::TotalSupplyAt { block } => to_binary(&get_total_supply_at(deps.storage, block)?),
        QueryMsg::TokenInfo {} => to_binary(&cw20_base::contract::query_token_info(deps)?),
        QueryMsg::Allowance { owner, spender } => to_binary(&cw20_base::allowances::query_allowance(deps, owner, spender)?),
        QueryMsg::Minter {} => to_binary(&cw20_base::contract::query_minter(deps)?),
        QueryMsg::MarketingInfo {} => to_binary(&cw20_base::contract::query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&cw20_base::contract::query_download_logo(deps)?),
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&cw20_base::enumerable::query_all_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => to_binary(&query_all_accounts(deps, start_after, limit)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::AvailableBalance { address } => to_binary(&query_available_balance(deps, address)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    if cw2::get_contract_version(deps.storage).is_err() {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, "1.0.8-beta.0".to_string())?;
    }

    //mig to v1.0.8-beta.0 to v1.0.8-beta.1
    // let info = cw2::get_contract_version(deps.storage)?;
    // if info.version == "v1.0.8-beta.0".to_string() {
    //     let router = &deps.api.addr_validate(msg.router.as_str())?;
    //     migrations::v108_beta0::migrate(deps.storage, &env, router)?;
    //
    //     set_contract_version(deps.storage, CONTRACT_NAME, "1.0.8-beta.1")?;
    // }

    Ok(Response::default())
}