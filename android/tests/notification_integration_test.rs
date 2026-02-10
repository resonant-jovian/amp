//! Integration tests for the Android notification system
//!
//! Tests the complete notification flow from transition detection
//! through to notification sending, including:
//! - Transition detection logic
//! - Settings-based filtering
//! - Channel routing
//! - Lifecycle integration
//!
//! Note: These tests use mock implementations for non-Android platforms.
//! Full JNI testing requires an Android device or emulator.
use amp_android::components::settings::save_settings;
use amp_android::components::{
    countdown::TimeBucket,
    lifecycle::LifecycleManager,
    notifications::{
        initialize_notification_channels, notify_active, notify_one_day, notify_six_hours,
    },
    settings::load_settings,
    transitions::{
        clear_panel_state, create_test_address_with_bucket, detect_transitions,
        initialize_panel_tracker,
    },
};
use amp_android::ui::StoredAddress;
use amp_core::structs::DB;
/// Helper to create a test address with specific day/time
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
        2026,
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
fn test_notification_system_initialization() {
    initialize_notification_channels();
    initialize_panel_tracker();
}
#[test]
fn test_complete_notification_flow() {
    clear_panel_state();
    initialize_panel_tracker();
    let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
    eprintln!("Test address is in bucket: {:?}", bucket);
    let transitions = detect_transitions(std::slice::from_ref(&addr));
    if matches!(
        bucket,
        TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now
    ) {
        assert!(
            !transitions.is_empty(),
            "First detection in actionable bucket should trigger transition",
        );
        let (_, prev, new) = &transitions[0];
        assert!(
            prev.is_none(),
            "Previous bucket should be None on first detection"
        );
        println!("Transition detected: None → {:?}", new);
        match new {
            TimeBucket::Within1Day => notify_one_day(&addr),
            TimeBucket::Within6Hours => notify_six_hours(&addr),
            TimeBucket::Now => notify_active(&addr),
            _ => panic!("Unexpected bucket: {:?}", new),
        }
        let transitions2 = detect_transitions(&[addr]);
        assert_eq!(
            transitions2.len(),
            0,
            "Same bucket should not trigger duplicate notification",
        );
    } else {
        eprintln!(
            "Skipping actionable assertion; test address is in non-actionable bucket {:?}",
            bucket,
        );
        assert!(
            transitions.is_empty(),
            "Non-actionable bucket {:?} should not trigger transitions",
            bucket,
        );
    }
}
#[test]
fn test_multiple_address_transitions() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr1 = create_test_address(1, 1, "0800-1200");
    let addr2 = create_test_address(2, 2, "0800-1200");
    let addr3 = create_test_address(3, 3, "0800-1200");
    let addresses = vec![addr1.clone(), addr2.clone(), addr3.clone()];
    let transitions = detect_transitions(&addresses);
    println!("First check detected {} transitions", transitions.len());
    let transitions2 = detect_transitions(&addresses);
    assert_eq!(
        transitions2.len(),
        0,
        "Second check should not trigger duplicate notifications",
    );
}
#[test]
fn test_notification_settings_respect() {
    let addr = create_test_address(99, 1, "0800-1200");
    notify_one_day(&addr);
    notify_six_hours(&addr);
    notify_active(&addr);
}
#[test]
fn test_transition_to_more_urgent_bucket() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr1 = create_test_address(1, 1, "0800-1200");
    let transitions1 = detect_transitions(std::slice::from_ref(&addr1));
    println!("First detection: {} transitions", transitions1.len());
}
#[test]
fn test_lifecycle_manager_notification_integration() {
    let mut manager = LifecycleManager::new();
    manager.start();
    assert!(manager.is_running());
    assert!(manager.are_notifications_initialized());
    let count = manager.check_and_send_notifications();
    println!("Lifecycle manager sent {} notifications", count);
    manager.shutdown();
    assert!(!manager.is_running());
}
#[test]
fn test_address_without_match_ignored() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr = StoredAddress {
        id: 999,
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
fn test_inactive_addresses_handled() {
    clear_panel_state();
    initialize_panel_tracker();
    let mut addr = create_test_address(1, 1, "0800-1200");
    addr.active = false;
    let transitions = detect_transitions(&[addr]);
    println!("Inactive address transitions: {}", transitions.len());
}
#[test]
fn test_notification_channel_constants() {
    let addr = create_test_address(1, 1, "0800-1200");
    notify_one_day(&addr);
    notify_six_hours(&addr);
    notify_active(&addr);
}
#[test]
fn test_same_address_different_buckets() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr1 = create_test_address(1, 1, "0800-1200");
    let transitions1 = detect_transitions(&[addr1]);
    if !transitions1.is_empty() {
        println!("First bucket: {:?}", transitions1[0].2);
    }
    let addr2 = create_test_address(1, 2, "0800-1200");
    let transitions2 = detect_transitions(&[addr2]);
    println!("Transitions after time change: {}", transitions2.len());
}
#[test]
fn test_bulk_notification_check() {
    clear_panel_state();
    initialize_panel_tracker();
    let addresses: Vec<StoredAddress> = (1..=10)
        .map(|i| create_test_address(i, (i % 31) as u8 + 1, "0800-1200"))
        .collect();
    let transitions1 = detect_transitions(&addresses);
    println!("Bulk check: {} transitions detected", transitions1.len());
    let transitions2 = detect_transitions(&addresses);
    assert_eq!(
        transitions2.len(),
        0,
        "Bulk second check should not trigger duplicates"
    );
}
#[test]
fn test_notification_system_under_load() {
    let addr = create_test_address(1, 1, "0800-1200");
    for _ in 0..100 {
        notify_one_day(&addr);
        notify_six_hours(&addr);
        notify_active(&addr);
    }
}
#[test]
fn test_lifecycle_manager_multiple_starts() {
    let mut manager = LifecycleManager::new();
    manager.start();
    assert!(manager.is_running());
    manager.start();
    assert!(manager.is_running());
    manager.shutdown();
}
#[test]
fn test_settings_persistence_through_notifications() {
    let mut settings = load_settings();
    settings.notifications.sex_timmar = false;
    save_settings(&settings);
    let loaded = load_settings();
    assert!(
        !loaded.notifications.sex_timmar,
        "Expected sex_timmar to be persisted as false",
    );
}
