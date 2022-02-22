use crate::helpers::check_whitelisted;
use crate::state::{ADDRESS_PROXIES, CONFIG};
use anchor_token::gov::VoteOption;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint256, WasmMsg,
};

use crossanchor::address_proxy::{
    Cw20HookMsg as AddressProxyHookMsg, ExecuteMsg as AddressProxyExecuteMsg,
};

use crossanchor::common::{coins_after_tax, forward_difference_msg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};

pub fn stake_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
    stake_amount: Uint256,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: vec![],
            msg: to_binary(&AddressProxyExecuteMsg::StakeVotingTokens { stake_amount })?,
        })]),
    )
}

pub fn unstake_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
    unstake_amount: Uint256,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: vec![],
            msg: to_binary(&AddressProxyExecuteMsg::WithdrawVotingTokens { unstake_amount })?,
        })]),
    )
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: vec![],
            msg: to_binary(&AddressProxyExecuteMsg::CastVote {
                poll_id,
                vote,
                amount,
            })?,
        })]),
    )
}
