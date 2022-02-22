use crate::helpers::check_whitelisted;
use crate::state::CONFIG;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use crossanchor::anchor::{Cw20HookMsg as AnchorHookMsg, ExecuteMsg as AnchorExecuteMsg};
use crossanchor::common::{coins_after_tax, forward_difference_msg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::AssetInfo;

pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    bridge_address: String,
    contract_address: String,
    amount: Uint128,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;

    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_address,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: cfg.market_address,
                amount,
                msg: to_binary(&AnchorHookMsg::RedeemStable {})?,
            })?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            bridge_address,
            true,
        )?,
    ]))
}

pub fn deposit_stable(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    check_whitelisted(deps.storage, info.sender.as_str())?;
    let cfg = CONFIG.load(deps.storage)?;

    Ok(Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.market_address,
            funds: coins_after_tax(&deps.querier, info.funds)?,
            msg: to_binary(&AnchorExecuteMsg::DepositStable {})?,
        }),
        forward_difference_msg(
            deps,
            env,
            AssetInfo::Token {
                contract_addr: cfg.aterra_address,
            },
            info.sender.to_string(),
            true,
        )?,
    ]))
}
