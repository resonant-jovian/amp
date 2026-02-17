//! Panel transition detection for notification triggering
//!
//! Tracks which time panel each address belongs to, detecting when
//! addresses move from one panel to another to trigger notifications.
//!
//! # State Management
//! Uses a global Mutex-protected HashMap to track the last known TimeBucket
//! for each address ID. State is persisted to `notification_state.parquet`
//! so that notifications are not re-fired after app restart.
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
use amp_core::parquet::{build_notification_state_parquet, read_notification_state_from_bytes};
use amp_core::structs::{NotificationStateEntry, DB};
use chrono::{Datelike, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
/// Global state tracking last known panel for each address
///
/// Maps address ID to its most recently observed TimeBucket.
/// Protected by Mutex for thread-safe access from UI and background tasks.
static PANEL_STATE: Mutex<Option<HashMap<usize, TimeBucket>>> = Mutex::new(None);
const NOTIFICATION_STATE_FILE_NAME: &str = "notification_state.parquet";
/// Convert a TimeBucket to its string representation for persistence
fn bucket_to_string(bucket: &TimeBucket) -> &'static str {
    match bucket {
        TimeBucket::Now => "Now",
        TimeBucket::Within6Hours => "Within6Hours",
        TimeBucket::Within1Day => "Within1Day",
        TimeBucket::Within1Month => "Within1Month",
        TimeBucket::MoreThan1Month => "MoreThan1Month",
        TimeBucket::Invalid => "Invalid",
    }
}
/// Convert a string back to a TimeBucket
fn bucket_from_string(s: &str) -> TimeBucket {
    match s {
        "Now" => TimeBucket::Now,
        "Within6Hours" => TimeBucket::Within6Hours,
        "Within1Day" => TimeBucket::Within1Day,
        "Within1Month" => TimeBucket::Within1Month,
        "MoreThan1Month" => TimeBucket::MoreThan1Month,
        _ => TimeBucket::Invalid,
    }
}
/// Get app-specific storage directory (same logic as settings.rs)
#[cfg(target_os = "android")]
fn get_storage_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("APP_FILES_DIR") {
        let path = PathBuf::from(dir);
        return Ok(path);
    }
    let app_dir = PathBuf::from("/data/local/tmp/amp_storage");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).map_err(|e| {
            format!(
                "[PanelTracker] Failed to create storage dir {:?}: {}",
                app_dir, e
            )
        })?;
    }
    Ok(app_dir)
}
#[cfg(not(target_os = "android"))]
fn get_storage_dir() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
}
/// Get the notification state file path
fn get_state_file_path() -> Result<PathBuf, String> {
    let mut path = get_storage_dir()?;
    path.push(NOTIFICATION_STATE_FILE_NAME);
    Ok(path)
}
/// Get current year-month as u32, e.g. 202602
fn current_year_month() -> u32 {
    let now = Utc::now();
    let swedish = now.with_timezone(&amp_core::structs::SWEDISH_TZ);
    (swedish.year() as u32) * 100 + swedish.month()
}
/// Load panel state from the persisted parquet file
///
/// Only loads entries matching the current year_month, so state
/// auto-resets each month.
fn load_panel_state_from_file() -> HashMap<usize, TimeBucket> {
    let path = match get_state_file_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[PanelTracker] Failed to get state file path: {}", e);
            return HashMap::new();
        }
    };
    if !path.exists() {
        eprintln!("[PanelTracker] No state file at {:?}, starting fresh", path);
        return HashMap::new();
    }
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[PanelTracker] Failed to read state file {:?}: {}", path, e);
            return HashMap::new();
        }
    };
    let entries = match read_notification_state_from_bytes(&bytes) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[PanelTracker] Failed to parse state parquet: {}", e);
            return HashMap::new();
        }
    };
    let ym = current_year_month();
    let mut map = HashMap::new();
    for entry in entries {
        if entry.year_month == ym {
            map.insert(entry.address_id as usize, bucket_from_string(&entry.bucket));
        }
    }
    eprintln!(
        "[PanelTracker] Loaded {} entries from file (year_month={})",
        map.len(),
        ym,
    );
    map
}
/// Save current panel state to parquet file
fn save_panel_state_to_file(state: &HashMap<usize, TimeBucket>) {
    if state.is_empty() {
        // Delete the file if state is empty
        if let Ok(path) = get_state_file_path()
            && path.exists()
        {
            let _ = std::fs::remove_file(&path);
        }
        return;
    }
    let ym = current_year_month();
    let entries: Vec<NotificationStateEntry> = state
        .iter()
        .map(|(id, bucket)| NotificationStateEntry {
            address_id: *id as u64,
            bucket: bucket_to_string(bucket).to_string(),
            year_month: ym,
        })
        .collect();
    let parquet_bytes = match build_notification_state_parquet(entries) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[PanelTracker] Failed to build state parquet: {}", e);
            return;
        }
    };
    let path = match get_state_file_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[PanelTracker] Failed to get state file path: {}", e);
            return;
        }
    };
    if let Some(parent) = path.parent()
        && !parent.exists()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "[PanelTracker] Failed to create directory {:?}: {}",
            parent, e
        );
        return;
    }
    if let Err(e) = std::fs::write(&path, parquet_bytes) {
        eprintln!("[PanelTracker] Failed to write state file {:?}: {}", path, e);
    } else {
        eprintln!("[PanelTracker] Saved {} entries to {:?}", state.len(), path);
    }
}
/// Initialize the panel state tracker
///
/// Loads persisted state from disk so that previously-seen transitions
/// are not re-fired. Safe to call multiple times (subsequent calls are no-ops).
///
/// # Examples
/// ```no_run
/// use amp_android::components::transitions::initialize_panel_tracker;
///
/// // Call during app startup
/// initialize_panel_tracker();
/// ```
pub fn initialize_panel_tracker() {
    let mut state = PANEL_STATE.lock().expect("Panel state poisoned");
    if state.is_none() {
        let loaded = load_panel_state_from_file();
        eprintln!(
            "[PanelTracker] Initialized with {} persisted entries",
            loaded.len(),
        );
        *state = Some(loaded);
    } else {
        eprintln!("[PanelTracker] Already initialized, skipping {:?}", state);
    }
}
/// Check for panel transitions and return list of addresses that changed panels
///
/// This function compares the current time bucket of each address against
/// its previously recorded state. When an address moves to a more urgent
/// bucket, it's included in the returned transitions list.
///
/// State is automatically saved to disk after detection so that
/// the same transitions are not re-fired on app restart.
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
        eprintln!("[PanelTracker] State not initialized, loading from file");
        *state_guard = Some(load_panel_state_from_file());
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
        let should_notify = matches!(
            (&previous_bucket, &new_bucket),
            (None, TimeBucket::Within1Day)
                | (None, TimeBucket::Within6Hours)
                | (None, TimeBucket::Now)
                | (Some(TimeBucket::MoreThan1Month), TimeBucket::Within1Day)
                | (Some(TimeBucket::MoreThan1Month), TimeBucket::Within6Hours)
                | (Some(TimeBucket::MoreThan1Month), TimeBucket::Now)
                | (Some(TimeBucket::Within1Month), TimeBucket::Within1Day)
                | (Some(TimeBucket::Within1Month), TimeBucket::Within6Hours)
                | (Some(TimeBucket::Within1Month), TimeBucket::Now)
                | (Some(TimeBucket::Within1Day), TimeBucket::Within6Hours)
                | (Some(TimeBucket::Within1Day), TimeBucket::Now)
                | (Some(TimeBucket::Within6Hours), TimeBucket::Now)
        );
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
    // Persist state after every detection pass
    save_panel_state_to_file(state);
    transitions
}
/// Clear the panel state (useful for testing or reset)
///
/// Removes all tracked address states and deletes the persisted file.
/// After calling this, the next call to `detect_transitions` will
/// treat all addresses as new.
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
    // Also delete the persisted file
    if let Ok(path) = get_state_file_path()
        && path.exists()
    {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!(
                "[PanelTracker] Failed to delete state file {:?}: {}",
                path, e
            );
        } else {
            eprintln!("[PanelTracker] Deleted state file {:?}", path);
        }
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
#[allow(dead_code)]
pub fn create_test_address_with_bucket(
    id: usize,
    day: u8,
    time: &str,
) -> (StoredAddress, TimeBucket) {
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
    let bucket = bucket_for(&db);
    let addr = StoredAddress {
        id,
        street: "Test Street".to_string(),
        street_number: id.to_string(),
        postal_code: "22100".to_string(),
        valid: true,
        active: true,
        matched_entry: Some(db),
        parking_info: None,
    };
    (addr, bucket)
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
            parking_info: None,
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
        let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
        if matches!(
            bucket,
            TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now
        ) {
            let transitions = detect_transitions(&[addr]);
            assert!(
                !transitions.is_empty(),
                "Should detect first occurrence for actionable bucket {:?}",
                bucket,
            );
        } else {
            eprintln!(
                "Skipping assertion: test address is in non-actionable bucket {:?}",
                bucket,
            );
        }
    }
    #[test]
    fn test_same_bucket_no_duplicate_notification() {
        clear_panel_state();
        initialize_panel_tracker();
        let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
        let first = detect_transitions(std::slice::from_ref(&addr));
        if first.is_empty() {
            eprintln!(
                "Skipping duplicate-notification assertion: \
             first detection did not trigger for bucket {:?}",
                bucket,
            );
            return;
        }
        let second = detect_transitions(&[addr]);
        assert_eq!(
            second.len(),
            0,
            "Same bucket should not trigger duplicate notification (bucket = {:?})",
            bucket,
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
            parking_info: None,
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
    #[test]
    fn test_bucket_string_roundtrip() {
        let buckets = vec![
            TimeBucket::Now,
            TimeBucket::Within6Hours,
            TimeBucket::Within1Day,
            TimeBucket::Within1Month,
            TimeBucket::MoreThan1Month,
            TimeBucket::Invalid,
        ];
        for bucket in buckets {
            let s = bucket_to_string(&bucket);
            let restored = bucket_from_string(s);
            assert_eq!(bucket, restored, "Roundtrip failed for {:?}", bucket);
        }
    }
    #[test]
    fn test_state_persists_across_restart() {
        // Clear everything
        clear_panel_state();
        initialize_panel_tracker();
        let addr = create_test_address(1, 1, "0800-1200");
        // First detection — may trigger transition
        let first = detect_transitions(&[addr.clone()]);
        let had_transition = !first.is_empty();
        // Simulate app restart: clear in-memory state but keep the file
        {
            let mut state = PANEL_STATE.lock().unwrap();
            *state = None;
        }
        // Re-initialize (should load from file)
        initialize_panel_tracker();
        // Second detection — should NOT re-fire the same transition
        let second = detect_transitions(&[addr]);
        if had_transition {
            assert!(
                second.is_empty(),
                "After restart, same bucket should not re-fire notification",
            );
        }
        // Cleanup
        clear_panel_state();
    }
    #[test]
    fn test_current_year_month() {
        let ym = current_year_month();
        assert!(ym >= 202001, "year_month should be >= 202001, got {}", ym);
        assert!(ym <= 210012, "year_month should be <= 210012, got {}", ym);
    }
}
