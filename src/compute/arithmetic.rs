//! Arithmetic operations support.

use super::SolidityValue;
use alloy::primitives::{Signed, Uint};

/// Supported arithmetic operators.
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

/// Performs Add, Sub, Mul or Div arithmetic operations on 16 or 256 bits signed or unsigned integers.
///
/// Add, Sub and Mul operations wrap around at the boundary of each type.
pub fn compute(
    operation: Operator,
    left_hand_operand: SolidityValue,
    right_hand_operand: SolidityValue,
) -> Result<SolidityValue, String> {
    match (operation, left_hand_operand, right_hand_operand) {
        (Operator::Add, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            Ok(SolidityValue::Uint16(a.wrapping_add(b)))
        }
        (Operator::Add, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            Ok(SolidityValue::Uint256(a.wrapping_add(b)))
        }
        (Operator::Add, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            Ok(SolidityValue::Int16(a.wrapping_add(b)))
        }
        (Operator::Add, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            Ok(SolidityValue::Int256(a.wrapping_add(b)))
        }
        (Operator::Sub, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            Ok(SolidityValue::Uint16(a.wrapping_sub(b)))
        }
        (Operator::Sub, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            Ok(SolidityValue::Uint256(a.wrapping_sub(b)))
        }
        (Operator::Sub, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            Ok(SolidityValue::Int16(a.wrapping_sub(b)))
        }
        (Operator::Sub, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            Ok(SolidityValue::Int256(a.wrapping_sub(b)))
        }
        (Operator::Mul, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            Ok(SolidityValue::Uint16(a.wrapping_mul(b)))
        }
        (Operator::Mul, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            Ok(SolidityValue::Uint256(a.wrapping_mul(b)))
        }
        (Operator::Mul, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            Ok(SolidityValue::Int16(a.wrapping_mul(b)))
        }
        (Operator::Mul, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            Ok(SolidityValue::Int256(a.wrapping_mul(b)))
        }
        (Operator::Div, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            if b != Uint::<16, 1>::ZERO {
                Ok(SolidityValue::Uint16(a / b))
            } else {
                Ok(SolidityValue::Uint16(Uint::<16, 1>::MAX))
            }
        }
        (Operator::Div, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            if b != Uint::<256, 4>::ZERO {
                Ok(SolidityValue::Uint256(a / b))
            } else {
                Ok(SolidityValue::Uint256(Uint::<256, 4>::MAX))
            }
        }
        (Operator::Div, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            if b != Signed::<16, 1>::ZERO {
                Ok(SolidityValue::Int16(a / b))
            } else {
                Ok(SolidityValue::Int16(Signed::<16, 1>::MAX))
            }
        }
        (Operator::Div, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            if b != Signed::<256, 4>::ZERO {
                Ok(SolidityValue::Int256(a / b))
            } else {
                Ok(SolidityValue::Int256(Signed::<256, 4>::MAX))
            }
        }
        _ => Err("Unsupported operation".to_string()),
    }
}

/// Performs checked Add, Sub, Mul or Div arithmetic operations on 16 or 256 bits signed or unsiged integers.
///
/// On overflow, the method will mostly return the (false, ZERO) tuple.
/// If a result can be computed without overflowing, the method will return a (true, result) tuple.
/// The second member of the returned tuple will be a valid SolidityValue.
pub fn safe_compute(
    operation: Operator,
    left_hand_operand: SolidityValue,
    right_hand_operand: SolidityValue,
) -> Result<(bool, SolidityValue), String> {
    match (operation, left_hand_operand, right_hand_operand) {
        (Operator::Add, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            let (success, result) = match a.checked_add(b) {
                Some(value) => (true, value),
                None => (false, Uint::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Uint16(result)))
        }
        (Operator::Add, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            let (success, result) = match a.checked_add(b) {
                Some(value) => (true, value),
                None => (false, Uint::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Uint256(result)))
        }
        (Operator::Add, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            let (success, result) = match a.checked_add(b) {
                Some(value) => (true, value),
                None => (false, Signed::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Int16(result)))
        }
        (Operator::Add, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            let (success, result) = match a.checked_add(b) {
                Some(value) => (true, value),
                None => (false, Signed::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Int256(result)))
        }
        (Operator::Sub, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            let (success, result) = match a.checked_sub(b) {
                Some(value) => (true, value),
                None => (false, Uint::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Uint16(result)))
        }
        (Operator::Sub, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            let (success, result) = match a.checked_sub(b) {
                Some(value) => (true, value),
                None => (false, Uint::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Uint256(result)))
        }
        (Operator::Sub, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            let (success, result) = match a.checked_sub(b) {
                Some(value) => (true, value),
                None => (false, Signed::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Int16(result)))
        }
        (Operator::Sub, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            let (success, result) = match a.checked_sub(b) {
                Some(value) => (true, value),
                None => (false, Signed::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Int256(result)))
        }
        (Operator::Mul, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            let (success, result) = match a.checked_mul(b) {
                Some(value) => (true, value),
                None => (false, Uint::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Uint16(result)))
        }
        (Operator::Mul, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            let (success, result) = match a.checked_mul(b) {
                Some(value) => (true, value),
                None => (false, Uint::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Uint256(result)))
        }
        (Operator::Mul, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            let (success, result) = match a.checked_mul(b) {
                Some(value) => (true, value),
                None => (false, Signed::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Int16(result)))
        }
        (Operator::Mul, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            let (success, result) = match a.checked_mul(b) {
                Some(value) => (true, value),
                None => (false, Signed::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Int256(result)))
        }
        (Operator::Div, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            let (success, result) = match a.checked_div(b) {
                Some(value) => (true, value),
                None => (false, Uint::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Uint16(result)))
        }
        (Operator::Div, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            let (success, result) = match a.checked_div(b) {
                Some(value) => (true, value),
                None => (false, Uint::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Uint256(result)))
        }
        (Operator::Div, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            let (success, result) = match a.checked_div(b) {
                Some(value) => (true, value),
                None => (false, Signed::<16, 1>::ZERO),
            };
            Ok((success, SolidityValue::Int16(result)))
        }
        (Operator::Div, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            let (success, result) = match a.checked_div(b) {
                Some(value) => (true, value),
                None => (false, Signed::<256, 4>::ZERO),
            };
            Ok((success, SolidityValue::Int256(result)))
        }
        _ => Err("Unsupported operation".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::hex;

    fn hex_decode(hex_str: &str) -> [u8; 32] {
        let trimmed = hex_str.trim_start_matches("0x");
        if 64 < trimmed.len() {
            return [0u8; 32];
        }
        let mut result = [0u8; 32];
        let bytes = hex::decode(hex_str).unwrap();
        result[32 - bytes.len()..32].copy_from_slice(&bytes);
        result
    }

    #[test]
    fn compute_succeeds_for_uint16_add_sub_and_div() {
        let left_hand_value = hex_decode("0x0200");
        let right_hand_value = hex_decode("0x0100");
        let left_hand_operand = SolidityValue::from_bytes(5_u8, left_hand_value).unwrap();
        let right_hand_operand = SolidityValue::from_bytes(5_u8, right_hand_value).unwrap();
        let add_result = compute(
            Operator::Add,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let div_result = compute(
            Operator::Div,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        assert_eq!(
            add_result,
            SolidityValue::Uint16(Uint::<16, 1>::from(768_u16))
        );
        assert_eq!(
            sub_result,
            SolidityValue::Uint16(Uint::<16, 1>::from(256_u16))
        );
        assert_eq!(
            div_result,
            SolidityValue::Uint16(Uint::<16, 1>::from(2_u16))
        );
    }

    #[test]
    fn compute_wraps_for_uint256_when_add_and_sub_overflow() {
        let left_hand_value =
            hex_decode("0x8000000000000000000000000000000000000000000000000000000000000000");
        let right_hand_value =
            hex_decode("0x8000000000000000000000000000000000000000000000000000000000000001");
        let left_hand_operand = SolidityValue::from_bytes(35_u8, left_hand_value).unwrap();
        let right_hand_operand = SolidityValue::from_bytes(35_u8, right_hand_value).unwrap();
        let add_result = compute(
            Operator::Add,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let div_result = compute(
            Operator::Div,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        assert_eq!(
            add_result,
            SolidityValue::Uint256(Uint::<256, 4>::from_str("1").unwrap())
        );
        assert_eq!(
            sub_result,
            SolidityValue::Uint256(
                Uint::<256, 4>::from_str(
                    "115792089237316195423570985008687907853269984665640564039457584007913129639935"
                )
                .unwrap()
            )
        );
        assert_eq!(
            div_result,
            SolidityValue::Uint256(Uint::<256, 4>::from_str("0").unwrap())
        );
    }

    #[test]
    fn compute_succeeds_for_int16_add_sub_and_div() {
        let left_hand_value = hex_decode("0xFE00");
        let right_hand_value = hex_decode("0x0100");
        let left_hand_operand = SolidityValue::from_bytes(37_u8, left_hand_value)
            .expect("should convert left hand value");
        let right_hand_operand = SolidityValue::from_bytes(37_u8, right_hand_value)
            .expect("should convert right hand value");
        let add_result = compute(
            Operator::Add,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        let div_result = compute(
            Operator::Div,
            left_hand_operand.clone(),
            right_hand_operand.clone(),
        )
        .unwrap();
        assert_eq!(
            add_result,
            SolidityValue::Int16(Signed::<16, 1>::from_str("-256").unwrap())
        );
        assert_eq!(
            sub_result,
            SolidityValue::Int16(Signed::<16, 1>::from_str("-768").unwrap())
        );
        assert_eq!(
            div_result,
            SolidityValue::Int16(Signed::<16, 1>::from_str("-2").unwrap())
        );
    }

    #[test]
    fn compute_returns_product_for_uint16_mul() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(3_u16));
        let b = SolidityValue::Uint16(Uint::<16, 1>::from(4_u16));
        let result = compute(Operator::Mul, a, b).unwrap();
        assert_eq!(result, SolidityValue::Uint16(Uint::<16, 1>::from(12_u16)));
    }

    #[test]
    fn compute_returns_product_for_uint256_mul() {
        let a = SolidityValue::Uint256(Uint::<256, 4>::from(100_u64));
        let b = SolidityValue::Uint256(Uint::<256, 4>::from(200_u64));
        let result = compute(Operator::Mul, a, b).unwrap();
        assert_eq!(
            result,
            SolidityValue::Uint256(Uint::<256, 4>::from(20_000_u64))
        );
    }

    #[test]
    fn compute_returns_max_when_div_by_zero_for_all_types() {
        let cases = [
            (
                SolidityValue::Uint16(Uint::<16, 1>::from(10_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
                SolidityValue::Uint16(Uint::<16, 1>::MAX),
            ),
            (
                SolidityValue::Int16(Signed::<16, 1>::from_str("100").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::ZERO),
                SolidityValue::Int16(Signed::<16, 1>::MAX),
            ),
            (
                SolidityValue::Uint256(Uint::<256, 4>::from(10_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::ZERO),
                SolidityValue::Uint256(Uint::<256, 4>::MAX),
            ),
            (
                SolidityValue::Int256(Signed::<256, 4>::from_str("100").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
                SolidityValue::Int256(Signed::<256, 4>::MAX),
            ),
        ];
        for (i, (a, b, expected)) in cases.into_iter().enumerate() {
            assert_eq!(compute(Operator::Div, a, b).unwrap(), expected, "case {i}");
        }
    }

    #[test]
    fn safe_compute_returns_true_and_result_when_no_overflow() {
        let cases = [
            (
                Operator::Add,
                SolidityValue::Uint16(Uint::<16, 1>::from(1_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(2_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(3_u16)),
            ),
            (
                Operator::Add,
                SolidityValue::Uint256(Uint::<256, 4>::from(10_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(20_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(30_u64)),
            ),
            (
                Operator::Add,
                SolidityValue::Int16(Signed::<16, 1>::from_str("5").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("3").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("8").unwrap()),
            ),
            (
                Operator::Add,
                SolidityValue::Int256(Signed::<256, 4>::from_str("10").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("20").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("30").unwrap()),
            ),
            (
                Operator::Sub,
                SolidityValue::Uint16(Uint::<16, 1>::from(10_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(3_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(7_u16)),
            ),
            (
                Operator::Sub,
                SolidityValue::Uint256(Uint::<256, 4>::from(100_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(30_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(70_u64)),
            ),
            (
                Operator::Sub,
                SolidityValue::Int16(Signed::<16, 1>::from_str("10").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("3").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("7").unwrap()),
            ),
            (
                Operator::Sub,
                SolidityValue::Int256(Signed::<256, 4>::from_str("100").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("30").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("70").unwrap()),
            ),
            (
                Operator::Mul,
                SolidityValue::Uint16(Uint::<16, 1>::from(3_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(4_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(12_u16)),
            ),
            (
                Operator::Mul,
                SolidityValue::Uint256(Uint::<256, 4>::from(5_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(6_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(30_u64)),
            ),
            (
                Operator::Mul,
                SolidityValue::Int16(Signed::<16, 1>::from_str("3").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("4").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("12").unwrap()),
            ),
            (
                Operator::Mul,
                SolidityValue::Int256(Signed::<256, 4>::from_str("5").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("6").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("30").unwrap()),
            ),
            (
                Operator::Div,
                SolidityValue::Uint16(Uint::<16, 1>::from(10_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(2_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::from(5_u16)),
            ),
            (
                Operator::Div,
                SolidityValue::Uint256(Uint::<256, 4>::from(100_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(4_u64)),
                SolidityValue::Uint256(Uint::<256, 4>::from(25_u64)),
            ),
            (
                Operator::Div,
                SolidityValue::Int16(Signed::<16, 1>::from_str("10").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("2").unwrap()),
                SolidityValue::Int16(Signed::<16, 1>::from_str("5").unwrap()),
            ),
            (
                Operator::Div,
                SolidityValue::Int256(Signed::<256, 4>::from_str("100").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("4").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::from_str("25").unwrap()),
            ),
        ];
        for (i, (op, a, b, expected)) in cases.into_iter().enumerate() {
            let (success, result) = safe_compute(op, a, b).unwrap();
            assert!(success, "case {i}");
            assert_eq!(result, expected, "case {i}");
        }
    }

    #[test]
    fn safe_compute_returns_false_and_zero_on_overflow_for_all_types_and_operators() {
        let cases = [
            (
                Operator::Add,
                SolidityValue::Uint16(Uint::<16, 1>::MAX),
                SolidityValue::Uint16(Uint::<16, 1>::from(1_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
            ),
            (
                Operator::Sub,
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
                SolidityValue::Uint16(Uint::<16, 1>::from(1_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
            ),
            (
                Operator::Mul,
                SolidityValue::Uint16(Uint::<16, 1>::MAX),
                SolidityValue::Uint16(Uint::<16, 1>::from(2_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
            ),
            (
                Operator::Div,
                SolidityValue::Uint16(Uint::<16, 1>::from(10_u16)),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
                SolidityValue::Uint16(Uint::<16, 1>::ZERO),
            ),
            (
                Operator::Add,
                SolidityValue::Int256(Signed::<256, 4>::MAX),
                SolidityValue::Int256(Signed::<256, 4>::from_str("1").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
            ),
            (
                Operator::Sub,
                SolidityValue::Int256(Signed::<256, 4>::MIN),
                SolidityValue::Int256(Signed::<256, 4>::from_str("1").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
            ),
            (
                Operator::Mul,
                SolidityValue::Int256(Signed::<256, 4>::MAX),
                SolidityValue::Int256(Signed::<256, 4>::from_str("2").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
            ),
            (
                Operator::Div,
                SolidityValue::Int256(Signed::<256, 4>::from_str("10").unwrap()),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
                SolidityValue::Int256(Signed::<256, 4>::ZERO),
            ),
        ];
        for (i, (op, a, b, expected_zero)) in cases.into_iter().enumerate() {
            let (success, result) = safe_compute(op, a, b).unwrap();
            assert!(!success, "case {i}");
            assert_eq!(result, expected_zero, "case {i}");
        }
    }

    #[test]
    fn compute_succeeds_for_int256_add_sub_and_div() {
        let a = SolidityValue::Int256(Signed::<256, 4>::from_str("-512").unwrap());
        let b = SolidityValue::Int256(Signed::<256, 4>::from_str("256").unwrap());
        let add_result = compute(Operator::Add, a.clone(), b.clone()).unwrap();
        let sub_result = compute(Operator::Sub, a.clone(), b.clone()).unwrap();
        let div_result = compute(Operator::Div, a, b).unwrap();
        assert_eq!(
            add_result,
            SolidityValue::Int256(Signed::<256, 4>::from_str("-256").unwrap())
        );
        assert_eq!(
            sub_result,
            SolidityValue::Int256(Signed::<256, 4>::from_str("-768").unwrap())
        );
        assert_eq!(
            div_result,
            SolidityValue::Int256(Signed::<256, 4>::from_str("-2").unwrap())
        );
    }

    #[test]
    fn compute_returns_product_for_int256_mul() {
        let a = SolidityValue::Int256(Signed::<256, 4>::from_str("-5").unwrap());
        let b = SolidityValue::Int256(Signed::<256, 4>::from_str("4").unwrap());
        let result = compute(Operator::Mul, a, b).unwrap();
        assert_eq!(
            result,
            SolidityValue::Int256(Signed::<256, 4>::from_str("-20").unwrap())
        );
    }

    #[test]
    fn compute_wraps_for_int256_when_add_and_sub_overflow() {
        let add_result = compute(
            Operator::Add,
            SolidityValue::Int256(Signed::<256, 4>::MAX),
            SolidityValue::Int256(Signed::<256, 4>::from_str("1").unwrap()),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            SolidityValue::Int256(Signed::<256, 4>::MIN),
            SolidityValue::Int256(Signed::<256, 4>::from_str("1").unwrap()),
        )
        .unwrap();
        assert_eq!(add_result, SolidityValue::Int256(Signed::<256, 4>::MIN));
        assert_eq!(sub_result, SolidityValue::Int256(Signed::<256, 4>::MAX));
    }

    #[test]
    fn compute_wraps_for_uint16_when_add_sub_and_mul_overflow() {
        let add_result = compute(
            Operator::Add,
            SolidityValue::Uint16(Uint::<16, 1>::MAX),
            SolidityValue::Uint16(Uint::<16, 1>::from(1_u16)),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            SolidityValue::Uint16(Uint::<16, 1>::ZERO),
            SolidityValue::Uint16(Uint::<16, 1>::from(1_u16)),
        )
        .unwrap();
        let mul_result = compute(
            Operator::Mul,
            SolidityValue::Uint16(Uint::<16, 1>::from(256_u16)),
            SolidityValue::Uint16(Uint::<16, 1>::from(256_u16)),
        )
        .unwrap();
        assert_eq!(add_result, SolidityValue::Uint16(Uint::<16, 1>::ZERO));
        assert_eq!(sub_result, SolidityValue::Uint16(Uint::<16, 1>::MAX));
        assert_eq!(mul_result, SolidityValue::Uint16(Uint::<16, 1>::ZERO));
    }

    #[test]
    fn compute_wraps_for_int16_when_add_sub_and_mul_overflow() {
        let add_result = compute(
            Operator::Add,
            SolidityValue::Int16(Signed::<16, 1>::MAX),
            SolidityValue::Int16(Signed::<16, 1>::from_str("1").unwrap()),
        )
        .unwrap();
        let sub_result = compute(
            Operator::Sub,
            SolidityValue::Int16(Signed::<16, 1>::MIN),
            SolidityValue::Int16(Signed::<16, 1>::from_str("1").unwrap()),
        )
        .unwrap();
        let mul_result = compute(
            Operator::Mul,
            SolidityValue::Int16(Signed::<16, 1>::MAX),
            SolidityValue::Int16(Signed::<16, 1>::from_str("2").unwrap()),
        )
        .unwrap();
        assert_eq!(add_result, SolidityValue::Int16(Signed::<16, 1>::MIN));
        assert_eq!(sub_result, SolidityValue::Int16(Signed::<16, 1>::MAX));
        assert_eq!(
            mul_result,
            SolidityValue::Int16(Signed::<16, 1>::from_str("-2").unwrap())
        );
    }

    #[test]
    fn compute_wraps_for_uint256_when_mul_overflows() {
        let result = compute(
            Operator::Mul,
            SolidityValue::Uint256(Uint::<256, 4>::MAX),
            SolidityValue::Uint256(Uint::<256, 4>::from(2_u64)),
        )
        .unwrap();
        assert_eq!(
            result,
            SolidityValue::Uint256(
                Uint::<256, 4>::from_str(
                    "115792089237316195423570985008687907853269984665640564039457584007913129639934"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn compute_wraps_for_int256_when_mul_overflows() {
        let result = compute(
            Operator::Mul,
            SolidityValue::Int256(Signed::<256, 4>::MAX),
            SolidityValue::Int256(Signed::<256, 4>::from_str("2").unwrap()),
        )
        .unwrap();
        assert_eq!(
            result,
            SolidityValue::Int256(Signed::<256, 4>::from_str("-2").unwrap())
        );
    }

    #[test]
    fn compute_returns_error_when_types_mismatch() {
        let a = SolidityValue::Uint16(Uint::<16, 1>::from(1_u16));
        let b = SolidityValue::Uint256(Uint::<256, 4>::from(1_u64));
        let result = compute(Operator::Add, a, b);
        assert!(result.is_err());
    }
}
