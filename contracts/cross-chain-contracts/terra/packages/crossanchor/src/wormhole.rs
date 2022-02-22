use crate::byte_utils::ByteUtils;
use cosmwasm_std::{Binary, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::Asset;

// Validator Action Approval(VAA) data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParsedVAA {
    pub version: u8,
    pub guardian_set_index: u32,
    pub timestamp: u32,
    pub nonce: u32,
    pub len_signers: u8,

    pub emitter_chain: u16,
    pub emitter_address: Vec<u8>,
    pub sequence: u64,
    pub consistency_level: u8,
    pub payload: Vec<u8>,

    pub hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormholeCoreBridgeQueryMsg {
    VerifyVAA { vaa: Binary, block_time: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormholeCoreBridgeExecuteMsg {
    SubmitVAA { vaa: Binary },
    PostMessage { message: Binary, nonce: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormholeTokenBridgeExecuteMsg {
    DepositTokens {},
    InitiateTransfer {
        asset: Asset,
        recipient_chain: u16,
        recipient: Binary,
        fee: Uint128,
        nonce: u32,
    },
    SubmitVaa {
        data: Binary,
    },
}

pub struct Action;

impl Action {
    pub const TRANSFER: u8 = 1;
    pub const ATTEST_META: u8 = 2;
}

// 0 u8 action
// 1 [u8] payload

pub struct TokenBridgeMessage {
    pub action: u8,
    pub payload: Vec<u8>,
}

impl TokenBridgeMessage {
    pub fn deserialize(data: &[u8]) -> StdResult<Self> {
        let action = data.get_u8(0);
        let payload = &data[1..];

        Ok(TokenBridgeMessage {
            action,
            payload: payload.to_vec(),
        })
    }
}

//     0   u256     amount
//     32  [u8; 32] token_address
//     64  u16      token_chain
//     66  [u8; 32] recipient
//     98  u16      recipient_chain
//     100 u256     fee

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferInfo {
    pub amount: (u128, u128),
    pub token_address: Vec<u8>,
    pub token_chain: u16,
    pub recipient: Vec<u8>,
    pub recipient_chain: u16,
    pub fee: (u128, u128),
}

impl TransferInfo {
    pub fn deserialize(data: &[u8]) -> StdResult<Self> {
        let amount = data.get_u256(0);
        let token_address = data.get_bytes32(32).to_vec();
        let token_chain = data.get_u16(64);
        let recipient = data.get_bytes32(66).to_vec();
        let recipient_chain = data.get_u16(98);
        let fee = data.get_u256(100);

        Ok(TransferInfo {
            amount,
            token_address,
            token_chain,
            recipient,
            recipient_chain,
            fee,
        })
    }
}
