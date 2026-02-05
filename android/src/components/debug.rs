//! Debug mode functionality for loading example addresses
//!
//! Provides a read-only debug mode that loads addresses from a bundled debug.parquet file.
//! Useful for testing and development without affecting user data.
//!
//! # Features
//! - Loads debug addresses from embedded debug.parquet asset (minimal format with address + postal code)
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
/// Contains 'adress' and 'postnummer' fields - all other fields are NULL
/// This mimics user input via "Add Address" button
static DEBUG_PARQUET: &[u8] = include_bytes!("../../assets/data/debug.parquet");
/// Load debug addresses from embedded debug.parquet file
///
/// Reads the minimal debug.parquet file that contains address strings and postal codes.
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
        Ok(debug_addresses) => {
            eprintln!(
                "[Debug] Successfully loaded {} debug addresses from minimal parquet",
                debug_addresses.len(),
            );
            let ui_addresses: Vec<StoredAddress> = debug_addresses
                .into_iter()
                .map(|debug_addr| {
                    let (street, street_number) = parse_address(&debug_addr.adress);
                    eprintln!(
                        "[Debug] Creating address: '{}' + '{}' (postal: {})",
                        street, street_number, debug_addr.postnummer,
                    );
                    StoredAddress::new(street, street_number, debug_addr.postnummer)
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
/// Parse address string into street name and street number
///
/// Addresses are in format: "Kornettsgatan 18C" or "Östra Kristinelundsvägen 27D"
/// This function splits the address into:
/// - Street name: all words except the last (if last contains digits)
/// - Street number: the last word (if it contains digits)
///
/// # Arguments
/// * `address` - Full address string
///
/// # Returns
/// Tuple of (street_name, street_number)
///
/// # Examples
/// ```
/// let (street, number) = parse_address("Kornettsgatan 18C");
/// assert_eq!(street, "Kornettsgatan");
/// assert_eq!(number, "18C");
///
/// let (street, number) = parse_address("Lantmannagatan 50 U1");
/// assert_eq!(street, "Lantmannagatan");
/// assert_eq!(number, "50 U1");
/// ```
fn parse_address(address: &str) -> (String, String) {
    let parts: Vec<&str> = address.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), String::new());
    }
    let mut number_start_idx = None;
    for (i, part) in parts.iter().enumerate() {
        if part.chars().any(|c| c.is_ascii_digit()) {
            number_start_idx = Some(i);
            break;
        }
    }
    match number_start_idx {
        Some(idx) if idx > 0 => {
            let street = parts[..idx].join(" ");
            let street_number = parts[idx..].join(" ");
            (street, street_number)
        }
        Some(_) => (String::new(), address.to_string()),
        None => (address.to_string(), String::new()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_debug_addresses() {
        let addresses = load_debug_addresses();
        assert!(!addresses.is_empty(), "Should load debug addresses");
        for addr in &addresses {
            assert!(
                !addr.street.is_empty() || !addr.street_number.is_empty(),
                "Address should have street or number: {:?}",
                addr,
            );
        }
    }
    #[test]
    fn test_parse_address_simple() {
        let (street, number) = parse_address("Kornettsgatan 18C");
        assert_eq!(street, "Kornettsgatan");
        assert_eq!(number, "18C");
    }
    #[test]
    fn test_parse_address_with_unit() {
        let (street, number) = parse_address("Lantmannagatan 50 U1");
        assert_eq!(street, "Lantmannagatan");
        assert_eq!(number, "50 U1");
    }
    #[test]
    fn test_parse_address_multi_word_street() {
        let (street, number) = parse_address("Östra Kristinelundsvägen 27D");
        assert_eq!(street, "Östra Kristinelundsvägen");
        assert_eq!(number, "27D");
    }
    #[test]
    fn test_parse_address_complex_unit() {
        let (street, number) = parse_address("Celsiusgatan 13A U1");
        assert_eq!(street, "Celsiusgatan");
        assert_eq!(number, "13A U1");
    }
    #[test]
    fn test_parse_address_complex_unit_2() {
        let (street, number) = parse_address("Östergårdsgatan 1 U13");
        assert_eq!(street, "Östergårdsgatan");
        assert_eq!(number, "1 U13");
    }
    #[test]
    fn test_parse_address_colon_in_name() {
        let (street, number) = parse_address("S:t Pauli kyrkogata 13B");
        assert_eq!(street, "S:t Pauli kyrkogata");
        assert_eq!(number, "13B");
    }
    #[test]
    fn test_parse_address_no_number() {
        let (street, number) = parse_address("Storgatan");
        assert_eq!(street, "Storgatan");
        assert_eq!(number, "");
    }
    #[test]
    fn test_debug_addresses_use_fuzzy_matching() {
        let addresses = load_debug_addresses();
        let valid_count = addresses.iter().filter(|a| a.valid).count();
        assert!(
            valid_count > 0,
            "Expected at least some addresses to match via fuzzy matching, got {}/{}",
            valid_count,
            addresses.len(),
        );
    }
    #[test]
    fn test_debug_addresses_have_postal_codes() {
        let addresses = load_debug_addresses();
        let with_postal = addresses
            .iter()
            .filter(|a| !a.postal_code.is_empty())
            .count();
        assert!(
            with_postal as f64 / addresses.len() as f64 > 0.9,
            "Expected >90% of addresses to have postal codes, got {}/{}",
            with_postal,
            addresses.len(),
        );
    }
}
