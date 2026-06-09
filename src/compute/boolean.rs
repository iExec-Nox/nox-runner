//! Boolean operations support.

use super::SolidityValue;

/// Supported boolean operators.
#[derive(Clone, Copy)]
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
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::{Signed, Uint};

    #[test]
    fn compare_returns_correct_result_for_all_operator_and_type_combinations() {
        let u16 = |n: u16| SolidityValue::Uint16(Uint::<16, 1>::from(n));
        let u256 = |n: u64| SolidityValue::Uint256(Uint::<256, 4>::from(n));
        let i16 = |s: &str| SolidityValue::Int16(Signed::<16, 1>::from_str(s).unwrap());
        let i256 = |s: &str| SolidityValue::Int256(Signed::<256, 4>::from_str(s).unwrap());

        let cases: &[(Operator, SolidityValue, SolidityValue, bool)] = &[
            // --- Uint16 ---
            (Operator::Eq, u16(42), u16(42), true),
            (Operator::Eq, u16(10), u16(20), false),
            (Operator::Ne, u16(10), u16(20), true),
            (Operator::Ne, u16(42), u16(42), false),
            (Operator::Gt, u16(20), u16(10), true),
            (Operator::Gt, u16(10), u16(10), false),
            (Operator::Ge, u16(10), u16(10), true),
            (Operator::Lt, u16(5), u16(10), true),
            (Operator::Le, u16(10), u16(10), true),
            // --- Int16 ---
            (Operator::Eq, i16("5"), i16("5"), true),
            (Operator::Ne, i16("-1"), i16("1"), true),
            (Operator::Ge, i16("5"), i16("5"), true),
            (Operator::Gt, i16("10"), i16("5"), true),
            (Operator::Lt, i16("-5"), i16("5"), true),
            (Operator::Le, i16("5"), i16("5"), true),
            // --- Uint256 ---
            (Operator::Eq, u256(1_000_000), u256(1_000_000), true),
            (Operator::Ne, u256(1), u256(2), true),
            (Operator::Ge, u256(100), u256(100), true),
            (Operator::Gt, u256(200), u256(100), true),
            (Operator::Lt, u256(100), u256(200), true),
            (Operator::Le, u256(50), u256(100), true),
            // --- Int256 ---
            (Operator::Eq, i256("-42"), i256("-42"), true),
            (Operator::Ne, i256("-1"), i256("1"), true),
            (Operator::Ge, i256("10"), i256("10"), true),
            (Operator::Gt, i256("100"), i256("50"), true),
            (Operator::Lt, i256("-1000"), i256("1000"), true),
            (Operator::Le, i256("-5"), i256("0"), true),
        ];
        for (i, (op, a, b, expected)) in cases.iter().enumerate() {
            assert_eq!(
                compare(*op, a.clone(), b.clone()).unwrap(),
                *expected,
                "case {i}"
            );
        }
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
