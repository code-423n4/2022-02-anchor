use crate::state::{Config, CONFIG};
use anchor_token::gov::VoteOption;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Uint128, Uint256, WasmMsg,
    WasmQuery,
};
use crossanchor::address_proxy::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crossanchor::anchor::{
    Cw20HookMsg as AnchorHookMsg, ExecuteMsg as AnchorExecuteMsg, QueryMsg as AnchorQueryMsg,
    WhitelistResponse,
};
use crossanchor::common::{coins_after_tax, forward_difference, forward_difference_msg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let cfg = Config {
        core_address: msg.core_address,
        market_address: msg.market_address,
        overseer_address: msg.overseer_address,
        aterra_address: msg.aterra_address,
        anc_token: msg.anc_token,
        anc_gov_address: msg.anc_gov_address,
        chain_id: msg.chain_id,
        address: msg.address,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    if let ExecuteMsg::Receive(msg) = msg.clone() {
        if msg.sender.as_str() != cfg.core_address.as_str() {
            return Err(StdError::generic_err("unauthorized"));
        }
    } else if info.sender.as_str() != cfg.core_address.as_str()
        && info.sender.as_str() != env.contract.address.as_str()
    {
        return Err(StdError::generic_err(format!(
            "unauthorized {} {}",
            info.sender.to_string(),
            cfg.core_address
        )));
    }

    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::RepayStable {} => repay_stable(deps, info),
        ExecuteMsg::UnlockCollateral { asset } => unlock_collateral(deps, env, asset),
        ExecuteMsg::BorrowStable { borrow_amount } => borrow_stable(deps, env, borrow_amount),
        ExecuteMsg::ClaimRewards {} => claim_rewards(deps, env),
        ExecuteMsg::ForwardDifference { asset_info, to } => {
            forward_difference(deps, env, info, asset_info, to, false)
        }
        ExecuteMsg::StakeVotingTokens { stake_amount } => {
            stake_voting_tokens(deps, env, stake_amount)
        }
        ExecuteMsg::WithdrawVotingTokens { unstake_amount } => {
            unstake_voting_tokens(deps, env, unstake_amount)
        }
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => cast_vote(deps, env, poll_id, vote, amount),
    }
}

pub fn stake_voting_tokens(deps: DepsMut, env: Env, stake_amount: Uint256) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.anc_gov_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::StakeVotingTokens { stake_amount })?,
        })]),
    )
}

pub fn unstake_voting_tokens(
    deps: DepsMut,
    env: Env,
    unstake_amount: Uint256,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.anc_gov_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::WithdrawVotingTokens { unstake_amount })?,
        })]),
    )
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.anc_gov_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::CastVote {
                poll_id,
                vote,
                amount,
            })?,
        })]),
    )
}

pub fn claim_rewards(deps: DepsMut, env: Env) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.market_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::ClaimRewards { to: None })?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::Token {
                contract_addr: cfg.anc_token,
            },
            cfg.core_address,
            false,
        )?,
    ]))
}

pub fn borrow_stable(deps: DepsMut, env: Env, borrow_amount: Uint256) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.market_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::BorrowStable {
                borrow_amount,
                to: None,
            })?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            cfg.core_address,
            false,
        )?,
    ]))
}

pub fn repay_stable(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.market_address,
            funds: coins_after_tax(&deps.querier, info.funds)?,
            msg: to_binary(&AnchorExecuteMsg::RepayStable {})?,
        })]),
    )
}

pub fn receive_cw20(deps: DepsMut, info: MessageInfo, msg: Cw20ReceiveMsg) -> StdResult<Response> {
    match from_binary(&msg.msg)? {
        Cw20HookMsg::LockCollateral {} => {
            lock_collateral(deps, info.sender.to_string(), msg.amount)
        }
    }
}

pub fn lock_collateral(
    deps: DepsMut,
    collateral_token: String,
    amount: Uint128,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let custody_contract = query_custody_contract(
        &deps.querier,
        cfg.overseer_address.clone(),
        collateral_token.clone(),
    )?;
    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collateral_token.clone(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: custody_contract,
                amount,
                msg: to_binary(&AnchorHookMsg::DepositCollateral {})?,
            })?,
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.overseer_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::LockCollateral {
                collaterals: vec![(collateral_token, Uint256::from(amount))],
            })?,
        }),
    ]))
}

pub fn unlock_collateral(deps: DepsMut, env: Env, asset: Asset) -> StdResult<Response> {
    let collateral_token;
    let amount;
    if let AssetInfo::Token { contract_addr } = asset.info.clone() {
        collateral_token = contract_addr;
        amount = asset.amount;
    } else {
        return Err(StdError::generic_err("collateral is not a contract"));
    }

    let cfg = CONFIG.load(deps.storage)?;
    let custody_contract = query_custody_contract(
        &deps.querier,
        cfg.overseer_address.clone(),
        collateral_token.clone(),
    )?;
    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.overseer_address,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::UnlockCollateral {
                collaterals: vec![(collateral_token, Uint256::from(amount))],
            })?,
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: custody_contract,
            funds: vec![],
            msg: to_binary(&AnchorExecuteMsg::WithdrawCollateral {
                amount: Some(Uint256::from(amount)),
            })?,
        }),
        forward_difference_msg(deps, env, asset.info, cfg.core_address, false)?,
    ]))
}

pub fn query_custody_contract(
    querier: &QuerierWrapper,
    overseer: String,
    collateral_token: String,
) -> StdResult<String> {
    let whitelist_res: WhitelistResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer,
            msg: to_binary(&AnchorQueryMsg::Whitelist {
                collateral_token: Some(collateral_token),
                start_after: None,
                limit: None,
            })?,
        }))?;

    Ok(whitelist_res.elems[0].custody_contract.clone())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
