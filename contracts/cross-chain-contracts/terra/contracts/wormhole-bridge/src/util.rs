use std::collections::HashSet;

use cosmwasm_std::{Addr, Deps, StdResult};
use crossanchor::byte_utils::extend_terra_address_to_32;
use cw_storage_plus::Map;

use crate::state::Config;

/// See README for details on Opcode specification

pub const FLAG_INCOMING_TRANSFER: u8 = 0b1000_0000;
pub const FLAG_OUTGOING_TRANSFER: u8 = 0b0100_0000;
pub const FLAG_BOTH_TRANSFERS: u8 = 0b1100_0000;

#[allow(dead_code)]
pub const FLAG_NO_ASSC_TRANSFER: u8 = 0b0000_0000;

//Operations that have both incoming and outgoing transfers
pub const OP_CODE_DEPOSIT_STABLE: u8 = 0 | FLAG_BOTH_TRANSFERS; // was 0
pub const OP_CODE_REDEEM_STABLE: u8 = 1 | FLAG_BOTH_TRANSFERS; // was 5

//Operations that have only incoming transfers
pub const OP_CODE_REPAY_STABLE: u8 = 0 | FLAG_INCOMING_TRANSFER; // was 1
pub const OP_CODE_LOCK_COLLATERAL: u8 = 1 | FLAG_INCOMING_TRANSFER; // was 6

//Operations that have only outgoing transfers
pub const OP_CODE_UNLOCK_COLLATERAL: u8 = 0 | FLAG_OUTGOING_TRANSFER; // was 2
pub const OP_CODE_BORROW_STABLE: u8 = 1 | FLAG_OUTGOING_TRANSFER; // was 3
pub const OP_CODE_CLAIM_REWARDS: u8 = 2 | FLAG_OUTGOING_TRANSFER; // was 4

lazy_static! {
    // List of Anchor Borrow operations; these operations require a lazy initialization of Address Proxy.
    pub static ref ANCHOR_BORROW_OPS: HashSet<u8> = {
        let mut s = HashSet::new();
        s.insert(OP_CODE_REPAY_STABLE);
        s.insert(OP_CODE_UNLOCK_COLLATERAL);
        s.insert(OP_CODE_BORROW_STABLE);
        s.insert(OP_CODE_CLAIM_REWARDS);
        s.insert(OP_CODE_LOCK_COLLATERAL);
        s
    };
}

// Returns the sequence of the next message sent by some emitter address
pub fn get_next_sequence(
    deps: Deps,
    config: &Config,
    emitter_address: &Addr
) -> StdResult<u64> {
    // This map is only meant to be used to query Wormhole core bridge for the token bridge's sequence number in a type-safe way.
    // No data will be stored through this map interface in the storage of this Anchor bridge contract.
    // See https://docs.rs/cw-storage-plus/latest/cw_storage_plus/struct.Map.html#method.query for details.
    const WORMHOLE_SEQUENCE_MAP: Map<&[u8], u64> = Map::new("sequence");
    let emitter_key = extend_terra_address_to_32(
        &deps
            .api
            .addr_canonicalize(emitter_address.as_str())?,
    );
    match WORMHOLE_SEQUENCE_MAP.query(
        &deps.querier,
        config.wormhole_core_bridge_addr.clone(),
        &emitter_key,
    ) {
        Ok(option_sequence) => Ok(option_sequence.unwrap_or(0u64)),
        Err(err) => Err(err),
    }
}
