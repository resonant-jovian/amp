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
        parking_info: None,
    }
}
/// Helper that returns both address and its current time bucket
fn create_test_address_with_bucket(id: usize, day: u8, time: &str) -> (StoredAddress, TimeBucket) {
    let addr = create_test_address(id, day, time);
    let bucket = bucket_for(addr.matched_entry.as_ref().unwrap());
    (addr, bucket)
}
/// Returns true if the bucket is considered actionable for notifications
fn is_actionable(bucket: &TimeBucket) -> bool {
    matches!(
        bucket,
        TimeBucket::Within1Day | TimeBucket::Within6Hours | TimeBucket::Now
    )
}
#[test]
fn test_first_time_address_detection() {
    clear_panel_state();
    initialize_panel_tracker();
    let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
    eprintln!("test_first_time_address_detection bucket: {:?}", bucket);
    let transitions = detect_transitions(std::slice::from_ref(&addr));
    if is_actionable(&bucket) {
        assert!(
            !transitions.is_empty(),
            "First time seeing address in actionable bucket should trigger transition (bucket = {:?})",
            bucket,
        );
    } else {
        assert!(
            transitions.is_empty(),
            "Non-actionable bucket {:?} should not trigger transitions",
            bucket,
        );
    }
}
#[test]
fn test_no_duplicate_notifications() {
    clear_panel_state();
    initialize_panel_tracker();
    let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
    let first_transitions = detect_transitions(std::slice::from_ref(&addr));
    if is_actionable(&bucket) {
        assert!(
            !first_transitions.is_empty(),
            "First detection in actionable bucket should trigger (bucket = {:?})",
            bucket,
        );
    } else {
        assert!(
            first_transitions.is_empty(),
            "Non-actionable bucket {:?} should not trigger transitions",
            bucket,
        );
    }
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
    let (addr1, bucket1) = create_test_address_with_bucket(1, 1, "0800-1200");
    let (addr2, bucket2) = create_test_address_with_bucket(2, 2, "0800-1200");
    let transitions1 = detect_transitions(std::slice::from_ref(&addr1));
    let expected1 = if is_actionable(&bucket1) { 1 } else { 0 };
    assert_eq!(
        transitions1.len(),
        expected1,
        "First address should have {} transition(s) for bucket {:?}",
        expected1,
        bucket1,
    );
    let transitions2 = detect_transitions(&[addr1.clone(), addr2.clone()]);
    let expected2 = expected1 + if is_actionable(&bucket2) { 1 } else { 0 };
    assert_eq!(
        transitions2.len(),
        expected2,
        "Second check: only newly actionable addresses should trigger (b1={:?}, b2={:?})",
        bucket1,
        bucket2,
    );
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
        parking_info: None,
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
    let (addr1, bucket1) = create_test_address_with_bucket(1, 10, "0800-1200");
    let _ = detect_transitions(std::slice::from_ref(&addr1));
    let mut addr2 = addr1.clone();
    let db_more_urgent = DB::from_dag_tid(
        Some("22100".to_string()),
        "Test Street 1".to_string(),
        Some("Test Street".to_string()),
        Some("1".to_string()),
        None,
        1,
        "0000-0400",
        None,
        None,
        None,
        2024,
        1,
    )
    .expect("Failed to create more urgent DB entry");
    addr2.matched_entry = Some(db_more_urgent);
    let bucket2 = bucket_for(addr2.matched_entry.as_ref().unwrap());
    let transitions2 = detect_transitions(std::slice::from_ref(&addr2));
    let should_notify = matches!(
        (&bucket1, &bucket2),
        (TimeBucket::MoreThan1Month, TimeBucket::Within1Day)
            | (TimeBucket::MoreThan1Month, TimeBucket::Within6Hours)
            | (TimeBucket::MoreThan1Month, TimeBucket::Now)
            | (TimeBucket::Within1Month, TimeBucket::Within1Day)
            | (TimeBucket::Within1Month, TimeBucket::Within6Hours)
            | (TimeBucket::Within1Month, TimeBucket::Now)
            | (TimeBucket::Within1Day, TimeBucket::Within6Hours)
            | (TimeBucket::Within1Day, TimeBucket::Now)
            | (TimeBucket::Within6Hours, TimeBucket::Now)
    );
    if should_notify {
        assert!(
            !transitions2.is_empty(),
            "Transition to more urgent bucket should trigger notification ({:?} -> {:?})",
            bucket1,
            bucket2,
        );
    } else {
        assert!(
            transitions2.is_empty(),
            "Non-notifying bucket change {:?} -> {:?} should not trigger",
            bucket1,
            bucket2,
        );
    }
}
#[test]
fn test_bucket_categorization() {
    let addr_tomorrow = create_test_address(1, 1, "0800-1200");
    let bucket = bucket_for(addr_tomorrow.matched_entry.as_ref().unwrap());
    assert!(
        matches!(
            bucket,
            TimeBucket::Within1Day
                | TimeBucket::Within6Hours
                | TimeBucket::Now
                | TimeBucket::Within1Month
        ),
        "Unexpected bucket for 'tomorrow': {:?}",
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
    let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
    let first = detect_transitions(std::slice::from_ref(&addr));
    if is_actionable(&bucket) {
        assert!(
            !first.is_empty(),
            "Initial actionable bucket should trigger (bucket = {:?})",
            bucket,
        );
    } else {
        assert!(
            first.is_empty(),
            "Non-actionable bucket {:?} should not trigger on first detection",
            bucket,
        );
    }
    detect_transitions(&[]);
    let transitions = detect_transitions(&[addr]);
    if is_actionable(&bucket) {
        assert!(
            !transitions.is_empty(),
            "Re-adding address in actionable bucket should trigger notification",
        );
    } else {
        assert!(
            transitions.is_empty(),
            "Re-adding non-actionable address should not trigger notification",
        );
    }
}
#[test]
fn test_multiple_transitions_in_one_check() {
    clear_panel_state();
    initialize_panel_tracker();
    let (addr1, bucket1) = create_test_address_with_bucket(1, 1, "0800-1200");
    let (addr2, bucket2) = create_test_address_with_bucket(2, 1, "1400-1800");
    let (addr3, bucket3) = create_test_address_with_bucket(3, 2, "0800-1200");
    let transitions = detect_transitions(&[addr1, addr2, addr3]);
    let expected = [bucket1, bucket2, bucket3]
        .into_iter()
        .filter(is_actionable)
        .count();
    assert_eq!(
        transitions.len(),
        expected,
        "Expected {} actionable address(es), got {} transitions",
        expected,
        transitions.len(),
    );
}
#[test]
fn test_state_persistence_across_checks() {
    clear_panel_state();
    initialize_panel_tracker();
    let (addr, bucket) = create_test_address_with_bucket(1, 1, "0800-1200");
    let transitions1 = detect_transitions(std::slice::from_ref(&addr));
    let expected1 = if is_actionable(&bucket) { 1 } else { 0 };
    assert_eq!(
        transitions1.len(),
        expected1,
        "First check should find {} transition(s) for bucket {:?}",
        expected1,
        bucket,
    );
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
    let bucket_before = bucket_for(addr.matched_entry.as_ref().unwrap());
    let _ = detect_transitions(std::slice::from_ref(&addr));
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
    .expect("Failed to create closer DB entry");
    addr.matched_entry = Some(db_closer);
    let bucket_after = bucket_for(addr.matched_entry.as_ref().unwrap());
    let transitions = detect_transitions(&[addr]);
    let should_notify = matches!(
        (&bucket_before, &bucket_after),
        (TimeBucket::MoreThan1Month, TimeBucket::Within1Day)
            | (TimeBucket::MoreThan1Month, TimeBucket::Within6Hours)
            | (TimeBucket::MoreThan1Month, TimeBucket::Now)
            | (TimeBucket::Within1Month, TimeBucket::Within1Day)
            | (TimeBucket::Within1Month, TimeBucket::Within6Hours)
            | (TimeBucket::Within1Month, TimeBucket::Now)
            | (TimeBucket::Within1Day, TimeBucket::Within6Hours)
            | (TimeBucket::Within1Day, TimeBucket::Now)
            | (TimeBucket::Within6Hours, TimeBucket::Now)
    );
    if should_notify {
        assert!(
            !transitions.is_empty(),
            "Modified address should trigger transition for {:?} -> {:?}",
            bucket_before,
            bucket_after,
        );
    } else {
        assert!(
            transitions.is_empty(),
            "Non-notifying bucket change {:?} -> {:?} should not trigger",
            bucket_before,
            bucket_after,
        );
    }
}
