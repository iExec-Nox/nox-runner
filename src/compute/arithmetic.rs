//! Arithmetic operations support.

use alloy_primitives::{Signed, Uint};

/// Supported arithmetic operators
pub enum Operator {
    Add,
    Sub,
    Div,
}

/// Wraps around signed and unsigned integers provided by alloy-primitives.
///
/// Types are ordered following Solidity types encoding specification.
#[derive(Clone, Debug, PartialEq)]
pub enum SolidityValue {
    Uint16(Uint<16, 1>),
    Uint256(Uint<256, 4>),
    Int16(Signed<16, 1>),
    Int256(Signed<256, 4>),
}

impl SolidityValue {
    /// Converts from 32 big-endian bytes to alloy-primitives type
    pub fn from_bytes(type_byte: u8, value_bytes: [u8; 32]) -> Result<Self, String> {
        match type_byte {
            5_u8 => Ok(SolidityValue::Uint16(Uint::<16, 1>::from_be_bytes::<2>(
                value_bytes[30..32]
                    .try_into()
                    .map_err(|_| "Failed to convert {value_bytes:?} bytes to uint16")?,
            ))),
            35_u8 => Ok(SolidityValue::Uint256(Uint::<256, 4>::from_be_bytes(
                value_bytes,
            ))),
            37_u8 => Ok(SolidityValue::Int16(Signed::<16, 1>::from_be_bytes::<2>(
                value_bytes[30..32]
                    .try_into()
                    .map_err(|_| format!("Failed to convert {value_bytes:?} bytes to int16"))?,
            ))),
            67_u8 => Ok(SolidityValue::Int256(Signed::<256, 4>::from_be_bytes(
                value_bytes,
            ))),
            v => Err(format!("Unsupported type {v} cannot be converted")),
        }
    }

    /// Converts from alloy-primitives type to 32 big-endian bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        match self {
            SolidityValue::Uint16(v) => result[30..32].copy_from_slice(&v.to_be_bytes::<2>()),
            SolidityValue::Uint256(v) => result.copy_from_slice(&v.to_be_bytes::<32>()),
            SolidityValue::Int16(v) => result[30..32].copy_from_slice(&v.to_be_bytes::<2>()),
            SolidityValue::Int256(v) => result.copy_from_slice(&v.to_be_bytes::<32>()),
        }
        result
    }
}

/// Performs Add, Sub or Div arithmetic operations on 16 or 256 bits signed or unsiged integers.
///
/// Add and Sub operations wrap around at the boundary of each type.
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
        (Operator::Div, SolidityValue::Uint16(a), SolidityValue::Uint16(b)) => {
            Ok(SolidityValue::Uint16(a / b))
        }
        (Operator::Div, SolidityValue::Uint256(a), SolidityValue::Uint256(b)) => {
            Ok(SolidityValue::Uint256(a / b))
        }
        (Operator::Div, SolidityValue::Int16(a), SolidityValue::Int16(b)) => {
            Ok(SolidityValue::Int16(a / b))
        }
        (Operator::Div, SolidityValue::Int256(a), SolidityValue::Int256(b)) => {
            Ok(SolidityValue::Int256(a / b))
        }
        _ => Err("Unsupported operation".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy_primitives::hex;

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
    fn check_uint16() {
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
    fn check_uint256() {
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
    fn check_int16() {
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
}
