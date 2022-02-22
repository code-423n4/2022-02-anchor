use crate::helpers::check_whitelisted;
use crate::state::{ADDRESS_PROXIES, CONFIG};
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,
    Uint128, Uint256, WasmMsg,
};
use crossanchor::address_proxy::{
    Cw20HookMsg as AddressProxyHookMsg, ExecuteMsg as AddressProxyExecuteMsg, MigrateMsg,
};
use crossanchor::common::{coins_after_tax, forward_difference_msg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};

/// 1. find the appropriate address proxy
/// 2. forward the message to the address proxy contract
/// 3. if there are any expected assets returned; forward those assets to the bridge/caller

pub fn lock_collateral(
    deps: DepsMut,
    sender_chain: u16,
    sender_address: Vec<u8>,
    collateral_address: String,
    collateral_amount: Uint128,
) -> StdResult<Response> {
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(Response::new()
        .add_message(migrate_address_proxy_msg(
            deps.storage,
            &address_proxy_addr,
        )?)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collateral_address,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: address_proxy_addr,
                amount: collateral_amount,
                msg: to_binary(&AddressProxyHookMsg::LockCollateral {})?,
            })?,
        })))
}

pub fn repay_stable(
    deps: DepsMut,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: &[u8],
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr =
        ADDRESS_PROXIES.load(deps.storage, (&sender_chain.to_be_bytes(), sender_address))?;
    Ok(Response::new()
        .add_message(migrate_address_proxy_msg(
            deps.storage,
            &address_proxy_addr,
        )?)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: coins_after_tax(&deps.querier, info.funds)?,
            msg: to_binary(&AddressProxyExecuteMsg::RepayStable {})?,
        })))
}

pub fn unlock_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
    asset: Asset,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(Response::new().add_messages(vec![
        migrate_address_proxy_msg(deps.storage, &address_proxy_addr)?,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: coins_after_tax(&deps.querier, info.funds)?,
            msg: to_binary(&AddressProxyExecuteMsg::UnlockCollateral {
                asset: asset.clone(),
            })?,
        }),
        forward_difference_msg(deps, env, asset.info, info.sender.to_string(), true)?,
    ]))
}

pub fn borrow_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
    borrow_amount: Uint256,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    Ok(Response::new().add_messages(vec![
        migrate_address_proxy_msg(deps.storage, &address_proxy_addr)?,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: vec![],
            msg: to_binary(&AddressProxyExecuteMsg::BorrowStable { borrow_amount })?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            info.sender.to_string(),
            true,
        )?,
    ]))
}

pub fn claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_chain: u16,
    sender_address: Vec<u8>,
) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let address_proxy_addr = ADDRESS_PROXIES.load(
        deps.storage,
        (&sender_chain.to_be_bytes(), sender_address.as_slice()),
    )?;
    let cfg = CONFIG.load(deps.storage)?;
    Ok(Response::new().add_messages(vec![
        migrate_address_proxy_msg(deps.storage, &address_proxy_addr)?,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address_proxy_addr,
            funds: coins_after_tax(&deps.querier, info.funds)?,
            msg: to_binary(&AddressProxyExecuteMsg::ClaimRewards {})?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::Token {
                contract_addr: cfg.anc_token,
            },
            info.sender.to_string(),
            true,
        )?,
    ]))
}

pub fn migrate_address_proxy_msg(
    storage: &dyn Storage,
    address_proxy_addr: &str,
) -> StdResult<CosmosMsg> {
    let cfg = CONFIG.load(storage)?;
    Ok(CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: address_proxy_addr.to_string(),
        new_code_id: cfg.address_proxy_code_id,
        msg: to_binary(&MigrateMsg {})?,
    }))
}
