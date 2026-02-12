use alloy_primitives::Address;
use serde::Deserialize;

/// Handle type for encrypted values (hex-encoded bytes32)
pub type Handle = String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArithmeticOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub result: Handle,
}

/// Encryption operation (plaintext to encrypted)
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionOperation {
    pub value: String,
    pub tee_type: u8,
    pub handle: Handle,
}

/// Event payload with typed variants
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Operator {
    PlaintextToEncrypted(EncryptionOperation),
    Add(ArithmeticOperation),
    Sub(ArithmeticOperation),
    Div(ArithmeticOperation),
}

/// Individual event within a transaction
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEvent {
    pub log_index: u64,
    pub caller: Address,
    #[serde(flatten)]
    pub operator: Operator,
}

/// Message format grouping events by transaction
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    /// Chain ID where the events occurred
    pub chain_id: u32,
    /// Block number
    pub block_number: u64,
    /// Caller address
    pub caller: Address,
    /// Transaction hash
    pub transaction_hash: String,
    /// Events in this transaction, ordered by log_index
    pub events: Vec<TransactionEvent>,
}
