use cosmwasm_std::{attr, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use crate::state::{BALANCES, capture_total_supply_history, Config};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw20_base::allowances::deduct_allowance;
use cw20_base::ContractError;
use cw20_base::state::TOKEN_INFO;
use terrapoker::utils::addr_validate_to_lower;

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    governance: Option<String>,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new();
    response = response.add_attribute("action", "update_config");

    let mut config = Config::load(deps.storage)?;

    if let Some(admin) = admin.as_ref() {
        Config::save_admin_nominee(deps.storage, &deps.api.addr_validate(admin)?)?;
        response = response.add_attribute("is_updated_admin_nominee", "true");
    }

    if let Some(governance) = governance {
        config.governance = deps.api.addr_validate(governance.as_str())?;
        response = response.add_attribute("is_updated_governance", "true");
    }

    config.save(deps.storage)?;
    Ok(response)
}

pub fn approve_admin_nominee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Execute
    let mut response = Response::new();
    response = response.add_attribute("action", "approve_admin_nominee");

    if let Some(admin_nominee) = Config::may_load_admin_nominee(deps.storage)? {
        if admin_nominee != info.sender {
            return Err(ContractError::Std(StdError::generic_err("It is not admin nominee")));
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = Config::load(deps.storage)?;
    config.admin = info.sender;
    response = response.add_attribute("is_updated_admin", "true");

    config.save(deps.storage)?;

    Ok(response)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let rcpt_addr = addr_validate_to_lower(deps.api, &recipient)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "transfer"),
        attr("from", info.sender),
        attr("to", rcpt_addr),
        attr("amount", amount),
    ]))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // Lower the sender's balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;

    // Reduce total_supply
    let token_info = TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    capture_total_supply_history(deps.storage, &env, token_info.total_supply)?;

    let res = Response::new().add_attributes(vec![
        attr("action", "burn"),
        attr("from", info.sender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let rcpt_addr = addr_validate_to_lower(deps.api, &contract)?;

    // Move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "send"),
            attr("from", &info.sender),
            attr("to", &rcpt_addr),
            attr("amount", amount),
        ])
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
                .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let rcpt_addr = addr_validate_to_lower(deps.api, &recipient)?;
    let owner_addr = addr_validate_to_lower(deps.api, &owner)?;

    // Deduct allowance before doing anything else
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    BALANCES.update(
        deps.storage,
        &owner_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_add(amount)?) },
    )?;

    let res = Response::new().add_attributes(vec![
        attr("action", "transfer_from"),
        attr("from", owner),
        attr("to", recipient),
        attr("by", info.sender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let owner_addr = addr_validate_to_lower(deps.api, &owner)?;

    // Deduct allowance before doing anything else
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // Lower balance
    BALANCES.update(
        deps.storage,
        &owner_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;

    // Reduce total_supply
    let token_info = TOKEN_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.total_supply = meta.total_supply.checked_sub(amount)?;
        Ok(meta)
    })?;

    capture_total_supply_history(deps.storage, &env, token_info.total_supply)?;

    let res = Response::new().add_attributes(vec![
        attr("action", "burn_from"),
        attr("from", owner),
        attr("by", info.sender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let rcpt_addr = addr_validate_to_lower(deps.api, &contract)?;
    let owner_addr = addr_validate_to_lower(deps.api, &owner)?;

    // Deduct allowance before doing anything else
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // Move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &owner_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_sub(amount)?) },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default().checked_add(amount)?) },
    )?;

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "send_from"),
            attr("from", &owner),
            attr("to", &contract),
            attr("by", &info.sender),
            attr("amount", amount),
        ])
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
                .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

pub fn mint(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    response = response.add_attribute("action", "mint");

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient,
            amount,
        })?,
    }));

    response = response.add_attribute("amount", amount.to_string());
    Ok(response)
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut config = TOKEN_INFO.load(deps.storage)?;

    if let Some(ref mint_data) = config.mint {
        if mint_data.minter.as_ref() != info.sender {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    // Update supply and enforce cap
    config.total_supply = config
        .total_supply
        .checked_add(amount)
        .map_err(StdError::from)?;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }

    TOKEN_INFO.save(deps.storage, &config)?;

    capture_total_supply_history(deps.storage, &env, config.total_supply)?;

    // Add amount to recipient balance
    let rcpt_addr = addr_validate_to_lower(deps.api, &recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "mint"),
        attr("to", rcpt_addr),
        attr("amount", amount),
    ]))
}