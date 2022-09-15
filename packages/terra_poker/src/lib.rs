use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use errors::*;

pub mod governance;
pub mod distributor;
pub mod lp_staking;
pub mod community;
pub mod xtpt;

pub mod errors;
pub mod common;
pub mod cw20;
pub mod message_factories;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub mod mock_querier;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>) -> ListResponse<T> {
        ListResponse {
            items,
        }
    }
}
