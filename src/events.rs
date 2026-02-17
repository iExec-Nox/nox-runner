//! Structs to deserialize received [`TransactionMessage`]s.

use alloy_primitives::Address;
use serde::Deserialize;

/// Handle type for encrypted values (hex-encoded bytes32)
pub type Handle = String;

/// Describes the 2 plaintext operands to encrypt and associat to the result handle.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionOperation {
    pub value: String,
    pub tee_type: u8,
    pub handle: Handle,
}

/// Describes the 2 operand and 1 result handles for an arithmetic operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArithmeticOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub result: Handle,
}

/// Describes the 2 operand and 2 result handles for a safe arithmetic operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeArithmeticOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub success: Handle,
    pub result: Handle,
}

/// Describes the 3 operand and 1 result handles for a boolean comparison.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanOperation {
    pub left_hand_operand: Handle,
    pub right_hand_operand: Handle,
    pub result: Handle,
}

/// Describes the 3 operand and 1 result handles for a select operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectOperation {
    pub condition: Handle,
    pub if_true: Handle,
    pub if_false: Handle,
    pub result: Handle,
}

/// Describes the 3 operand and 3 result handles for a transfer operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOperation {
    pub balance_from: Handle,
    pub balance_to: Handle,
    pub amount: Handle,
    pub success: Handle,
    pub new_balance_from: Handle,
    pub new_balance_to: Handle,
}

/// Describes the 3 operand and 3 result handles for a mint operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintOperation {
    pub balance_to: Handle,
    pub amount: Handle,
    pub total_supply: Handle,
    pub success: Handle,
    pub new_balance_to: Handle,
    pub new_total_supply: Handle,
}

/// Describes the 3 operand and 3 result handles for a burn operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnOperation {
    pub balance_from: Handle,
    pub amount: Handle,
    pub total_supply: Handle,
    pub success: Handle,
    pub new_balance_from: Handle,
    pub new_total_supply: Handle,
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
    Transfer(TransferOperation),
    Mint(MintOperation),
    Burn(BurnOperation),
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
