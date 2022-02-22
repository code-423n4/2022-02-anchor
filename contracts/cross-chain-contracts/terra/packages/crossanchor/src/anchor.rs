use anchor_token::gov::VoteOption;
use cosmwasm_std::{Decimal256, Uint128, Uint256};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketConfigResponse {
    pub owner_addr: String,
    pub aterra_contract: String,
    pub interest_model: String,
    pub distribution_model: String,
    pub overseer_contract: String,
    pub collector_contract: String,
    pub distributor_contract: String,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributorConfigResponse {
    pub gov_contract: String,
    pub anchor_token: String,
    pub whitelist: Vec<String>,
    pub spend_limit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OverseerConfigResponse {
    pub owner_addr: String,
    pub oracle_contract: String,
    pub market_contract: String,
    pub liquidation_contract: String,
    pub collector_contract: String,
    pub threshold_deposit_rate: Decimal256,
    pub target_deposit_rate: Decimal256,
    pub buffer_distribution_factor: Decimal256,
    pub anc_purchase_factor: Decimal256,
    pub stable_denom: String,
    pub epoch_period: u64,
    pub price_timeframe: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// just for mocking
    Receive(Cw20ReceiveMsg),
    /// to overseer
    LockCollateral {
        collaterals: Vec<(String, Uint256)>, // <(Collateral Token, Amount)>
    },
    UnlockCollateral {
        collaterals: Vec<(String, Uint256)>, // <(Collateral Token, Amount)>
    },
    /// to gov
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
    /// to market
    DepositStable {},

    BorrowStable {
        borrow_amount: Uint256,
        to: Option<String>,
    },

    RepayStable {},

    ClaimRewards {
        to: Option<String>,
    },
    /// to custody
    WithdrawCollateral {
        amount: Option<Uint256>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// to market and overseer
    Config {},
    /// to whitelist
    Whitelist {
        collateral_token: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// to custody
    DepositCollateral {},
    /// to market
    RedeemStable {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponseElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: String,
    pub collateral_token: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponse {
    pub elems: Vec<WhitelistResponseElem>,
}
