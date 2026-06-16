//! Advanced operations on confidential tokens
//!
//! Atomic operations are secured by using [`alloy::primitives::Uint::checked_add`] and [`alloy::primitives::Uint::checked_sub`] operations.

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Uint;

    fn u256(n: u64) -> SolidityValue {
        SolidityValue::Uint256(Uint::<256, 4>::from(n))
    }

    fn u256_max() -> SolidityValue {
        SolidityValue::Uint256(Uint::<256, 4>::MAX)
    }

    #[test]
    fn transfer_reduces_from_and_increases_to_when_balance_is_sufficient() {
        let (success, from, to) = transfer(u256(100), u256(50), u256(30)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(from, u256(70));
        assert_eq!(to, u256(80));
    }

    #[test]
    fn transfer_succeeds_when_amount_is_zero() {
        let (success, from, to) = transfer(u256(100), u256(50), u256(0)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(from, u256(100));
        assert_eq!(to, u256(50));
    }

    #[test]
    fn transfer_succeeds_when_amount_equals_balance() {
        let (success, from, to) = transfer(u256(50), u256(0), u256(50)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(from, u256(0));
        assert_eq!(to, u256(50));
    }

    #[test]
    fn transfer_insufficient_balance_returns_false_and_original_values() {
        let (success, from, to) = transfer(u256(10), u256(50), u256(50)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(from, u256(10));
        assert_eq!(to, u256(50));
    }

    #[test]
    fn transfer_returns_false_and_original_values_when_to_balance_overflows() {
        // from = 100 - 1 = 99 → ok; to = MAX + 1 → overflow
        let (success, from, to) = transfer(u256(100), u256_max(), u256(1)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(from, u256(100));
        assert_eq!(to, u256_max());
    }

    #[test]
    fn transfer_returns_false_and_original_values_when_both_conditions_fail() {
        // from = 0 - 1 → underflow; to = MAX + 1 → overflow
        let (success, from, to) = transfer(u256(0), u256_max(), u256(1)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(from, u256(0));
        assert_eq!(to, u256_max());
    }

    #[test]
    fn transfer_returns_error_when_type_is_wrong() {
        let result = transfer(SolidityValue::Boolean(true), u256(50), u256(10));
        assert!(result.is_err());
    }

    #[test]
    fn mint_increases_balance_and_supply_when_valid() {
        let (success, balance, supply) = mint(u256(100), u256(50), u256(1000)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(balance, u256(150));
        assert_eq!(supply, u256(1050));
    }

    #[test]
    fn mint_succeeds_when_amount_is_zero() {
        let (success, balance, supply) = mint(u256(100), u256(0), u256(1000)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(balance, u256(100));
        assert_eq!(supply, u256(1000));
    }

    #[test]
    fn mint_returns_false_and_original_values_on_overflow() {
        // supply overflow: total_supply(1) + amount(MAX) → fails
        let (success, balance, supply) = mint(u256(0), u256_max(), u256(1)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256(0));
        assert_eq!(supply, u256(1));
    }

    #[test]
    fn mint_returns_false_and_original_values_when_balance_overflows() {
        // balance(MAX) + amount(1) → overflow; supply returns original value (0)
        let (success, balance, supply) = mint(u256_max(), u256(1), u256(0)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256_max());
        assert_eq!(supply, u256(0));
    }

    #[test]
    fn mint_returns_false_and_original_values_when_both_overflow() {
        // balance = MAX + 1 → overflow; supply = MAX + 1 → overflow
        let (success, balance, supply) = mint(u256_max(), u256(1), u256_max()).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256_max());
        assert_eq!(supply, u256_max());
    }

    #[test]
    fn mint_returns_error_when_type_is_wrong() {
        let result = mint(SolidityValue::Boolean(false), u256(50), u256(1000));
        assert!(result.is_err());
    }

    #[test]
    fn burn_decreases_balance_and_supply_when_valid() {
        let (success, balance, supply) = burn(u256(100), u256(30), u256(1000)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(balance, u256(70));
        assert_eq!(supply, u256(970));
    }

    #[test]
    fn burn_succeeds_when_amount_equals_balance() {
        let (success, balance, supply) = burn(u256(50), u256(50), u256(1000)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(true));
        assert_eq!(balance, u256(0));
        assert_eq!(supply, u256(950));
    }

    #[test]
    fn burn_insufficient_balance_returns_false_and_original_values() {
        let (success, balance, supply) = burn(u256(10), u256(50), u256(1000)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256(10));
        assert_eq!(supply, u256(1000));
    }

    #[test]
    fn burn_returns_false_and_original_values_when_supply_underflows() {
        // balance = 100 - 50 = 50 → ok; supply = 30 - 50 → underflow
        let (success, balance, supply) = burn(u256(100), u256(50), u256(30)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256(100));
        assert_eq!(supply, u256(30));
    }

    #[test]
    fn burn_returns_false_and_original_values_when_both_underflow() {
        // balance = 10 - 50 → underflow; supply = 30 - 50 → underflow
        let (success, balance, supply) = burn(u256(10), u256(50), u256(30)).unwrap();
        assert_eq!(success, SolidityValue::Boolean(false));
        assert_eq!(balance, u256(10));
        assert_eq!(supply, u256(30));
    }

    #[test]
    fn burn_returns_error_when_type_is_wrong() {
        let result = burn(SolidityValue::Boolean(false), u256(50), u256(1000));
        assert!(result.is_err());
    }
}
