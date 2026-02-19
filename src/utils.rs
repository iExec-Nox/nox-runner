use alloy_primitives::hex;

/// Serialize bytes to hex string with prefix
pub fn to_hex_with_prefix(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}
