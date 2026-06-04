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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Signed, Uint};

    #[test]
    fn compare_returns_true_when_uint16_values_are_equal() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(42_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(42_u16));
        assert!(compare(Operator::Eq, a, b).unwrap());
    }

    #[test]
    fn compare_returns_false_when_uint16_values_differ() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(20_u16));
        assert!(!compare(Operator::Eq, a, b).unwrap());
    }

    #[test]
    fn compare_returns_true_for_gt_when_left_is_greater() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(20_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        assert!(compare(Operator::Gt, a, b).unwrap());
    }

    #[test]
    fn compare_returns_false_for_gt_when_values_are_equal() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        assert!(!compare(Operator::Gt, a, b).unwrap());
    }

    #[test]
    fn compare_returns_true_for_ge_when_values_are_equal() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        assert!(compare(Operator::Ge, a, b).unwrap());
    }

    #[test]
    fn compare_returns_true_for_lt_when_left_is_less() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(5_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        assert!(compare(Operator::Lt, a, b).unwrap());
    }

    #[test]
    fn compare_returns_true_for_le_when_values_are_equal() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(10_u16));
        assert!(compare(Operator::Le, a, b).unwrap());
    }

    #[test]
    fn compare_returns_true_for_lt_when_int16_left_is_negative() {
        use std::str::FromStr;
        let a = SolidityValue::Int16(Signed::<16, 1>::from_str("-5").unwrap());
        let b = SolidityValue::Int16(Signed::<16, 1>::from_str("5").unwrap());
        assert!(compare(Operator::Lt, a, b).unwrap());
    }

    #[test]
    fn compare_returns_error_when_types_mismatch() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let b = SolidityValue::Uint256(Uint::<256, 4>::from(1_u64));
        let result = compare(Operator::Eq, a, b);
        assert!(result.is_err());
    }

    #[test]
    fn select_returns_if_true_when_condition_is_true() {
        let cond = SolidityValue::Boolean(true);
        let yes = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let no = SolidityValue::Uint16(Uint::<16, 1>::from(0_u16));
        let result = select(cond, yes.clone(), no).unwrap();
        assert_eq!(result, yes);
    }

    #[test]
    fn select_returns_if_false_when_condition_is_false() {
        let cond = SolidityValue::Boolean(false);
        let yes = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let no = SolidityValue::Uint16(Uint::<16, 1>::from(0_u16));
        let result = select(cond, yes, no.clone()).unwrap();
        assert_eq!(result, no);
    }

    #[test]
    fn select_returns_error_when_condition_is_not_boolean() {
        let cond = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let yes = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let no = SolidityValue::Uint16(Uint::<16, 1>::from(0_u16));
        let result = select(cond, yes, no);
        assert!(result.is_err());
    }
}
