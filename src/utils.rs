use alloy_primitives::hex;

/// Strip the 0x prefix from a hex string if present.
pub fn strip_0x_prefix(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

/// Serialize bytes to hex string with prefix
pub fn to_hex_with_prefix(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}
