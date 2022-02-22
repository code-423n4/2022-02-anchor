use anchor_token::gov::VoteOption;
use cosmwasm_std::Uint128;
use cosmwasm_std::Uint256;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{Asset, AssetInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub market_address: String,
    pub overseer_address: String,
    pub anc_gov_address: String,
    pub aterra_address: String,
    pub anc_token: String,
    pub core_address: String,
    pub chain_id: u16,
    pub address: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// borrowers
    RepayStable {},

    /// unlock + withdraw collateral (withdraw and unlock on anchor are combined into a single op)
    UnlockCollateral {
        asset: Asset,
    },
    BorrowStable {
        borrow_amount: Uint256,
    },
    ClaimRewards {},

    /// Internal Only
    ForwardDifference {
        asset_info: AssetInfo,
        to: String,
    },

    /// gov
    StakeVotingTokens {
        stake_amount: Uint256,
    },
    WithdrawVotingTokens {
        unstake_amount: Uint256,
    },
    CastVote {
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// deposit + lock collateral (deposit and lock on anchor are combined into a single op)
    LockCollateral {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
