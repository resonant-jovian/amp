//! Dormant background check for parking restriction notifications
//!
//! This module provides the core logic for detecting parking restriction
//! transitions when the app is running in background (dormant) mode.
//! It is called from Kotlin's DormantService via JNI on an hourly schedule.
//!
//! # Flow
//! `DormantService (Kotlin)` → JNI → `dormant_hourly_check (Rust)`
//! → reads parquet, detects transitions → returns JSON
//! → `DormantService` → `NotificationHelper.showNotification()`
use crate::components::countdown::TimeBucket;
use crate::components::settings::load_settings;
use crate::components::storage::read_addresses_from_device;
use crate::components::transitions::detect_transitions;
use serde::Serialize;
/// Notification data returned to Kotlin for display
#[derive(Clone, Debug, Serialize)]
pub struct DormantNotification {
    pub channel_id: String,
    pub notification_id: i32,
    pub title: String,
    pub body: String,
}
/// Run the hourly dormant check
///
/// 1. Sets `APP_FILES_DIR` if not already set
/// 2. Reads stored addresses from parquet
/// 3. Filters to active addresses with matched entries
/// 4. Detects bucket transitions
/// 5. Maps transitions to notification data (respecting user settings)
///
/// Returns a list of notifications to send.
pub fn dormant_hourly_check(storage_path: &str) -> Vec<DormantNotification> {
    eprintln!(
        "[Dormant] Starting hourly check with storage_path={}",
        storage_path
    );
    if std::env::var("APP_FILES_DIR").is_err() {
        eprintln!("[Dormant] Setting APP_FILES_DIR={}", storage_path);
        unsafe {
            std::env::set_var("APP_FILES_DIR", storage_path);
        }
    }
    let addresses = read_addresses_from_device();
    eprintln!(
        "[Dormant] Loaded {} addresses from storage",
        addresses.len()
    );
    if addresses.is_empty() {
        eprintln!("[Dormant] No addresses stored, nothing to check");
        return Vec::new();
    }
    let active: Vec<_> = addresses
        .iter()
        .filter(|a| a.active && a.matched_entry.is_some())
        .cloned()
        .collect();
    eprintln!(
        "[Dormant] {} active addresses with matched entries",
        active.len()
    );
    if active.is_empty() {
        return Vec::new();
    }
    let transitions = detect_transitions(&active);
    eprintln!("[Dormant] Detected {} transitions", transitions.len());
    let settings = load_settings();
    let mut notifications = Vec::new();
    for (addr, _prev, new_bucket) in transitions {
        let (channel_id, title, body, should_send) = match new_bucket {
            TimeBucket::Now => (
                "amp_active".to_string(),
                "Städning NU!".to_string(),
                format!(
                    "Städning pågående på {}. Din bil på {} {} kan få böter!",
                    addr.street, addr.street, addr.street_number,
                ),
                settings.notifications.stadning_nu,
            ),
            TimeBucket::Within6Hours => (
                "amp_six_hours".to_string(),
                "Städning om 6 timmar".to_string(),
                format!(
                    "Städning börjar snart på {}. Du bör flytta din bil från {} {}.",
                    addr.street, addr.street, addr.street_number,
                ),
                settings.notifications.sex_timmar,
            ),
            TimeBucket::Within1Day => (
                "amp_one_day".to_string(),
                "Städning inom 1 dygn".to_string(),
                format!(
                    "Städning inom 1 dygn på {}. Planera att flytta din bil från {} {}.",
                    addr.street, addr.street, addr.street_number,
                ),
                settings.notifications.en_dag,
            ),
            _ => continue,
        };
        if !should_send {
            eprintln!(
                "[Dormant] Skipping notification for {} {} (disabled in settings)",
                addr.street, addr.street_number,
            );
            continue;
        }
        eprintln!(
            "[Dormant] Queuing notification: channel={}, title={}, addr={} {}",
            channel_id, title, addr.street, addr.street_number,
        );
        notifications.push(DormantNotification {
            channel_id,
            notification_id: addr.id as i32,
            title,
            body,
        });
    }
    eprintln!("[Dormant] Returning {} notifications", notifications.len());
    notifications
}
#[cfg(target_os = "android")]
use jni::JNIEnv;
#[cfg(target_os = "android")]
use jni::objects::{JClass, JString};
#[cfg(target_os = "android")]
use jni::sys::jstring;
/// JNI: Called by DormantBridge.dormantCheck(storagePath)
///
/// Returns a JSON string with notification data:
/// `[{"channel_id":"amp_active","notification_id":123,"title":"...","body":"..."}]`
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_se_malmo_skaggbyran_amp_DormantBridge_dormantCheck<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    storage_path: JString<'local>,
) -> jstring {
    let path: String = match env.get_string(&storage_path) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("[Dormant JNI] Failed to get storage path string: {:?}", e);
            let empty = env.new_string("[]").expect("Failed to create empty JSON");
            return empty.into_raw();
        }
    };
    let notifications = dormant_hourly_check(&path);
    let json = match serde_json::to_string(&notifications) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[Dormant JNI] Failed to serialize notifications: {:?}", e);
            "[]".to_string()
        }
    };
    eprintln!("[Dormant JNI] Returning JSON: {}", json);
    let output = env
        .new_string(&json)
        .expect("Failed to create JSON output string");
    output.into_raw()
}
/// JNI: Called by DormantBridge.initDormantStorage(storagePath)
///
/// Sets the APP_FILES_DIR environment variable for Rust storage modules.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_se_malmo_skaggbyran_amp_DormantBridge_initDormantStorage<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    storage_path: JString<'local>,
) {
    let path: String = match env.get_string(&storage_path) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("[Dormant JNI] Failed to get storage path string: {:?}", e);
            return;
        }
    };
    eprintln!("[Dormant JNI] Setting APP_FILES_DIR={}", path);
    unsafe {
        std::env::set_var("APP_FILES_DIR", &path);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dormant_notification_serialize() {
        let notif = DormantNotification {
            channel_id: "amp_active".to_string(),
            notification_id: 42,
            title: "Test".to_string(),
            body: "Test body".to_string(),
        };
        let json = serde_json::to_string(&[notif]).unwrap();
        assert!(json.contains("amp_active"));
        assert!(json.contains("42"));
        assert!(json.contains("Test"));
    }
    #[test]
    fn test_dormant_check_empty_storage() {
        let result = dormant_hourly_check("/tmp/nonexistent_amp_test");
        assert!(result.is_empty());
    }
}
