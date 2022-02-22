use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: String,
    pub address_proxy_code_id: u64,
    pub market_address: String,
    pub overseer_address: String,
    pub aterra_address: String,
    pub anc_token: String,
    pub anc_gov_address: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
// sender_chain, sender_address of the addressproxy contract being initialized
pub const CURRENT_ADDRESS_PROXY: Item<(u16, Vec<u8>)> = Item::new("current_address_proxy");

pub const WHITELISTED_BRIDGES: Map<&[u8], bool> = Map::new("whitelisted_bridges");
pub const ADDRESS_PROXIES: Map<(&[u8], &[u8]), String> = Map::new("address_proxies");
