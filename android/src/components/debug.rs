//! Debug mode functionality for loading example addresses
//!
//! Provides a read-only debug mode that loads addresses from a bundled debug.parquet file.
//! Useful for testing and development without affecting user data.
//!
//! # Features
//! - Loads debug addresses from embedded debug.parquet asset
//! - Read-only: never writes to storage
//! - Can be toggled on/off via settings
//! - Uses same address switching (active/inactive) as regular addresses
//!
//! # Usage
//! ```no_run
//! use amp_android::components::debug;
//!
//! // Load debug addresses
//! let debug_addresses = debug::load_debug_addresses();
//!
//! // Toggle active state (in-memory only, not persisted)
//! let mut addresses = debug_addresses;
//! if let Some(addr) = addresses.iter_mut().find(|a| a.id == some_id) {
//!     addr.active = !addr.active;
//! }
//! ```

use crate::ui::StoredAddress;
use amp_core::parquet::read_local_parquet_from_bytes;
use amp_core::structs::LocalData;

/// Debug parquet file embedded in the app
static DEBUG_PARQUET: &[u8] = include_bytes!("../../assets/data/debug.parquet");

/// Load debug addresses from embedded debug.parquet file
///
/// Reads the debug.parquet file that contains example addresses for testing.
/// These addresses are read-only and never written back to storage.
///
/// # Returns
/// Vector of debug addresses, empty if loading fails
///
/// # Examples
/// ```no_run
/// use amp_android::components::debug;
///
/// let debug_addrs = debug::load_debug_addresses();
/// println!("Loaded {} debug addresses", debug_addrs.len());
/// ```
pub fn load_debug_addresses() -> Vec<StoredAddress> {
    match read_local_parquet_from_bytes(DEBUG_PARQUET) {
        Ok(local_data) => {
            eprintln!(
                "[Debug] Loaded {} debug addresses from embedded parquet",
                local_data.len(),
            );
            local_data
                .into_iter()
                .enumerate()
                .filter(|(_, data)| !data.adress.is_empty())
                .map(|(idx, data)| from_local_data(data, idx))
                .collect()
        }
        Err(e) => {
            eprintln!("[Debug] Failed to load debug addresses: {}", e);
            Vec::new()
        }
    }
}

/// Extract postal code from address string
///
/// Debug.parquet addresses are in format: "Street Number PostalCode"
/// e.g., "Idunsgatan 67B 214 46" where "214 46" is the postal code
///
/// This function extracts the last 5 digits (removing spaces) as the postal code.
fn extract_postal_code_from_address(adress: &str) -> Option<String> {
    let parts: Vec<&str> = adress.split_whitespace().collect();
    
    // Try to find postal code in last 2 parts (handles "214 46" format)
    if parts.len() >= 2 {
        // Try last two parts combined (e.g., "214" + "46" = "21446")
        let last_two = format!("{}{}", parts[parts.len()-2], parts[parts.len()-1]);
        
        // Check if it's all digits and 5 characters
        if last_two.chars().all(|c| c.is_ascii_digit()) && last_two.len() == 5 {
            return Some(last_two);
        }
    }
    
    None
}

/// Convert LocalData from parquet to StoredAddress
///
/// This follows the same logic as storage.rs but adds special handling for
/// debug.parquet where postal codes are embedded in the address string.
///
/// The matched_entry is set to None here - validation happens later via
/// check_and_update_validity() when addresses are loaded in the UI.
fn from_local_data(data: LocalData, id: usize) -> StoredAddress {
    let (street, street_number) = if let Some(gata) = &data.gata {
        let street_number = data.gatunummer.clone().unwrap_or_default();
        (gata.clone(), street_number)
    } else {
        let parts: Vec<&str> = data.adress.rsplitn(2, ' ').collect();
        if parts.len() == 2 {
            (parts[1].to_string(), parts[0].to_string())
        } else {
            (data.adress.clone(), String::new())
        }
    };
    
    // Extract postal code: use postnummer if present, otherwise extract from address
    let postal_code = if let Some(ref pc) = data.postnummer {
        if !pc.is_empty() {
            pc.clone()
        } else {
            extract_postal_code_from_address(&data.adress).unwrap_or_default()
        }
    } else {
        extract_postal_code_from_address(&data.adress).unwrap_or_default()
    };
    
    StoredAddress {
        id,
        street,
        street_number,
        postal_code,
        valid: data.valid,
        active: data.active,
        matched_entry: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_debug_addresses() {
        let addresses = load_debug_addresses();
        for addr in &addresses {
            assert!(!addr.street.is_empty(), "Address should have a street name");
        }
    }

    #[test]
    fn test_extract_postal_code_from_address() {
        assert_eq!(
            extract_postal_code_from_address("Idunsgatan 67B 214 46"),
            Some("21446".to_string())
        );
        assert_eq!(
            extract_postal_code_from_address("Kornettsgatan 18C 211 50"),
            Some("21150".to_string())
        );
        assert_eq!(
            extract_postal_code_from_address("Storgatan 10"),
            None
        );
    }

    #[test]
    fn test_debug_addresses_have_postal_codes() {
        let addresses = load_debug_addresses();
        for addr in &addresses {
            if !addr.postal_code.is_empty() {
                assert_eq!(
                    addr.postal_code.len(),
                    5,
                    "Postal code should be 5 digits: {}",
                    addr.postal_code,
                );
                assert!(
                    addr.postal_code.chars().all(|c| c.is_ascii_digit()),
                    "Postal code should be all digits: {}",
                    addr.postal_code,
                );
            }
        }
    }
}
