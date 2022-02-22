use cosmwasm_std::{Uint256, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{Asset, AssetInfo};
use anchor_token::gov::{VoteOption}; 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub address_proxy_code_id: u64,
    pub owner: String,
    pub overseer_address: String,
    pub anc_gov_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InitializeAddressProxy {
        chain_id: u16,
        address: Vec<u8>,
    },
    Receive(Cw20ReceiveMsg),
    /// depositors
    DepositStable {
        sender_chain: u16,
        sender_address: Vec<u8>,
    },
    /// borrowers
    RepayStable {
        sender_chain: u16,
        sender_address: Vec<u8>,
    },

    /// unlock + withdraw collateral (withdraw and unlock on anchor are combined into a single op)
    UnlockCollateral {
        sender_chain: u16,
        sender_address: Vec<u8>,
        asset: Asset,
    },
    BorrowStable {
        sender_chain: u16,
        sender_address: Vec<u8>,
        borrow_amount: Uint256,
    },
    ClaimRewards {
        sender_chain: u16,
        sender_address: Vec<u8>,
    },

    /// support governance
    StakeVotingTokens {
        sender_chain: u16,
        sender_address: Vec<u8>,
        stake_amount: Uint256,
    },
    WithdrawVotingTokens {
        sender_chain: u16,
        sender_address: Vec<u8>,
        unstake_amount: Uint256,
    },
    CastVote {
        sender_chain: u16,
        sender_address: Vec<u8>,
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },

    
    /// admin and internal
    AddBridges {
        bridges: Vec<String>,
    },

    UpdateConfig {
        owner: Option<String>,
        address_proxy_code_id: Option<u64>,
    },

    /// Internal Only
    ForwardDifferenceAndInitiateBridgeTransfer {
        asset_info: AssetInfo,
        to: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// require transfer of some CW20
    RedeemStable {
        sender_chain: u16,
        sender_address: Vec<u8>,
    },
    /// deposit + lock collateral (deposit and lock on anchor are combined into a single op)
    LockCollateral {
        sender_chain: u16,
        sender_address: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TerraAddress {
        sender_chain: u16,
        sender_address: Vec<u8>
    }
}
