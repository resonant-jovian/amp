//! Debug mode functionality for loading example addresses
//!
//! Provides a read-only debug mode that loads addresses from a bundled debug.parquet file.
//! Useful for testing and development without affecting user data.
//!
//! # Features
//! - Loads debug addresses from embedded debug.parquet asset (minimal format with only address strings)
//! - Read-only: never writes to storage
//! - Can be toggled on/off via settings
//! - Uses fuzzy matching via StoredAddress::new() to match against static parking database
//! - Mimics clicking "Add Address" button multiple times with different addresses
//!
//! # Usage
//! ```no_run
//! use amp_android::components::debug;
//!
//! // Load debug addresses (automatically performs fuzzy matching)
//! let debug_addresses = debug::load_debug_addresses();
//!
//! // Toggle active state (in-memory only, not persisted)
//! let mut addresses = debug_addresses;
//! if let Some(addr) = addresses.iter_mut().find(|a| a.id == some_id) {
//!     addr.active = !addr.active;
//! }
//! ```
use crate::ui::StoredAddress;
use amp_core::parquet::load_debug_addresses as load_from_parquet;

/// Debug parquet file embedded in the app
/// Contains ONLY the 'adress' field - all other fields are NULL
/// This mimics user input via "Add Address" button
static DEBUG_PARQUET: &[u8] = include_bytes!("../../assets/data/debug.parquet");

/// Load debug addresses from embedded debug.parquet file
///
/// Reads the minimal debug.parquet file that contains ONLY address strings.
/// Each address is then processed using StoredAddress::new() which performs
/// fuzzy matching against the static parking database (just like clicking
/// "Add Address" in the UI).
///
/// # Returns
/// Vector of debug addresses with fuzzy matching applied, empty if loading fails
///
/// # Examples
/// ```no_run
/// use amp_android::components::debug;
///
/// let debug_addrs = debug::load_debug_addresses();
/// println!("Loaded {} debug addresses", debug_addrs.len());
/// 
/// // Check how many matched successfully
/// let valid_count = debug_addrs.iter().filter(|a| a.valid).count();
/// println!("{} addresses matched parking database", valid_count);
/// ```
pub fn load_debug_addresses() -> Vec<StoredAddress> {
    eprintln!("[Debug] Loading debug addresses from embedded parquet");
    
    match load_from_parquet(DEBUG_PARQUET) {
        Ok(stored_addresses) => {
            eprintln!(
                "[Debug] Successfully loaded {} debug addresses from minimal parquet",
                stored_addresses.len(),
            );
            
            // Convert core::StoredAddress to UI::StoredAddress using fuzzy matching
            let ui_addresses: Vec<StoredAddress> = stored_addresses
                .into_iter()
                .map(|core_addr| {
                    // Parse the address string to extract components
                    let (street, street_number, postal_code) = parse_debug_address(&core_addr.adress);
                    
                    eprintln!(
                        "[Debug] Creating UI address: street='{}', number='{}', postal='{}' from '{}'",
                        street, street_number, postal_code, core_addr.adress
                    );
                    
                    // Use StoredAddress::new() which performs fuzzy matching
                    StoredAddress::new(street, street_number, postal_code)
                })
                .collect();
            
            let valid_count = ui_addresses.iter().filter(|a| a.valid).count();
            eprintln!(
                "[Debug] Fuzzy matching results: {}/{} addresses matched parking database",
                valid_count,
                ui_addresses.len(),
            );
            
            ui_addresses
        }
        Err(e) => {
            eprintln!("[Debug] Failed to load debug addresses: {}", e);
            Vec::new()
        }
    }
}

/// Parse debug address string into components
///
/// Debug addresses from debug.txt are in format:
/// - "Kornettsgatan 18C" (no postal code in address string)
/// - "Idunsgatan 67B 214 46" (postal code embedded - but we extract from CSV column)
///
/// This function extracts:
/// 1. Street name (all words except last if last contains digits)
/// 2. Street number (last word if it contains digits)
/// 3. Postal code (from debug.txt first column, extracted from embedded postal if present)
///
/// # Arguments
/// * `address` - Full address string from debug.parquet
///
/// # Returns
/// Tuple of (street_name, street_number, postal_code)
fn parse_debug_address(address: &str) -> (String, String, String) {
    let parts: Vec<&str> = address.split_whitespace().collect();
    
    if parts.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    
    // Check if last two parts look like postal code (5 digits with optional space)
    let has_embedded_postal = parts.len() >= 3
        && parts[parts.len() - 2].chars().all(|c| c.is_ascii_digit())
        && parts[parts.len() - 1].chars().all(|c| c.is_ascii_digit())
        && parts[parts.len() - 2].len() == 3
        && parts[parts.len() - 1].len() == 2;
    
    if has_embedded_postal {
        // Format: "Street Number 211 50"
        let postal_code = format!("{}{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        
        if parts.len() >= 4 {
            let street_number = parts[parts.len() - 3].to_string();
            let street = parts[..parts.len() - 3].join(" ");
            return (street, street_number, postal_code);
        } else {
            // Only postal code, no street/number
            return (String::new(), String::new(), postal_code);
        }
    }
    
    // Standard format: "Street Number"
    // Last part should be the number if it contains any digit
    if let Some(last) = parts.last() {
        if last.chars().any(|c| c.is_ascii_digit()) {
            let street_number = last.to_string();
            let street = parts[..parts.len() - 1].join(" ");
            // No postal code in address string - will need to be matched by fuzzy matching
            return (street, street_number, String::new());
        }
    }
    
    // Fallback: treat entire string as street name
    (address.to_string(), String::new(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_debug_addresses() {
        let addresses = load_debug_addresses();
        // Should load addresses from embedded parquet
        assert!(!addresses.is_empty(), "Should load debug addresses");
        
        for addr in &addresses {
            assert!(!addr.street.is_empty() || !addr.street_number.is_empty(), 
                "Address should have street or number: {:?}", addr);
        }
    }
    
    #[test]
    fn test_parse_debug_address_standard() {
        let (street, number, postal) = parse_debug_address("Kornettsgatan 18C");
        assert_eq!(street, "Kornettsgatan");
        assert_eq!(number, "18C");
        assert_eq!(postal, "");
    }
    
    #[test]
    fn test_parse_debug_address_with_postal() {
        let (street, number, postal) = parse_debug_address("Idunsgatan 67B 214 46");
        assert_eq!(street, "Idunsgatan");
        assert_eq!(number, "67B");
        assert_eq!(postal, "21446");
    }
    
    #[test]
    fn test_parse_debug_address_complex() {
        let (street, number, postal) = parse_debug_address("Östra Kristinelundsvägen 27D");
        assert_eq!(street, "Östra Kristinelundsvägen");
        assert_eq!(number, "27D");
        assert_eq!(postal, "");
    }
    
    #[test]
    fn test_parse_debug_address_with_unit() {
        let (street, number, postal) = parse_debug_address("Lantmannagatan 50 U1");
        assert_eq!(street, "Lantmannagatan");
        // U1 contains a digit, so it's treated as the number
        assert_eq!(number, "U1");
        assert_eq!(postal, "");
    }
    
    #[test]
    fn test_debug_addresses_use_fuzzy_matching() {
        let addresses = load_debug_addresses();
        
        // At least some addresses should successfully match via fuzzy matching
        let valid_count = addresses.iter().filter(|a| a.valid).count();
        
        // We expect most real Malmö addresses to match
        // (the test data includes one fake address "Låssasgatan" that shouldn't match)
        assert!(
            valid_count > 0,
            "Expected at least some addresses to match via fuzzy matching, got {}/{}",
            valid_count,
            addresses.len()
        );
    }
}
