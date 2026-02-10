//! Address matching against parking restriction database
//!
//! Provides functions to validate and match user-provided addresses
//! against the pre-computed correlations from the amp server.
//!
//! # Overview
//! This module uses the new `DB` struct with proper timestamp handling.
//! It provides fast address lookup and validation for parking restrictions.
//!
//! # Examples
//! ```no_run
//! use amp_android::matching::{match_address, MatchResult};
//!
//! match match_address("Storgatan", "10", "22100") {
//!     MatchResult::Valid(entry) => {
//!         println!("Found restriction: {}", entry.adress);
//!         // Use entry.start_time and entry.end_time for time calculations
//!     }
//!     MatchResult::Invalid => println!("Validation failed"),
//! }
//! ```
use crate::components::static_data::{get_address_data, get_static_data};
use amp_core::structs::DB;
use std::collections::HashMap;
use std::fmt;
/// Validation errors for address input
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Street name is empty or whitespace-only
    EmptyStreet,
    /// Street number is empty or whitespace-only
    EmptyStreetNumber,
    /// Postal code is empty or whitespace-only
    EmptyPostalCode,
    /// Postal code format is invalid (must be 5 digits, optionally with space after 3rd digit)
    InvalidPostalCodeFormat(String),
    /// Street name exceeds maximum length
    StreetTooLong(usize),
    /// Street number exceeds maximum length
    StreetNumberTooLong(usize),
}
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyStreet => write!(f, "Street name cannot be empty"),
            ValidationError::EmptyStreetNumber => {
                write!(f, "Street number cannot be empty")
            }
            ValidationError::EmptyPostalCode => write!(f, "Postal code cannot be empty"),
            ValidationError::InvalidPostalCodeFormat(code) => {
                write!(
                    f,
                    "Invalid Swedish postal code format '{}' (expected 5 digits like '22100' or '221 00')",
                    code,
                )
            }
            ValidationError::StreetTooLong(len) => {
                write!(
                    f,
                    "Street name too long ({} characters, maximum is 100)",
                    len
                )
            }
            ValidationError::StreetNumberTooLong(len) => {
                write!(
                    f,
                    "Street number too long ({} characters, maximum is 20)",
                    len
                )
            }
        }
    }
}
impl std::error::Error for ValidationError {}
/// Result of address matching operation
///
/// Represents whether an address was found in the parking restriction database.
#[derive(Debug, Clone)]
pub enum MatchResult {
    /// Address found in database with full details
    ///
    /// Contains a `DB` struct with parking restriction information including
    /// timestamps, address details, and parking zone information.
    Valid(Box<DB>),
    /// Address not found or validation failed
    Invalid,
}
/// Maximum length for street names (prevents abuse)
const MAX_STREET_LENGTH: usize = 100;
/// Maximum length for street numbers (prevents abuse)
const MAX_STREET_NUMBER_LENGTH: usize = 20;
/// Get parking restriction data
///
/// Returns a reference to the parking data HashMap with address keys
/// mapping to DB entries containing parking restriction information.
///
/// # Returns
/// Static reference to parking data using DB struct
///
/// # Examples
/// ```no_run
/// use amp_android::matching::get_parking_data;
///
/// let data = get_parking_data();
/// println!("Loaded {} parking entries", data.len());
/// ```
pub fn get_parking_data() -> &'static HashMap<String, DB> {
    get_static_data()
}
/// Validate Swedish postal code format
///
/// Swedish postal codes are 5 digits, optionally formatted as "XXX XX" with a space.
/// Valid formats:
/// - "22100" (5 digits without space)
/// - "221 00" (5 digits with space after 3rd digit)
///
/// # Arguments
/// * `postal_code` - Postal code to validate
///
/// # Returns
/// * `Ok(())` - Postal code is valid
/// * `Err(ValidationError::InvalidPostalCodeFormat)` - Invalid format
///
/// # Examples
/// ```
/// use amp_android::matching::validate_postal_code;
///
/// assert!(validate_postal_code("22100").is_ok());
/// assert!(validate_postal_code("221 00").is_ok());
/// assert!(validate_postal_code("1234").is_err());  // Too short
/// assert!(validate_postal_code("12345a").is_err());  // Contains letter
/// ```
pub fn validate_postal_code(postal_code: &str) -> Result<(), ValidationError> {
    let trimmed = postal_code.trim();
    let normalized = trimmed.replace(' ', "");
    if normalized.len() == 5 && normalized.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::InvalidPostalCodeFormat(
            postal_code.to_string(),
        ))
    }
}
/// Validate address input fields with detailed error messages
///
/// Performs comprehensive validation:
/// - Non-empty fields after trimming
/// - Swedish postal code format (5 digits, optionally "XXX XX")
/// - Maximum length constraints
///
/// # Arguments
/// * `street` - Street name
/// * `street_number` - Street number (can contain letters like "10A")
/// * `postal_code` - Swedish postal code
///
/// # Returns
/// * `Ok(())` - All validations passed
/// * `Err(ValidationError)` - Specific validation error
///
/// # Examples
/// ```
/// use amp_android::matching::{validate_input, ValidationError};
///
/// assert!(validate_input("Storgatan", "10", "22100").is_ok());
/// assert!(validate_input("Storgatan", "10A", "221 00").is_ok());
///
/// match validate_input("", "10", "22100") {
///     Err(ValidationError::EmptyStreet) => println!("Street is empty"),
///     _ => {}
/// }
///
/// match validate_input("Storgatan", "10", "1234") {
///     Err(ValidationError::InvalidPostalCodeFormat(_)) => println!("Bad postal code"),
///     _ => {}
/// }
/// ```
pub fn validate_input(
    street: &str,
    street_number: &str,
    postal_code: &str,
) -> Result<(), ValidationError> {
    if street.trim().is_empty() {
        return Err(ValidationError::EmptyStreet);
    }
    if street_number.trim().is_empty() {
        return Err(ValidationError::EmptyStreetNumber);
    }
    if postal_code.trim().is_empty() {
        return Err(ValidationError::EmptyPostalCode);
    }
    if street.trim().len() > MAX_STREET_LENGTH {
        return Err(ValidationError::StreetTooLong(street.trim().len()));
    }
    if street_number.trim().len() > MAX_STREET_NUMBER_LENGTH {
        return Err(ValidationError::StreetNumberTooLong(
            street_number.trim().len(),
        ));
    }
    validate_postal_code(postal_code)?;
    Ok(())
}
/// Match user input address against static correlations from server
///
/// This checks if the provided address (street, street_number, postal_code) exists
/// in the pre-computed correlations generated by:
/// `cargo run --release -- output --android`
///
/// The function performs:
/// 1. Input validation with detailed error messages
/// 2. Key generation (format: "street_number_postalcode")
/// 3. Database lookup
///
/// # Arguments
/// * `street` - Street name (e.g., "Storgatan")
/// * `street_number` - Street number (e.g., "10" or "10A")
/// * `postal_code` - Swedish postal code (e.g., "22100" or "221 00")
///
/// # Returns
/// * `MatchResult::Valid(DB)` - Address found with parking restriction data
/// * `MatchResult::Invalid` - Validation failed or address not found
///
/// # Examples
/// ```no_run
/// use amp_android::matching::{match_address, MatchResult};
/// use chrono::Utc;
///
/// match match_address("Storgatan", "10", "22100") {
///     MatchResult::Valid(entry) => {
///         println!("Found: {}", entry.adress);
///         let now = Utc::now();
///         if entry.is_active(now) {
///             println!("Restriction is currently active!");
///         }
///     }
///     MatchResult::Invalid => {
///         eprintln!("Address not found or invalid");
///     }
/// }
/// ```
///
/// # Performance
/// This is an O(1) HashMap lookup operation after validation.
pub fn match_address(street: &str, street_number: &str, postal_code: &str) -> MatchResult {
    if let Err(e) = validate_input(street, street_number, postal_code) {
        eprintln!("[Matching] Validation error: {}", e);
        return MatchResult::Invalid;
    }
    let postal_normalized = postal_code.trim().replace(' ', "");
    match get_address_data(street, street_number, &postal_normalized) {
        Some(entry) => {
            eprintln!(
                "[Matching] Found address: {} {} {}",
                street, street_number, postal_code,
            );
            MatchResult::Valid(Box::from(entry.clone()))
        }
        None => {
            eprintln!(
                "[Matching] Address not found: {} {} {}",
                street, street_number, postal_code,
            );
            MatchResult::Invalid
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_postal_code() {
        assert!(validate_postal_code("22100").is_ok());
        assert!(validate_postal_code("221 00").is_ok());
        assert!(validate_postal_code(" 22100 ").is_ok());
        assert!(validate_postal_code(" 221 00 ").is_ok());
        assert!(validate_postal_code("1234").is_err());
        assert!(validate_postal_code("123456").is_err());
        assert!(validate_postal_code("12345a").is_err());
        assert!(validate_postal_code("abc12").is_err());
        assert!(validate_postal_code("").is_err());
    }
    #[test]
    fn test_validate_input() {
        assert!(validate_input("Storgatan", "10", "22100").is_ok());
        assert!(validate_input("Storgatan", "10A", "221 00").is_ok());
        assert!(validate_input(" Storgatan ", " 10 ", " 22100 ").is_ok());
        match validate_input("", "10", "22100") {
            Err(ValidationError::EmptyStreet) => {}
            _ => panic!("Expected EmptyStreet error"),
        }
        match validate_input("Storgatan", "", "22100") {
            Err(ValidationError::EmptyStreetNumber) => {}
            _ => panic!("Expected EmptyStreetNumber error"),
        }
        match validate_input("Storgatan", "10", "") {
            Err(ValidationError::EmptyPostalCode) => {}
            _ => panic!("Expected EmptyPostalCode error"),
        }
        match validate_input("Storgatan", "10", "1234") {
            Err(ValidationError::InvalidPostalCodeFormat(_)) => {}
            _ => panic!("Expected InvalidPostalCodeFormat error"),
        }
    }
    #[test]
    fn test_validate_input_max_length() {
        let long_street = "A".repeat(MAX_STREET_LENGTH + 1);
        match validate_input(&long_street, "10", "22100") {
            Err(ValidationError::StreetTooLong(_)) => {}
            _ => panic!("Expected StreetTooLong error"),
        }
        let long_number = "1".repeat(MAX_STREET_NUMBER_LENGTH + 1);
        match validate_input("Storgatan", &long_number, "22100") {
            Err(ValidationError::StreetNumberTooLong(_)) => {}
            _ => panic!("Expected StreetNumberTooLong error"),
        }
    }
    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::EmptyStreet;
        assert_eq!(format!("{}", err), "Street name cannot be empty");
        let err = ValidationError::InvalidPostalCodeFormat("1234".to_string());
        assert!(format!("{}", err).contains("1234"));
    }
}
