use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::Asset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_core_bridge_addr: String,
    pub wormhole_token_bridge_addr: String,
    pub cross_anchor_core_addr: String,
    pub aust_cw20_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ProcessAnchorMessage {
        instruction_vaa: Binary,
        option_token_transfer_vaa: Option<Binary>,
    },
    SendAsset {
        asset: Asset,
    },

    /// Admin
    RegisterWormholeChainInfo {
        // Wormhole chain id.
        chain_id: u16,
        // address of contract on remote chain
        address: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    SequenceInfo { chain_id: u16, sequence: u64 },
}
