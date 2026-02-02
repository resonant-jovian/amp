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
//!     MatchResult::Invalid(err) => println!("Validation failed: {}", err),
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
            ValidationError::EmptyStreetNumber => write!(f, "Street number cannot be empty"),
            ValidationError::EmptyPostalCode => write!(f, "Postal code cannot be empty"),
            ValidationError::InvalidPostalCodeFormat(code) => write!(
                f,
                "Invalid Swedish postal code format '{}' (expected 5 digits like '22100' or '221 00')",
                code
            ),
            ValidationError::StreetTooLong(len) => write!(
                f,
                "Street name too long ({} characters, maximum is 100)",
                len
            ),
            ValidationError::StreetNumberTooLong(len) => write!(
                f,
                "Street number too long ({} characters, maximum is 20)",
                len
            ),
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
    ///
    /// Contains the validation error that caused the failure.
    Invalid(ValidationError),
}

impl MatchResult {
    /// Check if the result is valid
    ///
    /// # Returns
    /// true if Valid variant, false if Invalid
    ///
    /// # Examples
    /// ```no_run
    /// use amp_android::matching::{match_address, MatchResult};
    ///
    /// let result = match_address("Storgatan", "10", "22100");
    /// if result.is_valid() {
    ///     println!("Address found!");
    /// }
    /// ```
    pub fn is_valid(&self) -> bool {
        matches!(self, MatchResult::Valid(_))
    }

    /// Check if the result is invalid
    ///
    /// # Returns
    /// true if Invalid variant, false if Valid
    pub fn is_invalid(&self) -> bool {
        matches!(self, MatchResult::Invalid(_))
    }

    /// Get the DB entry if valid
    ///
    /// # Returns
    /// Some(&DB) if Valid, None if Invalid
    ///
    /// # Examples
    /// ```no_run
    /// use amp_android::matching::match_address;
    ///
    /// let result = match_address("Storgatan", "10", "22100");
    /// if let Some(db) = result.as_ref() {
    ///     println!("Address: {}", db.adress);
    /// }
    /// ```
    pub fn as_ref(&self) -> Option<&DB> {
        match self {
            MatchResult::Valid(db) => Some(db),
            MatchResult::Invalid(_) => None,
        }
    }

    /// Consume the result and get the DB entry if valid
    ///
    /// # Returns
    /// Some(DB) if Valid, None if Invalid
    pub fn into_inner(self) -> Option<DB> {
        match self {
            MatchResult::Valid(db) => Some(*db),
            MatchResult::Invalid(_) => None,
        }
    }
    
    /// Get the validation error if invalid
    ///
    /// # Returns
    /// Some(&ValidationError) if Invalid, None if Valid
    pub fn error(&self) -> Option<&ValidationError> {
        match self {
            MatchResult::Invalid(err) => Some(err),
            MatchResult::Valid(_) => None,
        }
    }
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
    
    // Remove optional space for validation
    let normalized = trimmed.replace(' ', "");
    
    // Must be exactly 5 digits
    if normalized.len() == 5 && normalized.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::InvalidPostalCodeFormat(postal_code.to_string()))
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
    // Check non-empty
    if street.trim().is_empty() {
        return Err(ValidationError::EmptyStreet);
    }
    if street_number.trim().is_empty() {
        return Err(ValidationError::EmptyStreetNumber);
    }
    if postal_code.trim().is_empty() {
        return Err(ValidationError::EmptyPostalCode);
    }
    
    // Check maximum lengths (prevent abuse)
    if street.trim().len() > MAX_STREET_LENGTH {
        return Err(ValidationError::StreetTooLong(street.trim().len()));
    }
    if street_number.trim().len() > MAX_STREET_NUMBER_LENGTH {
        return Err(ValidationError::StreetNumberTooLong(street_number.trim().len()));
    }
    
    // Validate Swedish postal code format
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
/// * `MatchResult::Invalid(ValidationError)` - Validation failed or address not found
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
///     MatchResult::Invalid(err) => {
///         eprintln!("Validation failed: {}", err);
///     }
/// }
/// ```
///
/// # Performance
/// This is an O(1) HashMap lookup operation after validation.
pub fn match_address(street: &str, street_number: &str, postal_code: &str) -> MatchResult {
    if let Err(e) = validate_input(street, street_number, postal_code) {
        eprintln!("[Matching] Validation error: {}", e);
        return MatchResult::Invalid(e);
    }

    // Normalize postal code (remove space if present)
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
            // Address not found is treated as EmptyPostalCode for now
            // Could add a NotFound variant to ValidationError if needed
            MatchResult::Invalid(ValidationError::EmptyPostalCode)
        }
    }
}

/// Match address with fuzzy logic (case-insensitive, trimmed)
///
/// More lenient version of `match_address` that:
/// - Trims whitespace from all inputs
/// - Converts to lowercase for comparison
/// - Normalizes postal codes (removes spaces)
/// - Still validates format
///
/// # Arguments
/// * `street` - Street name (case-insensitive)
/// * `street_number` - Street number
/// * `postal_code` - Postal code (with or without space)
///
/// # Returns
/// MatchResult with found entry or validation error
///
/// # Examples
/// ```no_run
/// use amp_android::matching::match_address_fuzzy;
///
/// // These all work:
/// let r1 = match_address_fuzzy("STORGATAN", "10", "22100");
/// let r2 = match_address_fuzzy(" storgatan ", " 10 ", " 221 00 ");
/// let r3 = match_address_fuzzy("Storgatan", "10", "22100");
/// ```
pub fn match_address_fuzzy(street: &str, street_number: &str, postal_code: &str) -> MatchResult {
    let street_norm = street.trim().to_lowercase();
    let number_norm = street_number.trim();
    let postal_norm = postal_code.trim().replace(' ', "");

    if let Err(e) = validate_input(&street_norm, number_norm, &postal_norm) {
        return MatchResult::Invalid(e);
    }

    let data = get_static_data();
    for (_, entry) in data.iter() {
        let entry_street = entry
            .gata
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let entry_number = entry.gatunummer.as_deref().unwrap_or("");
        let entry_postal = entry.postnummer.as_deref().unwrap_or("");

        if entry_street == street_norm && entry_number == number_norm && entry_postal == postal_norm
        {
            eprintln!("[Matching] Fuzzy match found: {}", entry.adress);
            return MatchResult::Valid(Box::from(entry.clone()));
        }
    }

    eprintln!(
        "[Matching] No fuzzy match for: {} {} {}",
        street, street_number, postal_code,
    );
    MatchResult::Invalid(ValidationError::EmptyPostalCode)
}

/// Search for addresses matching a partial street name
///
/// Returns all addresses where the street name contains the search term.
/// Case-insensitive search.
///
/// # Arguments
/// * `partial_street` - Partial street name to search for
///
/// # Returns
/// Vector of DB entries matching the search
///
/// # Examples
/// ```no_run
/// use amp_android::matching::search_by_street;
///
/// // Find all addresses on streets containing "stor"
/// let results = search_by_street("stor");
/// for db in results {
///     println!("Found: {}", db.adress);
/// }
/// ```
pub fn search_by_street(partial_street: &str) -> Vec<DB> {
    let search_term = partial_street.trim().to_lowercase();
    if search_term.is_empty() {
        return Vec::new();
    }

    get_static_data()
        .values()
        .filter(|db| {
            db.gata
                .as_ref()
                .map(|gata| gata.to_lowercase().contains(&search_term))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// Get all addresses in a specific postal code
///
/// Convenience wrapper around `static_data::get_addresses_in_postal_code`
/// that returns owned DB entries instead of references.
///
/// # Arguments
/// * `postal_code` - Postal code to filter by
///
/// # Returns
/// Vector of DB entries in the postal code
///
/// # Examples
/// ```no_run
/// use amp_android::matching::get_addresses_in_area;
///
/// let addresses = get_addresses_in_area("22100");
/// println!("Found {} addresses in 22100", addresses.len());
/// ```
pub fn get_addresses_in_area(postal_code: &str) -> Vec<DB> {
    use crate::components::static_data::get_addresses_in_postal_code;
    get_addresses_in_postal_code(postal_code)
        .into_iter()
        .cloned()
        .collect()
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
            Err(ValidationError::EmptyStreet) => {},
            _ => panic!("Expected EmptyStreet error"),
        }

        match validate_input("Storgatan", "", "22100") {
            Err(ValidationError::EmptyStreetNumber) => {},
            _ => panic!("Expected EmptyStreetNumber error"),
        }

        match validate_input("Storgatan", "10", "") {
            Err(ValidationError::EmptyPostalCode) => {},
            _ => panic!("Expected EmptyPostalCode error"),
        }

        match validate_input("Storgatan", "10", "1234") {
            Err(ValidationError::InvalidPostalCodeFormat(_)) => {},
            _ => panic!("Expected InvalidPostalCodeFormat error"),
        }
    }

    #[test]
    fn test_validate_input_max_length() {
        let long_street = "A".repeat(MAX_STREET_LENGTH + 1);
        match validate_input(&long_street, "10", "22100") {
            Err(ValidationError::StreetTooLong(_)) => {},
            _ => panic!("Expected StreetTooLong error"),
        }

        let long_number = "1".repeat(MAX_STREET_NUMBER_LENGTH + 1);
        match validate_input("Storgatan", &long_number, "22100") {
            Err(ValidationError::StreetNumberTooLong(_)) => {},
            _ => panic!("Expected StreetNumberTooLong error"),
        }
    }

    #[test]
    fn test_match_result_is_valid() {
        let valid = MatchResult::Valid(Box::from(
            DB::from_dag_tid(
                Some("22100".to_string()),
                "Test".to_string(),
                None,
                None,
                None,
                15,
                "0800-1200",
                None,
                None,
                None,
                2024,
                1,
            )
            .unwrap(),
        ));
        assert!(valid.is_valid());
        assert!(!valid.is_invalid());

        let invalid = MatchResult::Invalid(ValidationError::EmptyStreet);
        assert!(invalid.is_invalid());
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_match_result_as_ref() {
        let db = DB::from_dag_tid(
            Some("22100".to_string()),
            "Test Street 10".to_string(),
            Some("Test Street".to_string()),
            Some("10".to_string()),
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2024,
            1,
        )
        .unwrap();
        let valid = MatchResult::Valid(Box::from(db.clone()));
        assert!(valid.as_ref().is_some());
        assert_eq!(valid.as_ref().unwrap().adress, "Test Street 10");

        let invalid = MatchResult::Invalid(ValidationError::EmptyStreet);
        assert!(invalid.as_ref().is_none());
    }

    #[test]
    fn test_match_result_into_inner() {
        let db = DB::from_dag_tid(
            Some("22100".to_string()),
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2024,
            1,
        )
        .unwrap();
        let valid = MatchResult::Valid(Box::from(db));
        let inner = valid.into_inner();
        assert!(inner.is_some());

        let invalid = MatchResult::Invalid(ValidationError::EmptyStreet);
        assert!(invalid.into_inner().is_none());
    }

    #[test]
    fn test_search_by_street_empty() {
        let results = search_by_street("");
        assert_eq!(results.len(), 0);

        let results = search_by_street("  ");
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::EmptyStreet;
        assert_eq!(format!("{}", err), "Street name cannot be empty");
        
        let err = ValidationError::InvalidPostalCodeFormat("1234".to_string());
        assert!(format!("{}", err).contains("1234"));
    }
}
