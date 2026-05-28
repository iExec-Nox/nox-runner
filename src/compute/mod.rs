//! Off-chain computations producing results compatible with Solidity.
//!
//! The module contains the SolidityValue enum to encode and decode handle values
//! to alloy-primitives associated types. This is the API which allows to perform
//! computations on all supported types.

use std::sync::atomic::{Ordering, compiler_fence};

use alloy_primitives::{Signed, Uint, hex};
use tracing::error;
use zeroize::Zeroize;

pub mod arithmetic;
pub mod boolean;
pub mod token;

const HANDLE_TYPE_BYTE: usize = 5;

/// Wraps around booleans and signed and unsigned integers provided by alloy-primitives.
///
/// Types are ordered following Nox protocol specification to represent and encode Solidity types.
///
/// For each supported Solidity type, the associated value is encoded following its
/// [`formal specification`](https://docs.soliditylang.org/en/latest/abi-spec.html#formal-specification-of-the-encoding).
#[derive(Clone, Debug, PartialEq)]
pub enum SolidityValue {
    Boolean(bool),
    Uint16(Uint<16, 1>),
    Uint256(Uint<256, 4>),
    Int16(Signed<16, 1>),
    Int256(Signed<256, 4>),
}

impl Zeroize for SolidityValue {
    fn zeroize(&mut self) {
        match self {
            SolidityValue::Boolean(v) => v.zeroize(),
            SolidityValue::Uint16(v) => {
                *v = Uint::<16, 1>::from_be_bytes([0u8; 2]);
                compiler_fence(Ordering::SeqCst);
            }
            SolidityValue::Uint256(v) => {
                *v = Uint::<256, 4>::from_be_bytes([0u8; 32]);
                compiler_fence(Ordering::SeqCst);
            }
            SolidityValue::Int16(v) => {
                *v = Signed::<16, 1>::from_be_bytes([0u8; 2]);
                compiler_fence(Ordering::SeqCst);
            }
            SolidityValue::Int256(v) => {
                *v = Signed::<256, 4>::from_be_bytes([0u8; 32]);
                compiler_fence(Ordering::SeqCst);
            }
        }
    }
}

impl SolidityValue {
    /// Converts from 32 big-endian bytes to alloy-primitives type.
    ///
    /// The following casting rules are implemented:
    /// * For booleans, when all 32 bytes from `value_bytes` are `0`, it returns `false`, `true` otherwise.
    /// * For signed and unsigned integers, `value_bytes` are truncated depending on the target type size.
    pub fn from_bytes(type_byte: u8, value_bytes: [u8; 32]) -> Result<Self, String> {
        Ok(match type_byte {
            0_u8 => {
                if value_bytes == [0u8; 32] {
                    SolidityValue::Boolean(false)
                } else {
                    SolidityValue::Boolean(true)
                }
            }
            5_u8 => SolidityValue::Uint16(Uint::<16, 1>::from_be_bytes::<2>(
                value_bytes[30..32]
                    .try_into()
                    .map_err(|_| format!("Failed to convert {value_bytes:?} bytes to uint16"))?,
            )),
            35_u8 => SolidityValue::Uint256(Uint::<256, 4>::from_be_bytes(value_bytes)),
            37_u8 => SolidityValue::Int16(Signed::<16, 1>::from_be_bytes::<2>(
                value_bytes[30..32]
                    .try_into()
                    .map_err(|_| format!("Failed to convert {value_bytes:?} bytes to int16"))?,
            )),
            67_u8 => SolidityValue::Int256(Signed::<256, 4>::from_be_bytes(value_bytes)),
            v => return Err(format!("Unsupported type {v} cannot be converted")),
        })
    }

    /// Converts from alloy-primitives type to 32 big-endian bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut result = [0_u8; 32];
        match self {
            SolidityValue::Boolean(v) => {
                if *v {
                    result[31] = 1
                }
            }
            SolidityValue::Uint16(v) => result[30..32].copy_from_slice(&v.to_be_bytes::<2>()),
            SolidityValue::Uint256(v) => result.copy_from_slice(&v.to_be_bytes::<32>()),
            SolidityValue::Int16(v) => result[30..32].copy_from_slice(&v.to_be_bytes::<2>()),
            SolidityValue::Int256(v) => result.copy_from_slice(&v.to_be_bytes::<32>()),
        }
        result
    }
}

/// Extracts solidity type from handle hex value
pub fn get_solidity_type_from_handle(handle: &str) -> Result<u8, String> {
    match hex::decode(handle) {
        Ok(v) => Ok(v[HANDLE_TYPE_BYTE]),
        Err(e) => Err(format!("Failed to decode handle hex value {e}")),
    }
}

/// Gets size in bytes of a given solidity type encoded as a byte
pub fn get_solidity_type_size(solidity_type: u8) -> Result<usize, String> {
    let solidity_type_size = match solidity_type {
        0 => 1,
        1 => 20,
        2..4 => 32,
        v @ 4..36 => v - 3,
        v @ 36..68 => v - 35,
        v @ 68..100 => v - 67,
        v => {
            let message = format!("Unsupported TEE type for encryption ({v})");
            error!(message);
            return Err(message);
        }
    };
    Ok(solidity_type_size as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Signed, Uint};

    // --- from_bytes ---

    #[test]
    fn from_bytes_boolean_false_when_all_zeros() {
        let result = SolidityValue::from_bytes(0_u8, [0u8; 32]).unwrap();
        assert_eq!(result, SolidityValue::Boolean(false));
    }

    #[test]
    fn from_bytes_boolean_true_when_not_all_zeros() {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        let result = SolidityValue::from_bytes(0_u8, bytes).unwrap();
        assert_eq!(result, SolidityValue::Boolean(true));
    }

    #[test]
    fn from_bytes_uint16() {
        let mut bytes = [0u8; 32];
        bytes[30] = 0x00;
        bytes[31] = 0x05; // valeur 5 en uint16
        let result = SolidityValue::from_bytes(5_u8, bytes).unwrap();
        assert_eq!(result, SolidityValue::Uint16(Uint::<16, 1>::from(5_u16)));
    }

    #[test]
    fn from_bytes_uint256() {
        let mut bytes = [0u8; 32];
        bytes[31] = 0xFF;
        let result = SolidityValue::from_bytes(35_u8, bytes).unwrap();
        assert_eq!(
            result,
            SolidityValue::Uint256(Uint::<256, 4>::from(255_u64))
        );
    }

    #[test]
    fn from_bytes_int16() {
        let mut bytes = [0u8; 32];
        bytes[30] = 0x00;
        bytes[31] = 0x0A; // valeur 10 en int16
        let result = SolidityValue::from_bytes(37_u8, bytes).unwrap();
        assert_eq!(
            result,
            SolidityValue::Int16(Signed::<16, 1>::try_from(10_i16).unwrap())
        );
    }

    #[test]
    fn from_bytes_unsupported_type_returns_error() {
        let result = SolidityValue::from_bytes(99_u8, [0u8; 32]);
        assert!(result.is_err());
    }

    // --- to_bytes ---

    #[test]
    fn to_bytes_boolean_true_sets_last_byte() {
        let val = SolidityValue::Boolean(true);
        let bytes = val.to_bytes();
        let mut expected = [0u8; 32];
        expected[31] = 1;
        assert_eq!(bytes, expected);
    }

    #[test]
    fn to_bytes_boolean_false_all_zeros() {
        let val = SolidityValue::Boolean(false);
        assert_eq!(val.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn from_bytes_to_bytes_roundtrip_uint16() {
        let mut original = [0u8; 32];
        original[31] = 42;
        let val = SolidityValue::from_bytes(5_u8, original).unwrap();
        assert_eq!(val.to_bytes(), original);
    }

    #[test]
    fn from_bytes_to_bytes_roundtrip_uint256() {
        let mut original = [0u8; 32];
        original[31] = 0xFF;
        let val = SolidityValue::from_bytes(35_u8, original).unwrap();
        assert_eq!(val.to_bytes(), original);
    }

    // --- get_solidity_type_from_handle ---

    #[test]
    fn get_solidity_type_from_handle_extracts_byte_at_position_5() {
        // handle hex de 32 bytes : le byte à l'index 5 vaut 0x23 = 35 (Uint256)
        let handle = "0x0000000000230000000000000000000000000000000000000000000000000000";
        let result = get_solidity_type_from_handle(handle).unwrap();
        assert_eq!(result, 35_u8);
    }

    #[test]
    fn get_solidity_type_from_handle_invalid_hex_returns_error() {
        let result = get_solidity_type_from_handle("not_hex");
        assert!(result.is_err());
    }

    // --- get_solidity_type_size ---

    #[test]
    fn get_solidity_type_size_boolean_is_1_byte() {
        assert_eq!(get_solidity_type_size(0).unwrap(), 1);
    }

    #[test]
    fn get_solidity_type_size_address_is_20_bytes() {
        assert_eq!(get_solidity_type_size(1).unwrap(), 20);
    }

    #[test]
    fn get_solidity_type_size_uint16_is_2_bytes() {
        // type_byte=5 → 5 - 3 = 2
        assert_eq!(get_solidity_type_size(5).unwrap(), 2);
    }

    #[test]
    fn get_solidity_type_size_uint256_is_32_bytes() {
        // type_byte=35 → 35 - 3 = 32
        assert_eq!(get_solidity_type_size(35).unwrap(), 32);
    }

    #[test]
    fn get_solidity_type_size_unsupported_returns_error() {
        assert!(get_solidity_type_size(200).is_err());
    }
}
