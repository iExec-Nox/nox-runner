/// Strip the 0x prefix from a hex string if present.
pub fn strip_0x_prefix(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}
