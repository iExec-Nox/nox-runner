//! Off-chain computations producing results compatible with Solidity.

use alloy_primitives::hex;
use tracing::error;

pub mod arithmetic;

/// Extracts solidity type from handle hex value
pub fn get_solidity_type_from_handle(handle: &str) -> Result<u8, String> {
    match hex::decode(handle) {
        Ok(v) => Ok(v[30]),
        Err(e) => Err(format!("Failed to decode handle hex value {e}")),
    }
}

/// Gets size in bytes of a given solidity type encoded as a byte
pub fn get_solidity_type_size(solidity_type: u8) -> Result<usize, String> {
    let solidity_type_size = match solidity_type {
        0 => 1,
        1 => 20,
        2..4 => 32,
        v @ 4..36 => v - 3,
        v @ 36..68 => v - 35,
        v @ 68..100 => v - 67,
        v => {
            let message = format!("Unsupported TEE type for encryption ({v})");
            error!(message);
            return Err(message);
        }
    };
    Ok(solidity_type_size as usize)
}
