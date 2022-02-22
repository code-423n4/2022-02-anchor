use crate::borrows::{
    borrow_stable, claim_rewards, lock_collateral, repay_stable, unlock_collateral,
};
use crate::deposits::{deposit_stable, redeem_stable};
use crate::gov::{cast_vote, stake_voting_tokens, unstake_voting_tokens};
use crate::helpers::check_whitelisted;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{Config, ADDRESS_PROXIES, CONFIG, CURRENT_ADDRESS_PROXY, WHITELISTED_BRIDGES};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use crossanchor::address_proxy::InstantiateMsg as AddressProxyInstantiateMsg;
use crossanchor::anchor::{
    DistributorConfigResponse, MarketConfigResponse, OverseerConfigResponse,
    QueryMsg as AnchorQueryMsg,
};
use crossanchor::common::forward_difference;
use crossanchor::core::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw20::Cw20ReceiveMsg;
use protobuf::Message;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let overseer_config: OverseerConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.overseer_address.clone(),
            msg: to_binary(&AnchorQueryMsg::Config {})?,
        }))?;

    let market_config: MarketConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer_config.market_contract.clone(),
            msg: to_binary(&AnchorQueryMsg::Config {})?,
        }))?;

    let distributor_config: DistributorConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market_config.distributor_contract.clone(),
            msg: to_binary(&AnchorQueryMsg::Config {})?,
        }))?;

    let cfg = Config {
        owner: msg.owner,
        address_proxy_code_id: msg.address_proxy_code_id,
        market_address: overseer_config.market_contract,
        overseer_address: msg.overseer_address,
        aterra_address: market_config.aterra_contract,
        anc_token: distributor_config.anchor_token,
        anc_gov_address: msg.anc_gov_address,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::InitializeAddressProxy { chain_id, address } => {
            initialize_address_proxy(deps, env, chain_id, &address)
        }
        ExecuteMsg::DepositStable { .. } => deposit_stable(deps, env, info),
        ExecuteMsg::RepayStable {
            sender_chain,
            sender_address,
        } => repay_stable(deps, info, sender_chain, &sender_address),
        ExecuteMsg::UnlockCollateral {
            sender_chain,
            sender_address,
            asset,
        } => unlock_collateral(deps, env, info, sender_chain, sender_address, asset),
        ExecuteMsg::BorrowStable {
            sender_chain,
            sender_address,
            borrow_amount,
        } => borrow_stable(deps, env, info, sender_chain, sender_address, borrow_amount),
        ExecuteMsg::ClaimRewards {
            sender_chain,
            sender_address,
        } => claim_rewards(deps, env, info, sender_chain, sender_address),
        ExecuteMsg::AddBridges { bridges } => add_bridges(deps, info, &bridges),
        ExecuteMsg::ForwardDifferenceAndInitiateBridgeTransfer { asset_info, to } => {
            forward_difference(deps, env, info, asset_info, to, true)
        }
        ExecuteMsg::UpdateConfig {
            owner,
            address_proxy_code_id,
        } => update_config(deps, info, owner, address_proxy_code_id),
        ExecuteMsg::StakeVotingTokens {
            sender_chain,
            sender_address,
            stake_amount,
        } => stake_voting_tokens(deps, env, info, sender_chain, sender_address, stake_amount),
        ExecuteMsg::WithdrawVotingTokens {
            sender_chain,
            sender_address,
            unstake_amount,
        } => unstake_voting_tokens(
            deps,
            env,
            info,
            sender_chain,
            sender_address,
            unstake_amount,
        ),
        ExecuteMsg::CastVote {
            sender_chain,
            sender_address,
            poll_id,
            vote,
            amount,
        } => cast_vote(
            deps,
            env,
            info,
            sender_chain,
            sender_address,
            poll_id,
            vote,
            amount,
        ),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    address_proxy_code_id: Option<u64>,
) -> StdResult<Response> {
    let mut cfg = CONFIG.load(deps.storage)?;
    if cfg.owner.as_str() != info.sender.as_str() {
        return Err(StdError::generic_err("unauthorized"));
    }
    cfg.owner = owner.unwrap_or(cfg.owner);
    cfg.address_proxy_code_id = address_proxy_code_id.unwrap_or(cfg.address_proxy_code_id);

    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let sender = msg.sender;
    check_whitelisted(deps.storage, sender.as_str())?;
    match from_binary(&msg.msg)? {
        Cw20HookMsg::LockCollateral {
            sender_chain,
            sender_address,
        } => lock_collateral(
            deps,
            sender_chain,
            sender_address,
            info.sender.to_string(),
            msg.amount,
        ),
        Cw20HookMsg::RedeemStable { .. } => {
            redeem_stable(deps, env, sender, info.sender.to_string(), msg.amount)
        }
    }
}

pub fn add_bridges(deps: DepsMut, info: MessageInfo, bridges: &[String]) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    if cfg.owner.as_str() != info.sender.as_str() {
        return Err(StdError::generic_err("unauthorized"));
    }

    for bridge in bridges {
        WHITELISTED_BRIDGES.save(deps.storage, bridge.as_bytes(), &true)?;
    }
    Ok(Response::new())
}

pub fn initialize_address_proxy(
    deps: DepsMut,
    env: Env,
    chain_id: u16,
    address: &[u8],
) -> StdResult<Response> {
    // deploy a contract that represents some user on a different chain
    // this is necessary for liquidations to be handled properly
    if ADDRESS_PROXIES
        .load(deps.storage, (&chain_id.to_be_bytes(), address))
        .is_ok()
    {
        return Ok(Response::new());
    }
    CURRENT_ADDRESS_PROXY.save(deps.storage, &(chain_id, address.to_vec()))?;
    let cfg = CONFIG.load(deps.storage)?;

    Ok(Response::new().add_submessage(SubMsg {
        id: 0u64,
        msg: WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: cfg.address_proxy_code_id,
            msg: to_binary(&AddressProxyInstantiateMsg {
                market_address: cfg.market_address,
                overseer_address: cfg.overseer_address,
                anc_gov_address: cfg.anc_gov_address,
                aterra_address: cfg.aterra_address,
                anc_token: cfg.anc_token,
                core_address: env.contract.address.to_string(),
                chain_id,
                address: address.to_vec(),
            })?,
            funds: vec![],
            label: "".to_string(),
        }
        .into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(msg.result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let (chain_id, address) = CURRENT_ADDRESS_PROXY.load(deps.storage)?;
    let address_proxy_address = res.get_contract_address().to_owned();
    ADDRESS_PROXIES.save(
        deps.storage,
        (&chain_id.to_be_bytes(), address.as_slice()),
        &address_proxy_address,
    )?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::TerraAddress {
            sender_chain,
            sender_address,
        } => {
            let terra_address = ADDRESS_PROXIES.load(
                deps.storage,
                (&sender_chain.to_be_bytes(), sender_address.as_slice()),
            )?;
            to_binary(&terra_address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
