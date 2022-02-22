use std::vec;

use crate::state::{
    Config, OutgoingTokenTransferInfo, SequenceInfo, CHAIN_ID_TO_ANCHOR_BRIDGE_ADDRESS_MAP,
    COMPLETED_INSTRUCTIONS, CONFIG, SEQUENCE_STORE, TERRA_CHAIN_ID,
    TMP_OUTGOING_TOKEN_TRANSFER_INFO,
};
use crate::util::{
    get_next_sequence, ANCHOR_BORROW_OPS, FLAG_INCOMING_TRANSFER, FLAG_OUTGOING_TRANSFER,
    OP_CODE_BORROW_STABLE, OP_CODE_CLAIM_REWARDS, OP_CODE_DEPOSIT_STABLE, OP_CODE_REDEEM_STABLE,
    OP_CODE_REPAY_STABLE, OP_CODE_UNLOCK_COLLATERAL,
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Coin, ContractResult, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use crossanchor::bridge::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crossanchor::byte_utils::{extend_terra_address_to_32, ByteUtils};
use crossanchor::core;
use crossanchor::wormhole::{
    Action, ParsedVAA, TokenBridgeMessage, TransferInfo, WormholeCoreBridgeExecuteMsg,
    WormholeCoreBridgeQueryMsg, WormholeTokenBridgeExecuteMsg,
};
use cw_storage_plus::U16Key;
use terraswap::asset::{Asset, AssetInfo};

static TOKEN_TRANSFER_SUBMIT_VAA_MSG_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            wormhole_core_bridge_addr: deps.api.addr_validate(&msg.wormhole_core_bridge_addr)?,
            wormhole_token_bridge_addr: deps.api.addr_validate(&msg.wormhole_token_bridge_addr)?,
            cross_anchor_core_addr: deps.api.addr_validate(&msg.cross_anchor_core_addr)?,
            aust_cw20_addr: deps.api.addr_validate(&msg.aust_cw20_addr)?,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        // execute instructions from the e side
        ExecuteMsg::ProcessAnchorMessage {
            instruction_vaa,
            option_token_transfer_vaa,
        } => process_anchor_message(deps, env, instruction_vaa, option_token_transfer_vaa),
        ExecuteMsg::SendAsset { asset } => send_asset(deps, env, info, asset),
        ExecuteMsg::RegisterWormholeChainInfo { chain_id, address } => {
            register_wormhole_chain_info(deps, info, chain_id, address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SequenceInfo { chain_id, sequence } => to_binary(&SEQUENCE_STORE.load(
            deps.storage,
            (&chain_id.to_be_bytes(), &sequence.to_be_bytes()),
        )?),
    }
}

fn register_wormhole_chain_info(
    deps: DepsMut,
    info: MessageInfo,
    chain_id: u16,
    address: Vec<u8>,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender.as_str() != cfg.owner.as_str() {
        return Err(StdError::generic_err("unauthorized"));
    }
    CHAIN_ID_TO_ANCHOR_BRIDGE_ADDRESS_MAP.save(deps.storage, U16Key::from(chain_id), &address)?;
    Ok(Response::default())
}

fn get_parsed_vaa(deps: Deps, env: &Env, config: &Config, vaa: &Binary) -> StdResult<ParsedVAA> {
    deps.querier.query_wasm_smart(
        config.wormhole_core_bridge_addr.clone(),
        &WormholeCoreBridgeQueryMsg::VerifyVAA {
            vaa: vaa.clone(),
            block_time: env.block.time.seconds(),
        },
    )
}

/*
struct Instruction {
    uint8 op_code; // [1 byte]
    bytes32 sender_address; // [32 bytes]
    one_of {
        // [deposit_stable] opcode = 0
        uint64 sequence; // [8 bytes]

        // [repay_stable] opcode = 1
        uint64 sequence; // [8 bytes]

        // [unlock_collateral] opcode = 2
        bytes32 collorateral_token_address; // [32 bytes]
        uint128 amount; // [16 bytes]

        // [borrow_stable] opcode = 3
        uint256 amount; // [16 bytes]

        // [claim rewards] opcode = 4
        // N/A for now

        // [redeem_stable] opcode = 5
        uint64 sequence; // [8 bytes]

        // [lock_collateral] opcode = 6
        uint64 sequence; // [8 bytes]
    }
}
*/

fn create_core_execute_message(
    config: &Config,
    msg: core::ExecuteMsg,
    funds: Vec<Coin>,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cross_anchor_core_addr.to_string(),
        msg: to_binary(&msg).unwrap(),
        funds,
    })
}

fn process_anchor_message(
    deps: DepsMut,
    env: Env,
    // generic message
    instruction_vaa: Binary,
    // accompanying token transfer
    option_token_transfer_vaa: Option<Binary>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let parsed_instruction_vaa = get_parsed_vaa(deps.as_ref(), &env, &config, &instruction_vaa)?;
    let expected_anchor_bridge_address = CHAIN_ID_TO_ANCHOR_BRIDGE_ADDRESS_MAP.load(
        deps.storage,
        U16Key::from(parsed_instruction_vaa.emitter_chain),
    )?;
    if parsed_instruction_vaa.emitter_address != expected_anchor_bridge_address {
        return Err(StdError::generic_err("unexpected Anchor bridge address"));
    }

    // block replay attacks
    let completed = COMPLETED_INSTRUCTIONS
        .load(deps.storage, parsed_instruction_vaa.hash.as_slice())
        .unwrap_or(false);
    if completed {
        return Err(StdError::generic_err("instruction already completed"));
    }
    COMPLETED_INSTRUCTIONS.save(deps.storage, parsed_instruction_vaa.hash.as_slice(), &true)?;

    static OP_CODE_INDEX: usize = 0;
    static SENDER_ADDRESS_INDEX: usize = 1;
    static ADDRESS_LEN: usize = 32;
    static OP_SPECIFIC_PAYLOAD_INDEX: usize = 33;
    let op_code = parsed_instruction_vaa
        .payload
        .as_slice()
        .get_u8(OP_CODE_INDEX);
    let sender_chain = parsed_instruction_vaa.emitter_chain;
    let sender_address = parsed_instruction_vaa.payload.as_slice()
        [SENDER_ADDRESS_INDEX..SENDER_ADDRESS_INDEX + ADDRESS_LEN]
        .to_vec();

    // this contract is going to call crossanchor-core
    // crossanchor-core will send tokens to this address and this contract
    // is expected to forward those tokens over the wormhole
    // take note of the recipient here
    if op_code & FLAG_OUTGOING_TRANSFER != 0 {
        let outgoing_token_transfer_info = OutgoingTokenTransferInfo {
            chain_id: sender_chain,
            token_recipient_address: sender_address.clone(),
            token_transfer_sequence: get_next_sequence(
                deps.as_ref(),
                &config,
                &config.wormhole_token_bridge_addr,
            )?,
            instruction_sequence: parsed_instruction_vaa.sequence,
        };
        TMP_OUTGOING_TOKEN_TRANSFER_INFO.save(deps.storage, &outgoing_token_transfer_info)?;
    }

    // record an ack on the terra side
    SEQUENCE_STORE.save(
        deps.storage,
        (
            &sender_chain.to_be_bytes(),
            &parsed_instruction_vaa.sequence.to_be_bytes(),
        ),
        &SequenceInfo {
            outgoing_sequence_expected: op_code & FLAG_OUTGOING_TRANSFER != 0,
            outgoing_sequence: None,
        },
    )?;

    let mut response = Response::new();
    // borrow ops are routed through an address proxy contract
    // make sure its initialized; if its already initialized this is a no-op
    if ANCHOR_BORROW_OPS.contains(&op_code) {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.cross_anchor_core_addr.to_string(),
            msg: to_binary(&core::ExecuteMsg::InitializeAddressProxy {
                chain_id: sender_chain,
                address: sender_address.clone(),
            })?,
            funds: vec![],
        }));
    }

    // ensure the incoming asset has been properly sent
    if op_code & FLAG_INCOMING_TRANSFER != 0 {
        let expected_sequence = parsed_instruction_vaa
            .payload
            .as_slice()
            .get_u64(OP_SPECIFIC_PAYLOAD_INDEX);
        let token_transfer_vaa = option_token_transfer_vaa.unwrap();
        let asset = process_token_transfer_message(
            deps.as_ref(),
            env,
            &config,
            sender_chain,
            expected_sequence,
            &token_transfer_vaa,
        )?;
        // attempt to complete the transfer; if it has already been completed, no-op
        response = response.add_submessage(SubMsg {
            id: TOKEN_TRANSFER_SUBMIT_VAA_MSG_ID,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.wormhole_token_bridge_addr.to_string(),
                funds: vec![],
                msg: to_binary(&WormholeTokenBridgeExecuteMsg::SubmitVaa {
                    data: token_transfer_vaa,
                })?,
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Error,
        });

        // forward appropriate message to core
        if op_code == OP_CODE_DEPOSIT_STABLE {
            response = response.add_message(create_core_execute_message(
                &config,
                core::ExecuteMsg::DepositStable {
                    sender_chain,
                    sender_address,
                },
                vec![asset.deduct_tax(&deps.querier)?],
            ));
        } else if op_code == OP_CODE_REPAY_STABLE {
            response = response.add_message(create_core_execute_message(
                &config,
                core::ExecuteMsg::RepayStable {
                    sender_chain,
                    sender_address,
                },
                vec![asset.deduct_tax(&deps.querier)?],
            ));
        } else if let AssetInfo::Token { contract_addr } = asset.info {
            let cw20_hook_msg = if op_code == OP_CODE_REDEEM_STABLE {
                core::Cw20HookMsg::RedeemStable {
                    sender_chain,
                    sender_address,
                }
            } else {
                core::Cw20HookMsg::LockCollateral {
                    sender_chain,
                    sender_address,
                }
            };
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                    contract: config.cross_anchor_core_addr.to_string(),
                    amount: asset.amount,
                    msg: to_binary(&cw20_hook_msg)?,
                })?,
                funds: vec![],
            }));
        } else {
            return Err(StdError::generic_err(
                "expecting a Terra cw20 token but received Terra native token",
            ));
        }
    } else {
        match op_code {
            OP_CODE_UNLOCK_COLLATERAL => {
                response = response.add_message(create_core_execute_message(
                    &config,
                    core::ExecuteMsg::UnlockCollateral {
                        sender_chain,
                        sender_address,
                        asset: Asset {
                            info: AssetInfo::Token {
                                contract_addr: deps
                                    .api
                                    .addr_humanize(
                                        &parsed_instruction_vaa
                                            .payload
                                            .as_slice()
                                            .get_address(OP_SPECIFIC_PAYLOAD_INDEX),
                                    )?
                                    .to_string(),
                            },
                            amount: Uint128::from(
                                parsed_instruction_vaa
                                    .payload
                                    .as_slice()
                                    .get_u128_be(OP_SPECIFIC_PAYLOAD_INDEX + ADDRESS_LEN),
                            ),
                        },
                    },
                    vec![],
                ));
            }
            OP_CODE_BORROW_STABLE => {
                response = response.add_message(create_core_execute_message(
                    &config,
                    core::ExecuteMsg::BorrowStable {
                        sender_chain,
                        sender_address,
                        borrow_amount: parsed_instruction_vaa
                            .payload
                            .as_slice()
                            .get_uint256(OP_SPECIFIC_PAYLOAD_INDEX),
                    },
                    vec![],
                ));
            }
            OP_CODE_CLAIM_REWARDS => {
                response = response.add_message(create_core_execute_message(
                    &config,
                    core::ExecuteMsg::ClaimRewards {
                        sender_chain,
                        sender_address,
                    },
                    vec![],
                ));
            }
            _ => unreachable!(),
        }
    }

    Ok(response)
}

fn process_token_transfer_message(
    deps: Deps,
    env: Env,
    config: &Config,
    expected_emitter_chain: u16,
    expected_sequence: u64,
    token_transfer_vaa: &Binary,
) -> StdResult<Asset> {
    let parsed_token_transfer_vaa = get_parsed_vaa(deps, &env, config, token_transfer_vaa)?;
    if expected_emitter_chain != parsed_token_transfer_vaa.emitter_chain {
        return Err(StdError::generic_err(
            "unexpected token transfer emitter chain",
        ));
    }

    // NOTE: no need to validate the emitter address; this is automatically done
    // when trying to complete the transfer
    if expected_sequence != parsed_token_transfer_vaa.sequence {
        return Err(StdError::generic_err("unexpected token transfer sequence"));
    }
    let token_bridge_message = TokenBridgeMessage::deserialize(&parsed_token_transfer_vaa.payload)?;
    if token_bridge_message.action != Action::TRANSFER {
        return Err(StdError::generic_err("unexpected token transfer action"));
    }
    let transfer_info = TransferInfo::deserialize(&token_bridge_message.payload)?;
    if transfer_info.recipient_chain != TERRA_CHAIN_ID
        || transfer_info.recipient
            != extend_terra_address_to_32(
                &deps.api.addr_canonicalize(env.contract.address.as_str())?,
            )
    {
        return Err(StdError::generic_err("unexpected token transfer recipient"));
    }
    parse_token_transfer_asset(deps, transfer_info)
}

fn parse_token_transfer_asset(deps: Deps, transfer_info: TransferInfo) -> StdResult<Asset> {
    if transfer_info.token_chain != TERRA_CHAIN_ID {
        return Err(StdError::generic_err(
            "transferred token is not a Terra token",
        ));
    }
    let (_, mut amount) = transfer_info.amount;
    let (_, fee) = transfer_info.fee;
    amount = amount.checked_sub(fee).unwrap();

    static WORMHOLE_TERRA_NATIVE_TOKEN_INDICATOR: u8 = 1;
    let asset =
        if transfer_info.token_address.as_slice()[0] == WORMHOLE_TERRA_NATIVE_TOKEN_INDICATOR {
            let mut token_address = transfer_info.token_address;
            let token_address = token_address.as_mut_slice();
            token_address[0] = 0;
            let mut denom = token_address.to_vec();
            denom.retain(|&c| c != 0);
            let mut asset = Asset {
                info: AssetInfo::NativeToken {
                    denom: String::from_utf8(denom)?,
                },
                amount: Uint128::from(amount),
            };
            // This accounts for tax deducted for the transfer from the token bridge to this Anchor bridge contract.
            asset.amount = asset.deduct_tax(&deps.querier)?.amount;
            asset
        } else {
            Asset {
                info: AssetInfo::Token {
                    contract_addr: deps
                        .api
                        .addr_humanize(&transfer_info.token_address.as_slice().get_address(0))?
                        .to_string(),
                },
                amount: Uint128::from(amount),
            }
        };
    Ok(asset)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id != TOKEN_TRANSFER_SUBMIT_VAA_MSG_ID {
        return Err(StdError::generic_err("unexpected reply id"));
    }
    if let ContractResult::Err(err) = msg.result {
        if err == "Generic error: VaaAlreadyExecuted: execute wasm contract failed" {
            Ok(Response::default())
        } else {
            Err(StdError::generic_err(err))
        }
    } else {
        Err(StdError::generic_err("unexpected success reply msg"))
    }
}

// crossanchor-core hits this method when it wants to relay assets back over the bridge
fn send_asset(deps: DepsMut, env: Env, info: MessageInfo, mut asset: Asset) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.as_str() != config.cross_anchor_core_addr.as_str() {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut response = Response::new();
    if asset.is_native_token() {
        let coin_after_tax = asset.deduct_tax(&deps.querier)?;
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.wormhole_token_bridge_addr.to_string(),
            msg: to_binary(&WormholeTokenBridgeExecuteMsg::DepositTokens {})?,
            funds: vec![coin_after_tax.clone()],
        }));
        asset.amount = coin_after_tax.amount;
    } else if let AssetInfo::Token { contract_addr } = asset.clone().info {
        response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.wormhole_token_bridge_addr.to_string(),
                amount: asset.amount,
                expires: None,
            })?,
            funds: vec![],
        }))
    }

    const TOKEN_TRANSFER_NONCE: u32 = 135792468u32;
    const TOKEN_TRANSFER_INFO_NONCE: u32 = 24680135u32;
    let outgoing_token_transfer_info = TMP_OUTGOING_TOKEN_TRANSFER_INFO.load(deps.storage)?;
    let cross_bridge_address = CHAIN_ID_TO_ANCHOR_BRIDGE_ADDRESS_MAP.load(
        deps.storage,
        U16Key::from(outgoing_token_transfer_info.chain_id),
    )?;

    let outgoing_sequence = get_next_sequence(deps.as_ref(), &config, &env.contract.address)?;

    // record sequence number of outgoing instruction
    SEQUENCE_STORE.save(
        deps.storage,
        (
            &outgoing_token_transfer_info.chain_id.to_be_bytes(),
            &outgoing_token_transfer_info
                .instruction_sequence
                .to_be_bytes(),
        ),
        &SequenceInfo {
            outgoing_sequence_expected: true,
            outgoing_sequence: Some(outgoing_sequence),
        },
    )?;

    response = response.add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.wormhole_token_bridge_addr.to_string(),
            msg: to_binary(&WormholeTokenBridgeExecuteMsg::InitiateTransfer {
                asset,
                recipient_chain: outgoing_token_transfer_info.chain_id,
                // transfer to the appropriate bridge on the other side
                recipient: cross_bridge_address.as_slice().into(),
                fee: Uint128::zero(),
                nonce: TOKEN_TRANSFER_NONCE,
            })?,
            funds: vec![],
        }),
        // associated generic message provides info about who can claim it from the bridge
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.wormhole_core_bridge_addr.to_string(),
            msg: to_binary(&WormholeCoreBridgeExecuteMsg::PostMessage {
                message: Binary::from(outgoing_token_transfer_info.serialize()),
                nonce: TOKEN_TRANSFER_INFO_NONCE,
            })?,
            funds: vec![],
        }),
    ]);

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
