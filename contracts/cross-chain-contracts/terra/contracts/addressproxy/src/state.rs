use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub core_address: String,
    pub market_address: String,
    pub overseer_address: String,
    pub aterra_address: String,
    pub anc_token: String,
    pub anc_gov_address: String,
    pub chain_id: u16,
    pub address: Vec<u8>,
}

pub const CONFIG: Item<Config> = Item::new("config");
