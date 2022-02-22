use crate::state::WHITELISTED_BRIDGES;
use cosmwasm_std::{StdError, StdResult, Storage};

pub fn check_whitelisted(storage: &dyn Storage, sender: &str) -> StdResult<()> {
    match WHITELISTED_BRIDGES.load(storage, sender.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(StdError::generic_err("sender is not whitelisted bridge")),
    }
}
