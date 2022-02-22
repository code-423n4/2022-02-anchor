use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map, U16Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
    pub wormhole_core_bridge_addr: Addr,
    pub wormhole_token_bridge_addr: Addr,
    pub cross_anchor_core_addr: Addr,
    pub aust_cw20_addr: Addr,
}

// Wormhole chain id for Terra (columbus-5 and bombay-12).
pub const TERRA_CHAIN_ID: u16 = 3u16;

// Config singleton.
pub const CONFIG: Item<Config> = Item::new("config");

// Map from Wormhole chain id to the Anchor bridge contract address.
type ChainIdKey = U16Key;
pub const CHAIN_ID_TO_ANCHOR_BRIDGE_ADDRESS_MAP: Map<ChainIdKey, Vec<u8>> =
    Map::new("chain_id_to_anchor_bridge_address_map");

pub const SEQUENCE_STORE: Map<(&[u8], &[u8]), SequenceInfo> = Map::new("sequence_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SequenceInfo {
    pub outgoing_sequence_expected: bool,
    pub outgoing_sequence: Option<u64>,
}

// Map for storing hashes of completed Anchor instruction messages.
pub const COMPLETED_INSTRUCTIONS: Map<&[u8], bool> = Map::new("completed_instructions");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OutgoingTokenTransferInfo {
    pub chain_id: u16,
    pub token_recipient_address: Vec<u8>,
    // Sequence of the outgoing token transfer.
    pub token_transfer_sequence: u64,
    // Sequence of the instruction message to which this token transfer is in response.
    // For example, if `instruction_sequence` is a deposit_stable op, then `token_transfer_sequence` is the sequence
    // of sending back aUST.
    pub instruction_sequence: u64,
}

impl OutgoingTokenTransferInfo {
    pub fn serialize(&self) -> Vec<u8> {
        [
            self.chain_id.to_be_bytes().to_vec(),
            self.token_recipient_address.clone(),
            self.token_transfer_sequence.to_be_bytes().to_vec(),
            self.instruction_sequence.to_be_bytes().to_vec(),
        ]
        .concat()
    }
}

pub const TMP_OUTGOING_TOKEN_TRANSFER_INFO: Item<OutgoingTokenTransferInfo> =
    Item::new("outgoing_token_transfer_info");
