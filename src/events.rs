//! Structs to deserialize received [`TransactionMessage`]s.

use alloy_primitives::Address;
use serde::Deserialize;

/// Handle type for encrypted values (hex-encoded bytes32)
pub type Handle = String;

/// Encryption operation (plaintext to encrypted)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionOperation {
    pub value: String,
    pub tee_type: u8,
    pub handle: Handle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArithmeticOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub result: Handle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeArithmeticOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub success: Handle,
    pub result: Handle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub result: Handle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectOperation {
    pub condition: Handle,
    pub if_true: Handle,
    pub if_false: Handle,
    pub result: Handle,
}

/// Event payload with typed variants
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Operator {
    PlaintextToEncrypted(EncryptionOperation),
    Add(ArithmeticOperation),
    Sub(ArithmeticOperation),
    Mul(ArithmeticOperation),
    Div(ArithmeticOperation),
    SafeAdd(SafeArithmeticOperation),
    SafeSub(SafeArithmeticOperation),
    SafeMul(SafeArithmeticOperation),
    SafeDiv(SafeArithmeticOperation),
    Eq(BooleanOperation),
    Ne(BooleanOperation),
    Ge(BooleanOperation),
    Gt(BooleanOperation),
    Le(BooleanOperation),
    Lt(BooleanOperation),
    Select(SelectOperation),
}

/// Individual event within a transaction
#[derive(Debug, Deserialize)]
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
