//! Integration tests for the notification system
//!
//! These tests verify the complete flow from address transitions
//! to notification sending, including:
//! - Transition detection
//! - Notification triggering
//! - Settings respect
//! - State persistence
//!
//! Run with: `cargo test --test notification_integration_tests`
use amp_android::components::countdown::{TimeBucket, bucket_for};
use amp_android::components::settings::load_settings;
use amp_android::components::transitions::{
    clear_panel_state, detect_transitions, initialize_panel_tracker,
};
use amp_android::ui::StoredAddress;
use amp_core::structs::DB;
/// Helper to create a test address with a specific day and time
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
/// Helper to count expected notifications for a set of transitions
fn count_expected_notifications(
    transitions: &[(StoredAddress, Option<TimeBucket>, TimeBucket)],
) -> usize {
    transitions
        .iter()
        .filter(|(_, _, new_bucket)| {
            matches!(
                new_bucket,
                TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now
            )
        })
        .count()
}
#[test]
fn test_first_time_address_detection() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr = create_test_address(1, 1, "0800-1200");
    let transitions = detect_transitions(&[addr]);
    assert!(
        !transitions.is_empty(),
        "First time seeing address in actionable bucket should trigger transition",
    );
    assert_eq!(count_expected_notifications(&transitions), 1);
}
#[test]
fn test_no_duplicate_notifications() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr = create_test_address(1, 1, "0800-1200");
    let first_transitions = detect_transitions(std::slice::from_ref(&addr));
    assert!(
        !first_transitions.is_empty(),
        "First detection should trigger"
    );
    let second_transitions = detect_transitions(&[addr]);
    assert_eq!(
        second_transitions.len(),
        0,
        "Same bucket should not trigger duplicate notification",
    );
}
#[test]
fn test_multiple_addresses_independent_tracking() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr1 = create_test_address(1, 1, "0800-1200");
    let addr2 = create_test_address(2, 2, "0800-1200");
    let transitions1 = detect_transitions(std::slice::from_ref(&addr1));
    assert_eq!(transitions1.len(), 1, "First address should trigger");
    let transitions2 = detect_transitions(&[addr1.clone(), addr2.clone()]);
    assert_eq!(transitions2.len(), 1, "Only new address should trigger");
    let transitions3 = detect_transitions(&[addr1, addr2]);
    assert_eq!(
        transitions3.len(),
        0,
        "No transitions when both addresses stay in same buckets",
    );
}
#[test]
fn test_address_without_match_ignored() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr_no_match = StoredAddress {
        id: 99,
        street: "No Match Street".to_string(),
        street_number: "1".to_string(),
        postal_code: "99999".to_string(),
        valid: false,
        active: false,
        matched_entry: None,
    };
    let transitions = detect_transitions(&[addr_no_match]);
    assert_eq!(
        transitions.len(),
        0,
        "Addresses without matches should not trigger notifications",
    );
}
#[test]
fn test_transition_to_more_urgent_bucket() {
    clear_panel_state();
    initialize_panel_tracker();
    let mut addr = create_test_address(1, 15, "0800-1200");
    detect_transitions(std::slice::from_ref(&addr));
    let db_1day = DB::from_dag_tid(
        Some("22100".to_string()),
        "Test Street 1".to_string(),
        Some("Test Street".to_string()),
        Some("1".to_string()),
        None,
        1,
        "0800-1200",
        None,
        None,
        None,
        2024,
        1,
    )
    .unwrap();
    addr.matched_entry = Some(db_1day);
    let transitions = detect_transitions(&[addr]);
    assert!(
        !transitions.is_empty(),
        "Transition to more urgent bucket should trigger notification",
    );
}
#[test]
fn test_bucket_categorization() {
    let addr_tomorrow = create_test_address(1, 1, "0800-1200");
    let bucket = bucket_for(addr_tomorrow.matched_entry.as_ref().unwrap());
    assert!(
        matches!(
            bucket,
            TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now
        ),
        "Address tomorrow should be in actionable bucket, got {:?}",
        bucket,
    );
}
#[test]
fn test_settings_integration() {
    let settings = load_settings();
    assert!(
        settings.notifications.stadning_nu,
        "Settings should have stadning_nu field",
    );
    assert!(
        settings.notifications.sex_timmar,
        "Settings should have sex_timmar field"
    );
    assert!(
        settings.notifications.en_dag,
        "Settings should have en_dag field"
    );
}
#[test]
fn test_lifecycle_manager_integration() {
    use amp_android::components::lifecycle::LifecycleManager;
    let mut manager = LifecycleManager::new();
    assert!(!manager.is_running(), "New manager should not be running");
    assert!(
        !manager.are_notifications_initialized(),
        "Notifications should not be initialized yet",
    );
    manager.start();
    assert!(manager.is_running(), "Started manager should be running");
    assert!(
        manager.are_notifications_initialized(),
        "Notifications should be initialized after start",
    );
    let count = manager.check_and_send_notifications();
    assert_eq!(
        count, 0,
        "Test environment should have no addresses to notify about"
    );
    manager.shutdown();
    assert!(
        !manager.is_running(),
        "Shutdown manager should not be running"
    );
}
#[test]
fn test_notification_flow_end_to_end() {
    use amp_android::components::lifecycle::LifecycleManager;
    clear_panel_state();
    let mut manager = LifecycleManager::new();
    manager.start();
    let notification_count = manager.check_and_send_notifications();
    assert_eq!(notification_count, 0);
    manager.shutdown();
}
#[test]
fn test_removed_address_cleaned_from_state() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr = create_test_address(1, 1, "0800-1200");
    detect_transitions(std::slice::from_ref(&addr));
    detect_transitions(&[]);
    let transitions = detect_transitions(&[addr]);
    assert!(
        !transitions.is_empty(),
        "Re-adding removed address should trigger notification",
    );
}
#[test]
fn test_multiple_transitions_in_one_check() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr1 = create_test_address(1, 1, "0800-1200");
    let addr2 = create_test_address(2, 1, "1400-1800");
    let addr3 = create_test_address(3, 2, "0800-1200");
    let transitions = detect_transitions(&[addr1, addr2, addr3]);
    assert_eq!(
        transitions.len(),
        3,
        "All three addresses should trigger notifications"
    );
}
#[test]
fn test_state_persistence_across_checks() {
    clear_panel_state();
    initialize_panel_tracker();
    let addr = create_test_address(1, 1, "0800-1200");
    let transitions1 = detect_transitions(std::slice::from_ref(&addr));
    assert_eq!(transitions1.len(), 1, "First check should find transition");
    let transitions2 = detect_transitions(std::slice::from_ref(&addr));
    assert_eq!(
        transitions2.len(),
        0,
        "Second check should find no new transitions"
    );
    let transitions3 = detect_transitions(&[addr]);
    assert_eq!(
        transitions3.len(),
        0,
        "Third check should still find no new transitions",
    );
}
#[test]
fn test_address_modification_detected() {
    clear_panel_state();
    initialize_panel_tracker();
    let mut addr = create_test_address(1, 10, "0800-1200");
    detect_transitions(std::slice::from_ref(&addr));
    let db_closer = DB::from_dag_tid(
        Some("22100".to_string()),
        "Test Street 1".to_string(),
        Some("Test Street".to_string()),
        Some("1".to_string()),
        None,
        1,
        "0800-1200",
        None,
        None,
        None,
        2024,
        1,
    )
    .unwrap();
    addr.matched_entry = Some(db_closer);
    let transitions = detect_transitions(&[addr]);
    assert!(
        !transitions.is_empty(),
        "Modified address should trigger transition"
    );
}
