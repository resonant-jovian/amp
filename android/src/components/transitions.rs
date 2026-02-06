//! Panel transition detection for notification triggering
//!
//! Tracks which time panel each address belongs to, detecting when
//! addresses move from one panel to another to trigger notifications.
//!
//! # State Management
//! Uses a global Mutex-protected HashMap to track the last known TimeBucket
//! for each address ID. This ensures notifications only fire when an address
//! enters a new, more urgent panel.
//!
//! # Transition Rules
//! Notifications are sent when addresses move to more urgent panels:
//! - First detection in Within1Day/Within6Hours/Now → notify
//! - Within1Month → Within1Day → notify
//! - Within1Day → Within6Hours → notify
//! - Within6Hours → Now → notify
//!
//! # Examples
//! ```no_run
//! use amp_android::components::transitions::{initialize_panel_tracker, detect_transitions};
//! use amp_android::components::storage::read_addresses_from_device;
//!
//! // Initialize once on app startup
//! initialize_panel_tracker();
//!
//! // Check for transitions periodically (e.g., every 60 seconds)
//! let addresses = read_addresses_from_device();
//! let transitions = detect_transitions(&addresses);
//!
//! for (addr, prev, new) in transitions {
//!     println!("Address {} transitioned from {:?} to {:?}", addr.id, prev, new);
//! }
//! ```
use crate::components::countdown::{TimeBucket, bucket_for};
use crate::ui::StoredAddress;
use std::collections::HashMap;
use std::sync::Mutex;
/// Global state tracking last known panel for each address
///
/// Maps address ID to its most recently observed TimeBucket.
/// Protected by Mutex for thread-safe access from UI and background tasks.
static PANEL_STATE: Mutex<Option<HashMap<usize, TimeBucket>>> = Mutex::new(None);
/// Initialize the panel state tracker
///
/// Call this once during app initialization to set up the state HashMap.
/// Safe to call multiple times (subsequent calls are no-ops).
///
/// # Examples
/// ```no_run
/// use amp_android::components::transitions::initialize_panel_tracker;
///
/// // Call during app startup
/// initialize_panel_tracker();
/// ```
pub fn initialize_panel_tracker() {
    let mut state = PANEL_STATE.lock().unwrap();
    if state.is_none() {
        *state = Some(HashMap::new());
        eprintln!("[PanelTracker] Initialized state tracking");
    } else {
        eprintln!("[PanelTracker] Already initialized, skipping");
    }
}
/// Check for panel transitions and return list of addresses that changed panels
///
/// This function compares the current time bucket of each address against
/// its previously recorded state. When an address moves to a more urgent
/// bucket, it's included in the returned transitions list.
///
/// This should be called:
/// - On app startup (to detect any addresses that became urgent while app was closed)
/// - Periodically (e.g., every 60 seconds) to catch time-based transitions
/// - After loading addresses from storage
/// - After updating address data
///
/// # Arguments
/// * `addresses` - Current list of addresses with their matched parking data
///
/// # Returns
/// Vector of tuples containing:
/// - `StoredAddress` - The address that transitioned
/// - `Option<TimeBucket>` - Previous bucket (None if first time seeing address)
/// - `TimeBucket` - New bucket that triggered the notification
///
/// # Examples
/// ```no_run
/// use amp_android::components::transitions::detect_transitions;
/// use amp_android::components::storage::read_addresses_from_device;
/// use amp_android::components::countdown::TimeBucket;
///
/// let addresses = read_addresses_from_device();
/// let transitions = detect_transitions(&addresses);
///
/// for (addr, prev, new) in transitions {
///     match new {
///         TimeBucket::Within1Day => println!("1-day warning for {}", addr.street),
///         TimeBucket::Within6Hours => println!("6-hour warning for {}", addr.street),
///         TimeBucket::Now => println!("Active now: {}", addr.street),
///         _ => {}
///     }
/// }
/// ```
pub fn detect_transitions(
    addresses: &[StoredAddress],
) -> Vec<(StoredAddress, Option<TimeBucket>, TimeBucket)> {
    let mut state_guard = PANEL_STATE.lock().unwrap();
    if state_guard.is_none() {
        eprintln!("[PanelTracker] State not initialized, initializing now");
        *state_guard = Some(HashMap::new());
    }
    let state = state_guard.as_mut().unwrap();
    let mut transitions = Vec::new();
    for addr in addresses {
        let matched_entry = match &addr.matched_entry {
            Some(entry) => entry,
            None => {
                state.remove(&addr.id);
                continue;
            }
        };
        let new_bucket = bucket_for(matched_entry);
        let previous_bucket = state.get(&addr.id).cloned();
        let should_notify = match (&previous_bucket, &new_bucket) {
            (None, TimeBucket::Within1Day) => true,
            (None, TimeBucket::Within6Hours) => true,
            (None, TimeBucket::Now) => true,
            (Some(TimeBucket::MoreThan1Month), TimeBucket::Within1Day) => true,
            (Some(TimeBucket::MoreThan1Month), TimeBucket::Within6Hours) => true,
            (Some(TimeBucket::MoreThan1Month), TimeBucket::Now) => true,
            (Some(TimeBucket::Within1Month), TimeBucket::Within1Day) => true,
            (Some(TimeBucket::Within1Month), TimeBucket::Within6Hours) => true,
            (Some(TimeBucket::Within1Month), TimeBucket::Now) => true,
            (Some(TimeBucket::Within1Day), TimeBucket::Within6Hours) => true,
            (Some(TimeBucket::Within1Day), TimeBucket::Now) => true,
            (Some(TimeBucket::Within6Hours), TimeBucket::Now) => true,
            _ => false,
        };
        if should_notify {
            eprintln!(
                "[PanelTracker] Transition detected: {} {} (id={}) {:?} → {:?}",
                addr.street, addr.street_number, addr.id, previous_bucket, new_bucket,
            );
            transitions.push((addr.clone(), previous_bucket, new_bucket.clone()));
        }
        state.insert(addr.id, new_bucket);
    }
    if !transitions.is_empty() {
        eprintln!(
            "[PanelTracker] Detected {} transition(s) requiring notifications",
            transitions.len(),
        );
    }
    transitions
}
/// Clear the panel state (useful for testing or reset)
///
/// Removes all tracked address states. After calling this, the next
/// call to `detect_transitions` will treat all addresses as new.
///
/// # Examples
/// ```no_run
/// use amp_android::components::transitions::clear_panel_state;
///
/// // Clear all tracked state
/// clear_panel_state();
/// ```
#[allow(dead_code)]
pub fn clear_panel_state() {
    let mut state = PANEL_STATE.lock().unwrap();
    if let Some(map) = state.as_mut() {
        let count = map.len();
        map.clear();
        eprintln!("[PanelTracker] Cleared {} tracked address(es)", count);
    } else {
        eprintln!("[PanelTracker] State not initialized, nothing to clear");
    }
}
/// Get the number of currently tracked addresses
///
/// Returns the count of addresses being tracked in the panel state.
/// Useful for debugging and monitoring.
#[allow(dead_code)]
pub fn tracked_address_count() -> usize {
    let state = PANEL_STATE.lock().unwrap();
    state.as_ref().map_or(0, |map| map.len())
}
#[cfg(test)]
mod tests {
    use super::*;
    use amp_core::structs::DB;
    /// Helper to create a test address with a specific day/time
    fn create_test_address(id: usize, day: u8, time: &str) -> StoredAddress {
        let db = DB::from_dag_tid(
            Some("22100".to_string()),
            format!("Test Street {}", id),
            Some("Test Street".to_string()),
            Some(id.to_string()),
            None,
            day,
            time,
            None,
            None,
            None,
            2024,
            1,
        )
        .expect("Failed to create test DB entry");
        StoredAddress {
            id,
            street: "Test Street".to_string(),
            street_number: id.to_string(),
            postal_code: "22100".to_string(),
            valid: true,
            active: true,
            matched_entry: Some(db),
        }
    }
    #[test]
    fn test_initialize_panel_tracker() {
        clear_panel_state();
        initialize_panel_tracker();
        assert_eq!(tracked_address_count(), 0);
    }
    #[test]
    fn test_first_detection_in_actionable_bucket() {
        clear_panel_state();
        initialize_panel_tracker();
        let addr = create_test_address(1, 1, "0800-1200");
        let transitions = detect_transitions(&[addr]);
        assert!(!transitions.is_empty(), "Should detect first occurrence");
    }
    #[test]
    fn test_same_bucket_no_duplicate_notification() {
        clear_panel_state();
        initialize_panel_tracker();
        let addr = create_test_address(1, 1, "0800-1200");
        let first = detect_transitions(&[addr.clone()]);
        assert!(!first.is_empty(), "First detection should trigger");
        let second = detect_transitions(&[addr]);
        assert_eq!(
            second.len(),
            0,
            "Same bucket should not trigger duplicate notification",
        );
    }
    #[test]
    fn test_address_without_match_ignored() {
        clear_panel_state();
        initialize_panel_tracker();
        let addr = StoredAddress {
            id: 99,
            street: "No Match Street".to_string(),
            street_number: "1".to_string(),
            postal_code: "99999".to_string(),
            valid: false,
            active: false,
            matched_entry: None,
        };
        let transitions = detect_transitions(&[addr]);
        assert_eq!(
            transitions.len(),
            0,
            "Addresses without matches should not trigger notifications",
        );
    }
    #[test]
    fn test_tracked_count_increases() {
        clear_panel_state();
        initialize_panel_tracker();
        let addr1 = create_test_address(1, 1, "0800-1200");
        let addr2 = create_test_address(2, 2, "0800-1200");
        detect_transitions(&[addr1]);
        assert_eq!(tracked_address_count(), 1);
        detect_transitions(&[addr2]);
        assert_eq!(tracked_address_count(), 2);
    }
    #[test]
    fn test_clear_state() {
        clear_panel_state();
        initialize_panel_tracker();
        let addr = create_test_address(1, 1, "0800-1200");
        detect_transitions(&[addr]);
        assert!(tracked_address_count() > 0);
        clear_panel_state();
        assert_eq!(tracked_address_count(), 0);
    }
}
