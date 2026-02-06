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

use amp_android::components::{
    countdown::{bucket_for, TimeBucket},
    lifecycle::LifecycleManager,
    notifications::{initialize_notification_channels, notify_active, notify_one_day, notify_six_hours},
    settings::{load_settings, NotificationSettings, save_settings, AppSettings},
    transitions::{clear_panel_state, detect_transitions, initialize_panel_tracker},
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
    // Test that initialization doesn't panic
    initialize_notification_channels();
    initialize_panel_tracker();
}

#[test]
fn test_complete_notification_flow() {
    // Clear any previous state
    clear_panel_state();
    initialize_panel_tracker();

    // Create address that will be in an actionable bucket
    let addr = create_test_address(1, 1, "0800-1200");
    let bucket = bucket_for(addr.matched_entry.as_ref().unwrap());

    println!("Test address is in bucket: {:?}", bucket);

    // First detection should trigger notification
    let transitions = detect_transitions(&[addr.clone()]);
    assert!(
        !transitions.is_empty(),
        "First detection in actionable bucket should trigger transition"
    );

    let (_, prev, new) = &transitions[0];
    assert!(prev.is_none(), "Previous bucket should be None on first detection");
    println!("Transition detected: None → {:?}", new);

    // Simulate sending notification
    match new {
        TimeBucket::Within1Day => notify_one_day(&addr),
        TimeBucket::Within6Hours => notify_six_hours(&addr),
        TimeBucket::Now => notify_active(&addr),
        _ => panic!("Unexpected bucket: {:?}", new),
    }

    // Second detection should NOT trigger (same bucket)
    let transitions2 = detect_transitions(&[addr]);
    assert_eq!(
        transitions2.len(),
        0,
        "Same bucket should not trigger duplicate notification"
    );
}

#[test]
fn test_multiple_address_transitions() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create multiple addresses in different buckets
    let addr1 = create_test_address(1, 1, "0800-1200");
    let addr2 = create_test_address(2, 2, "0800-1200");
    let addr3 = create_test_address(3, 3, "0800-1200");

    let addresses = vec![addr1.clone(), addr2.clone(), addr3.clone()];

    // First check should detect all addresses
    let transitions = detect_transitions(&addresses);
    println!("First check detected {} transitions", transitions.len());

    // At least some should trigger (depends on time buckets)
    // We can't assert exact count without knowing current time

    // Second check should detect nothing (all in same buckets)
    let transitions2 = detect_transitions(&addresses);
    assert_eq!(
        transitions2.len(),
        0,
        "Second check should not trigger duplicate notifications"
    );
}

#[test]
fn test_notification_settings_respect() {
    // Save settings with all notifications disabled
    let settings = AppSettings {
        notifications: NotificationSettings {
            stadning_nu: false,
            sex_timmar: false,
            en_dag: false,
        },
        ..Default::default()
    };
    save_settings(&settings);

    // Create test address
    let addr = create_test_address(99, 1, "0800-1200");

    // These should not panic even with settings disabled
    // They should log that notifications are skipped
    notify_one_day(&addr);
    notify_six_hours(&addr);
    notify_active(&addr);

    // Restore default settings
    let default_settings = AppSettings::default();
    save_settings(&default_settings);
}

#[test]
fn test_transition_to_more_urgent_bucket() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create address that will transition through buckets
    // (This is a simulation - in practice, time would change)
    let addr1 = create_test_address(1, 1, "0800-1200");

    // First detection
    let transitions1 = detect_transitions(&[addr1.clone()]);
    println!("First detection: {} transitions", transitions1.len());

    // To properly test bucket transitions, we'd need to manipulate time
    // or create addresses with different time characteristics
    // For now, verify the basic logic doesn't panic
}

#[test]
fn test_lifecycle_manager_notification_integration() {
    // Test that lifecycle manager can check notifications
    let mut manager = LifecycleManager::new();
    manager.start();

    assert!(manager.is_running());
    assert!(manager.are_notifications_initialized());

    // Check for notifications (should not panic)
    let count = manager.check_and_send_notifications();
    println!("Lifecycle manager sent {} notifications", count);

    manager.shutdown();
    assert!(!manager.is_running());
}

#[test]
fn test_address_without_match_ignored() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create address without parking match
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
        "Addresses without matches should not trigger notifications"
    );
}

#[test]
fn test_inactive_addresses_handled() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create address marked as inactive
    let mut addr = create_test_address(1, 1, "0800-1200");
    addr.active = false;

    // Transition detection should still work (active flag is UI-only)
    let transitions = detect_transitions(&[addr]);

    // The detection should still occur if the address has a parking match
    // The active flag is used for UI display filtering
    println!("Inactive address transitions: {}", transitions.len());
}

#[test]
fn test_notification_channel_constants() {
    // Verify channel IDs match between Rust and expected Kotlin values
    // This is important for JNI integration
    
    // Channel constants are private in notifications.rs
    // We verify they work by calling the functions
    let addr = create_test_address(1, 1, "0800-1200");
    
    // These should not panic
    notify_one_day(&addr);
    notify_six_hours(&addr);
    notify_active(&addr);
}

#[test]
fn test_same_address_different_buckets() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create address in first bucket
    let addr1 = create_test_address(1, 1, "0800-1200");
    let transitions1 = detect_transitions(&[addr1]);
    
    if !transitions1.is_empty() {
        println!("First bucket: {:?}", transitions1[0].2);
    }

    // Create same ID address but different time (simulating time passing)
    // Note: In practice this would require actual time change
    // For testing, we verify the logic doesn't break
    let addr2 = create_test_address(1, 2, "0800-1200");
    let transitions2 = detect_transitions(&[addr2]);
    
    println!("Transitions after time change: {}", transitions2.len());
}

#[test]
fn test_bulk_notification_check() {
    clear_panel_state();
    initialize_panel_tracker();

    // Create 10 test addresses
    let addresses: Vec<StoredAddress> = (1..=10)
        .map(|i| create_test_address(i, (i % 31) as u8 + 1, "0800-1200"))
        .collect();

    // First check
    let transitions1 = detect_transitions(&addresses);
    println!("Bulk check: {} transitions detected", transitions1.len());

    // Second check should show no new transitions
    let transitions2 = detect_transitions(&addresses);
    assert_eq!(
        transitions2.len(),
        0,
        "Bulk second check should not trigger duplicates"
    );
}

#[test]
fn test_notification_system_under_load() {
    // Test notification system with many rapid calls
    let addr = create_test_address(1, 1, "0800-1200");

    // Rapidly call notification functions
    for _ in 0..100 {
        notify_one_day(&addr);
        notify_six_hours(&addr);
        notify_active(&addr);
    }

    // Should not panic or cause issues
    // In real Android, this would create many notifications
    // but the system handles deduplication
}

#[test]
fn test_lifecycle_manager_multiple_starts() {
    let mut manager = LifecycleManager::new();

    // Start multiple times (should be idempotent)
    manager.start();
    assert!(manager.is_running());

    manager.start(); // Second start should be no-op
    assert!(manager.is_running());

    manager.shutdown();
}

#[test]
fn test_settings_persistence_through_notifications() {
    // Save specific settings
    let settings = AppSettings {
        notifications: NotificationSettings {
            stadning_nu: true,
            sex_timmar: false,
            en_dag: true,
        },
        ..Default::default()
    };
    save_settings(&settings);

    // Load and verify
    let loaded = load_settings();
    assert!(loaded.notifications.stadning_nu);
    assert!(!loaded.notifications.sex_timmar);
    assert!(loaded.notifications.en_dag);

    // Test notifications with these settings
    let addr = create_test_address(1, 1, "0800-1200");
    notify_one_day(&addr); // Should work (enabled)
    notify_six_hours(&addr); // Should be skipped (disabled)
    notify_active(&addr); // Should work (enabled)

    // Restore defaults
    save_settings(&AppSettings::default());
}
