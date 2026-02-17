//! Advanced operations on confidential tokens
//!
//! Atomic operations are secured by using [`alloy_primitives::Uint::checked_add`] and [`alloy_primitives::Uint::checked_sub`] operations.

use super::SolidityValue;

/// Confidential tokens transfer.
///
/// The values will be updated only if all atomic operations can be performed,
/// initial values will be returned otherwise.
///
/// # Errors
///
/// Returns an [`Err`] if all operands are not of [`SolidityValue::Uint256`] type.
pub fn transfer(
    wrapped_balance_from: SolidityValue,
    wrapped_balance_to: SolidityValue,
    wrapped_amount: SolidityValue,
) -> Result<(SolidityValue, SolidityValue, SolidityValue), String> {
    let (balance_from_value, balance_to_value, amount_value) =
        match (wrapped_balance_from, wrapped_balance_to, wrapped_amount) {
            (
                SolidityValue::Uint256(balance_from),
                SolidityValue::Uint256(balance_to),
                SolidityValue::Uint256(amount),
            ) => (balance_from, balance_to, amount),
            _ => return Err("Unsupported operation, invalid type in operands".to_string()),
        };
    let new_balance_from = balance_from_value.checked_sub(amount_value);
    let new_balance_to = balance_to_value.checked_add(amount_value);
    match (new_balance_from, new_balance_to) {
        (Some(new_balance_from_value), Some(new_balance_to_value)) => Ok((
            SolidityValue::Boolean(true),
            SolidityValue::Uint256(new_balance_from_value),
            SolidityValue::Uint256(new_balance_to_value),
        )),
        _ => Ok((
            SolidityValue::Boolean(false),
            SolidityValue::Uint256(balance_from_value),
            SolidityValue::Uint256(balance_to_value),
        )),
    }
}

/// Confidential tokens mint.
///
/// The values will be updated only if all atomic operations can be performed,
/// initial values will be returned otherwise.
///
/// # Errors
///
/// Returns an [`Err`] if all operands are not of [`SolidityValue::Uint256`] type.
pub fn mint(
    wrapped_balance_to: SolidityValue,
    wrapped_amount: SolidityValue,
    wrapped_total_supply: SolidityValue,
) -> Result<(SolidityValue, SolidityValue, SolidityValue), String> {
    let (balance_to_value, amount_value, total_supply_value) =
        match (wrapped_balance_to, wrapped_amount, wrapped_total_supply) {
            (
                SolidityValue::Uint256(balance_to),
                SolidityValue::Uint256(amount),
                SolidityValue::Uint256(total_supply),
            ) => (balance_to, amount, total_supply),
            _ => return Err("Unsupported operation, invalid type in operands".to_string()),
        };
    let new_balance_to = balance_to_value.checked_add(amount_value);
    let new_total_supply = total_supply_value.checked_add(amount_value);
    match (new_balance_to, new_total_supply) {
        (Some(new_balance_to_value), Some(new_total_supply_value)) => Ok((
            SolidityValue::Boolean(true),
            SolidityValue::Uint256(new_balance_to_value),
            SolidityValue::Uint256(new_total_supply_value),
        )),
        _ => Ok((
            SolidityValue::Boolean(false),
            SolidityValue::Uint256(balance_to_value),
            SolidityValue::Uint256(total_supply_value),
        )),
    }
}

/// Confidential tokens burn.
///
/// The values will be updated only if all atomic operations can be performed,
/// initial values will be returned otherwise.
///
/// # Errors
///
/// Returns an [`Err`] if all operands are not of [`SolidityValue::Uint256`] type.
pub fn burn(
    wrapped_balance_from: SolidityValue,
    wrapped_amount: SolidityValue,
    wrapped_total_supply: SolidityValue,
) -> Result<(SolidityValue, SolidityValue, SolidityValue), String> {
    let (balance_from_value, amount_value, total_supply_value) =
        match (wrapped_balance_from, wrapped_amount, wrapped_total_supply) {
            (
                SolidityValue::Uint256(balance_from),
                SolidityValue::Uint256(amount),
                SolidityValue::Uint256(total_supply),
            ) => (balance_from, amount, total_supply),
            _ => return Err("Unsupported operation, invalid type in operands".to_string()),
        };
    let new_balance_from = balance_from_value.checked_sub(amount_value);
    let new_total_supply = total_supply_value.checked_sub(amount_value);
    match (new_balance_from, new_total_supply) {
        (Some(new_balance_from_value), Some(new_total_supply_value)) => Ok((
            SolidityValue::Boolean(true),
            SolidityValue::Uint256(new_balance_from_value),
            SolidityValue::Uint256(new_total_supply_value),
        )),
        _ => Ok((
            SolidityValue::Boolean(false),
            SolidityValue::Uint256(balance_from_value),
            SolidityValue::Uint256(total_supply_value),
        )),
    }
}
