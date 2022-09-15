use cosmwasm_std::{Uint128, Decimal, Response, StdResult, StdError, Addr, Api};
use std::num::ParseIntError;

pub fn make_response(action: &str) -> Response {
    let mut response = Response::new();

    response = response.add_attribute("action", action);

    response
}

pub fn map_u128(value: Vec<Uint128>) -> Vec<u128> {
    value.iter().map(|v| v.u128()).collect()
}

pub fn map_uint128(value: Vec<u128>) -> Vec<Uint128> {
    value.iter().map(|&v| Uint128::from(v)).collect()
}

pub fn split_uint128(value: Uint128, ratio: &Vec<Uint128>) -> Vec<Uint128> {
    let total_amount = ratio.iter().sum::<Uint128>();

    ratio.iter()
        .map(|v| value.multiply_ratio(*v, total_amount))
        .collect()
}

pub fn split_ratio_uint128(value: Uint128, ratio: &Vec<Decimal>) -> Vec<Uint128> {
    ratio.iter().map(|r| *r * value).collect()
}

pub fn to_ratio_uint128(values: &Vec<Uint128>) -> Vec<Decimal> {
    let total_amount = values.iter().sum::<Uint128>();

    values.iter()
        .map(|v| Decimal::from_ratio(*v, total_amount))
        .collect()
}

pub fn parse_uint128(value: &str) -> Result<Uint128, ParseIntError> {
    let r = value.parse::<u128>().map(|v| Uint128::from(v));
    return if r.is_ok() {
        Ok(r.unwrap())
    } else {
        Err(r.unwrap_err() as ParseIntError)
    }
}

pub fn find_mut_or_push<T, P: Fn(&T) -> bool, N: Fn() -> T, F: Fn(&mut T)>(
    vec: &mut Vec<T>,
    predicate: P,
    new: N,
    f: F,
) {
    let item = vec.iter_mut().find(|v| predicate(*v));

    match item {
        None => vec.push(new()),
        Some(item) => f(item),
    }
}

pub fn find<T, P: Fn(&T) -> bool>(
    vec: &[T],
    predicate: P,
) -> Option<&T> {
    for each in vec {
        if predicate(each) {
            return Some(each)
        }
    }

    None
}

static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);
pub fn calc_ratio_amount(value: Uint128, ratio: Decimal) -> (Uint128, Uint128) {
    let base = value.multiply_ratio(DECIMAL_FRACTION, DECIMAL_FRACTION * ratio + DECIMAL_FRACTION);

    (value.checked_sub(base).unwrap(), base)
}

pub fn add_query_parameter(url: &str, key: &str, value: &str) -> String {
    let mut result = String::from(url);

    if result.contains('?') {
        if !(result.ends_with('&') || result.ends_with('?')) {
            result.push('&');
        }
    } else {
        result.push('?');
    }
    result.push_str(&key);
    result.push('=');
    result.push_str(&value);

    result
}

pub fn put_query_parameter(url: &str, key: &str, value: &str) -> String {
    let query_start_index = url.find('?');
    if query_start_index.is_none() {
        return add_query_parameter(url, key, value);
    }
    let query_start_index = query_start_index.unwrap();

    let query_string = &url[query_start_index..];
    let mut key_start_index = query_string.find(format!("?{}", key).as_str());
    if key_start_index.is_none() {
        key_start_index = query_string.find(format!("&{}", key).as_str());
    }
    if key_start_index.is_none() {
        return add_query_parameter(url, key, value);
    }
    let key_start_index = query_start_index + key_start_index.unwrap() + 1;

    let mut result = String::from(url);

    let key_end_index = key_start_index + key.len() - 1;
    if key_end_index >= url.len() - 1 {
        result.push('=');
    } else if url.chars().nth(key_end_index + 1).unwrap() != '=' {
        result.insert(key_end_index + 1, '=');
    }

    let value_start_index = key_start_index + key.len() + 1;
    if value_start_index > url.len() - 1 {
        result.push_str(value);
        return result
    }

    let mut value_end_index = result[value_start_index..].find('&');
    while value_end_index.is_some() && result[value_end_index.unwrap()..].starts_with("&amp") {
        value_end_index = result[(value_end_index.unwrap() + 1)..].find('&');
    }
    let value_end_index = value_end_index.unwrap_or(url.len() - 1);

    if value_start_index > value_end_index {
        result.insert_str(value_start_index, value);
    } else {
        result.replace_range(value_start_index..(value_end_index + 1), "");
        result.insert_str(value_start_index, value);
    }

    result
}

const TERRA_ADDRESS_LENGTH: usize = 44;

pub fn is_contract(address: &Addr) -> bool {
    address.to_string().len() > TERRA_ADDRESS_LENGTH
}

pub fn is_valid_schedule(distribution_schedule: &Vec<(u64, u64, Uint128)>) -> bool {
    let mut check_block = 0;

    for (start, end, _amount) in distribution_schedule.iter() {
        if start.clone() < check_block || start.clone() >= end.clone() {
            return false;
        }
        check_block = end.clone();
    }

    return true;
}

pub fn validate_zero_to_one(value: Decimal, name: &str) -> StdResult<()> {
    if Decimal::zero() <= value && value <= Decimal::one() {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("{} must be 0 to 1", name)))
    }
}

pub fn addr_validate_to_lower(api: &dyn Api, addr: impl Into<String>) -> StdResult<Addr> {
    let addr = addr.into();
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {} should be lowercase",
            addr
        )));
    }
    api.addr_validate(&addr)
}

pub fn addr_opt_validate(api: &dyn Api, addr: &Option<String>) -> StdResult<Option<Addr>> {
    addr.as_ref()
        .map(|addr| api.addr_validate(addr))
        .transpose()
}