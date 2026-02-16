//! Boolean operations support.

use super::SolidityValue;

/// Supported boolean operators.
pub enum Operator {
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

/// Implements boolean comparisons on 16 or 256 bits signed or unsigned integers.
pub fn compare(
    operation: Operator,
    left_hand_operand: SolidityValue,
    right_hand_operand: SolidityValue,
) -> Result<bool, String> {
    match (operation, left_hand_operand, right_hand_operand) {
        (Operator::Eq, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a == b),
        (Operator::Eq, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a == b),
        (Operator::Eq, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a == b),
        (Operator::Eq, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a == b),
        (Operator::Ne, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a != b),
        (Operator::Ne, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a != b),
        (Operator::Ne, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a != b),
        (Operator::Ne, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a != b),
        (Operator::Ge, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a >= b),
        (Operator::Ge, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a >= b),
        (Operator::Ge, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a >= b),
        (Operator::Ge, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a >= b),
        (Operator::Gt, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a > b),
        (Operator::Gt, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a > b),
        (Operator::Gt, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a > b),
        (Operator::Gt, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a > b),
        (Operator::Le, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a <= b),
        (Operator::Le, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a <= b),
        (Operator::Le, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a <= b),
        (Operator::Le, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a <= b),
        (Operator::Lt, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => Ok(a < b),
        (Operator::Lt, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => Ok(a < b),
        (Operator::Lt, SolidityValue::Int16(a), SolidityValue::Int16(b)) => Ok(a < b),
        (Operator::Lt, SolidityValue::Int256(a), SolidityValue::Int256(b)) => Ok(a < b),
        _ => Err("Unsupported operation".to_string()),
    }
}

/// Returns if_true or if_false depending on condition boolean value.
///
/// Returns an error if condition is not a valid boolean value.
pub fn select(
    condition: SolidityValue,
    if_true: SolidityValue,
    if_false: SolidityValue,
) -> Result<SolidityValue, String> {
    match condition {
        SolidityValue::Boolean(condition) => {
            if condition {
                Ok(if_true)
            } else {
                Ok(if_false)
            }
        }
        _ => Err(
            "Unsupported operation, condition does not represent a solidity boolean".to_string(),
        ),
    }
}
