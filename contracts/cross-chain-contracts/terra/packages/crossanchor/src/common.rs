use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, DepsMut, Env, MessageInfo, QuerierWrapper, Response,
    StdError, StdResult, Uint128, WasmMsg,
};
use cw_storage_plus::Map;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::querier::{query_balance, query_token_balance};

use crate::{address_proxy, core};

pub const FORWARD_BALANCES: Map<&[u8], Uint128> = Map::new("forward_balances");

pub fn coins_after_tax(querier: &QuerierWrapper, coins: Vec<Coin>) -> StdResult<Vec<Coin>> {
    let mut res = vec![];
    for coin in coins {
        let asset = Asset {
            amount: coin.amount,
            info: AssetInfo::NativeToken {
                denom: coin.denom.clone(),
            },
        };
        res.push(asset.deduct_tax(querier)?);
    }
    Ok(res)
}

pub fn query_asset_balance(
    querier: &QuerierWrapper,
    env: &Env,
    asset_info: &AssetInfo,
) -> StdResult<Uint128> {
    match asset_info.clone() {
        AssetInfo::NativeToken { denom } => {
            query_balance(querier, env.contract.address.clone(), denom)
        }
        AssetInfo::Token { contract_addr } => query_token_balance(
            querier,
            Addr::unchecked(contract_addr),
            env.contract.address.clone(),
        ),
    }
}

pub fn forward_difference_msg(
    deps: DepsMut,
    env: Env,
    asset_info: AssetInfo,
    to: String,
    initiate_bridge_transfer: bool,
) -> StdResult<CosmosMsg> {
    // only forward the amount gained between the creation of this message
    // and the execution of ForwardDifference
    // contracts that call this one may have expectations of the amounted forwarded
    // and already existing funds that third parties can send may mess that up
    let current_balance = query_asset_balance(&deps.querier, &env, &asset_info)?;
    FORWARD_BALANCES.save(
        deps.storage,
        asset_info.to_string().as_bytes(),
        &current_balance,
    )?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: if initiate_bridge_transfer {
            to_binary(
                &core::ExecuteMsg::ForwardDifferenceAndInitiateBridgeTransfer { asset_info, to },
            )?
        } else {
            to_binary(&address_proxy::ExecuteMsg::ForwardDifference { asset_info, to })?
        },
        funds: vec![],
    }))
}

pub fn forward_difference(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_info: AssetInfo,
    to: String,
    initiate_bridge_transfer: bool,
) -> StdResult<Response> {
    if info.sender.as_str() != env.contract.address.as_str() {
        return Err(StdError::generic_err("unauthorized"));
    }

    let old_balance = FORWARD_BALANCES.load(deps.storage, asset_info.to_string().as_bytes())?;

    let mut asset = Asset {
        info: asset_info.clone(),
        amount: query_asset_balance(&deps.querier, &env, &asset_info)? - old_balance,
    };
    let mut response = Response::new();
    response = response.add_message(
        asset
            .clone()
            .into_msg(&deps.querier, Addr::unchecked(to.clone()))?,
    );
    if initiate_bridge_transfer {
        if asset.is_native_token() {
            asset.amount = asset.deduct_tax(&deps.querier)?.amount;
        }
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: to,
            msg: to_binary(&crate::bridge::ExecuteMsg::SendAsset { asset })?,
            funds: vec![],
        }));
    }
    Ok(response)
}
