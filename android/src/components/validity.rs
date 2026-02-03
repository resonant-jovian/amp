//! Address validity checking for date-dependent parking restrictions
//!
//! Handles validation of addresses where parking restrictions depend on specific days
//! of the month. Special attention to days 29 and 30 which may not exist in February.
//!
//! # Validity Rules
//!
//! - Addresses with `dag` (day) field set to 1-28: Always valid
//! - Addresses with `dag` 29: Valid in all months except non-leap-year February
//! - Addresses with `dag` 30: Valid in all months except February
//! - Addresses with `dag` 31: Valid only in months with 31 days
//!
//! # When to Check
//!
//! Validity should be checked:
//! - Once per day (background task)
//! - When month changes (detected by comparing stored month with current month)
//! - After adding/removing addresses
//! - On app startup
//!
//! # Examples
//!
//! ```no_run
//! use amp_android::components::validity::check_and_update_validity;
//! use amp_android::ui::StoredAddress;
//!
//! let mut addresses = vec![
//!     StoredAddress {
//!         id: 1,
//!         street: "Storgatan".to_string(),
//!         street_number: "10".to_string(),
//!         postal_code: "22100".to_string(),
//!         valid: true,
//!         active: true,
//!         matched_entry: None, // Would have dag: Some(30)
//!     },
//! ];
//!
//! // Returns true if any address validity changed
//! if check_and_update_validity(&mut addresses) {
//!     println!("Some addresses changed validity");
//! }
//! ```

use crate::ui::StoredAddress;
use chrono::{Datelike, Local};

/// Check if the current year is a leap year
///
/// # Arguments
/// * `year` - Year to check (e.g., 2024)
///
/// # Returns
/// `true` if the year is a leap year, `false` otherwise
///
/// # Examples
/// ```
/// assert!(is_leap_year(2024));
/// assert!(!is_leap_year(2023));
/// assert!(is_leap_year(2000));
/// assert!(!is_leap_year(1900));
/// ```
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a given month
///
/// # Arguments
/// * `month` - Month number (1-12)
/// * `year` - Year (for February leap year detection)
///
/// # Returns
/// Number of days in the month (28-31)
///
/// # Panics
/// Panics if month is not in range 1-12
pub fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month: {}", month),
    }
}

/// Check if an address should be valid in the current month
///
/// # Arguments
/// * `dag` - Day of month for the parking restriction (1-31)
///
/// # Returns
/// `true` if the restriction day exists in current month, `false` otherwise
///
/// # Examples
/// ```no_run
/// // In February 2023 (non-leap year):
/// assert!(is_valid_in_current_month(Some(28)));
/// assert!(!is_valid_in_current_month(Some(29)));
/// assert!(!is_valid_in_current_month(Some(30)));
///
/// // In January:
/// assert!(is_valid_in_current_month(Some(31)));
/// ```
pub fn is_valid_in_current_month(dag: Option<u8>) -> bool {
    let dag = match dag {
        Some(d) => d as u32,
        None => return true, // No day restriction = always valid
    };
    
    let now = Local::now();
    let current_month = now.month();
    let current_year = now.year();
    
    let max_days = days_in_month(current_month, current_year);
    
    dag <= max_days
}

/// Check and update validity for all addresses based on current month
///
/// Iterates through all addresses and updates their `valid` field if their
/// restriction day doesn't exist in the current month.
///
/// # Arguments
/// * `addresses` - Mutable slice of addresses to check and update
///
/// # Returns
/// `true` if any address validity changed, `false` otherwise
///
/// # Side Effects
/// Modifies the `valid` field of addresses in the vector
///
/// # Examples
/// ```no_run
/// use amp_android::components::validity::check_and_update_validity;
///
/// let mut addresses = get_addresses();
/// if check_and_update_validity(&mut addresses) {
///     // Save updated addresses
///     save_addresses(&addresses);
/// }
/// ```
pub fn check_and_update_validity(addresses: &mut [StoredAddress]) -> bool {
    let mut changed = false;
    
    for addr in addresses.iter_mut() {
        // Extract dag from matched_entry if available
        let dag = addr.matched_entry.as_ref().and_then(|entry| {
            // Extract day from start_time
            // Since DB stores DateTime, we need to get the day component
            Some(entry.start_time_swedish().day() as u8)
        });
        
        // Check if this address should be valid in current month
        let should_be_valid = is_valid_in_current_month(dag);
        
        // Only consider addresses that have a matched entry
        // Addresses without matches remain in their current valid state
        if addr.matched_entry.is_some() && addr.valid != should_be_valid {
            eprintln!(
                "[Validity] Address {} {} validity changed: {} -> {}",
                addr.street,
                addr.street_number,
                addr.valid,
                should_be_valid
            );
            addr.valid = should_be_valid;
            changed = true;
        }
    }
    
    if changed {
        eprintln!("[Validity] Address validity updated for current month");
    }
    
    changed
}

/// Get current month number (1-12)
pub fn current_month() -> u32 {
    Local::now().month()
}

/// Get current year
pub fn current_year() -> i32 {
    Local::now().year()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(1900));
    }
    
    #[test]
    fn test_days_in_month() {
        // January (31 days)
        assert_eq!(days_in_month(1, 2023), 31);
        
        // February (28 in non-leap year)
        assert_eq!(days_in_month(2, 2023), 28);
        
        // February (29 in leap year)
        assert_eq!(days_in_month(2, 2024), 29);
        
        // April (30 days)
        assert_eq!(days_in_month(4, 2023), 30);
        
        // December (31 days)
        assert_eq!(days_in_month(12, 2023), 31);
    }
    
    #[test]
    fn test_validity_no_restriction() {
        // Addresses with no day restriction are always valid
        assert!(is_valid_in_current_month(None));
    }
}
