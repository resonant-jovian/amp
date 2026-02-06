//! Application lifecycle and background task management
//!
//! Manages background operations including:
//! - Daily storage operations (read once, write once per day)
//! - Validity checks for date-dependent addresses
//! - Panel transition detection and notifications
//! - Graceful shutdown with data persistence
//!
//! # Background Tasks
//!
//! ## Daily Operations
//! - **00:00 (midnight)**: Read storage, check validity, write if changed
//! - **Every 60 seconds**: Check for panel transitions and send notifications
//! - **On state change**: Write to storage immediately
//! - **On crash/exit**: Write to storage (graceful shutdown)
//!
//! ## Android Integration
//!
//! This component should be integrated with Android's:
//! - `WorkManager` for scheduled daily tasks
//! - `ForegroundService` for continuous background operation
//! - `BroadcastReceiver` for boot completion (RECEIVE_BOOT_COMPLETED)
//!
//! # Examples
//!
//! ```no_run
//! use amp_android::components::lifecycle::LifecycleManager;
//!
//! // Create lifecycle manager
//! let mut manager = LifecycleManager::new();
//!
//! // Start background tasks (includes notification system)
//! manager.start();
//!
//! // Periodically check for notifications (e.g., every 60 seconds)
//! manager.check_and_send_notifications();
//!
//! // On app shutdown
//! manager.shutdown();
//! ```
use crate::components::countdown::TimeBucket;
use crate::components::notifications::{notify_active, notify_one_day, notify_six_hours};
use crate::components::storage::{read_addresses_from_device, write_addresses_to_device};
use crate::components::transitions::detect_transitions;
use crate::components::validity::check_and_update_validity;
use crate::ui::StoredAddress;
use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};
/// Lifecycle manager for background operations
///
/// Handles scheduling and execution of background tasks including
/// daily storage operations, validity checks, and notification management.
pub struct LifecycleManager {
    /// Last time daily tasks were run
    last_daily_run: Arc<Mutex<Option<DateTime<Local>>>>,
    /// Whether the manager is currently running
    running: Arc<Mutex<bool>>,
    /// Whether notification system has been initialized
    notifications_initialized: Arc<Mutex<bool>>,
}
impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            last_daily_run: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            notifications_initialized: Arc::new(Mutex::new(false)),
        }
    }
    /// Start the lifecycle manager background tasks
    ///
    /// This should be called on app startup and will:
    /// - Initialize notification channels and panel tracker
    /// - Load addresses from storage
    /// - Check validity
    /// - Perform initial notification check
    /// - Start background task scheduler
    pub fn start(&mut self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            eprintln!("[Lifecycle] Already running");
            return;
        }
        *running = true;
        drop(running);
        eprintln!("[Lifecycle] Starting lifecycle manager");
        self.initialize_notifications();
        self.perform_daily_tasks();
        self.check_and_send_notifications();
        let mut last_run = self.last_daily_run.lock().unwrap();
        *last_run = Some(Local::now());
    }
    /// Initialize notification system (channels and transition tracker)
    ///
    /// Called once on app startup. Safe to call multiple times.
    fn initialize_notifications(&self) {
        let mut initialized = self.notifications_initialized.lock().unwrap();
        if *initialized {
            eprintln!("[Lifecycle] Notifications already initialized");
            return;
        }
        eprintln!("[Lifecycle] Initializing notification system");
        crate::components::notifications::initialize_notification_channels();
        crate::components::transitions::initialize_panel_tracker();
        *initialized = true;
        eprintln!("[Lifecycle] Notification system initialized");
    }
    /// Check for panel transitions and send notifications
    ///
    /// This should be called periodically (e.g., every 60 seconds) to:
    /// - Load current addresses from storage
    /// - Detect transitions between time panels
    /// - Send appropriate notifications based on new panel
    ///
    /// # Returns
    /// Number of notifications sent
    ///
    /// # Examples
    /// ```no_run
    /// use amp_android::components::lifecycle::LifecycleManager;
    ///
    /// let mut manager = LifecycleManager::new();
    /// manager.start();
    ///
    /// // Call periodically (e.g., from a timer)
    /// let count = manager.check_and_send_notifications();
    /// println!("Sent {} notifications", count);
    /// ```
    pub fn check_and_send_notifications(&self) -> usize {
        eprintln!("[Lifecycle] Checking for notification-worthy transitions");
        let addresses = read_addresses_from_device();
        let transitions = detect_transitions(&addresses);
        if transitions.is_empty() {
            eprintln!("[Lifecycle] No transitions detected");
            return 0;
        }
        eprintln!("[Lifecycle] Processing {} transition(s)", transitions.len());
        let mut sent_count = 0;
        for (addr, prev_bucket, new_bucket) in transitions {
            eprintln!(
                "[Lifecycle] Transition: {} {} (id={}) {:?} → {:?}",
                addr.street, addr.street_number, addr.id, prev_bucket, new_bucket,
            );
            match new_bucket {
                TimeBucket::Within1Day => {
                    notify_one_day(&addr);
                    sent_count += 1;
                }
                TimeBucket::Within6Hours => {
                    notify_six_hours(&addr);
                    sent_count += 1;
                }
                TimeBucket::Now => {
                    notify_active(&addr);
                    sent_count += 1;
                }
                _ => {
                    eprintln!(
                        "[Lifecycle] Bucket {:?} does not trigger notification",
                        new_bucket,
                    );
                }
            }
        }
        eprintln!("[Lifecycle] Sent {} notification(s)", sent_count);
        sent_count
    }
    /// Shutdown the lifecycle manager
    ///
    /// This should be called on app shutdown and will:
    /// - Write current state to storage
    /// - Stop background tasks
    pub fn shutdown(&mut self) {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return;
        }
        *running = false;
        drop(running);
        eprintln!("[Lifecycle] Shutting down lifecycle manager");
        let addresses = read_addresses_from_device();
        if let Err(e) = write_addresses_to_device(&addresses) {
            eprintln!("[Lifecycle] Failed to save on shutdown: {}", e);
        } else {
            eprintln!("[Lifecycle] Final save completed");
        }
    }
    /// Check if daily tasks should run and execute them
    ///
    /// Daily tasks run if:
    /// - Never run before, OR
    /// - Last run was on a different day
    ///
    /// # Returns
    /// `true` if tasks were executed, `false` if skipped
    pub fn check_and_run_daily_tasks(&self) -> bool {
        let last_run = self.last_daily_run.lock().unwrap();
        let now = Local::now();
        let should_run = match *last_run {
            None => true,
            Some(last) => last.date_naive() != now.date_naive(),
        };
        drop(last_run);
        if should_run {
            self.perform_daily_tasks();
            let mut last_run = self.last_daily_run.lock().unwrap();
            *last_run = Some(now);
            true
        } else {
            false
        }
    }
    /// Perform daily maintenance tasks
    ///
    /// - Read addresses from storage
    /// - Check and update validity
    /// - Write back if changes occurred
    fn perform_daily_tasks(&self) {
        eprintln!("[Lifecycle] Running daily tasks at {}", Local::now());
        let mut addresses = read_addresses_from_device();
        let validity_changed = check_and_update_validity(&mut addresses);
        if validity_changed {
            if let Err(e) = write_addresses_to_device(&addresses) {
                eprintln!("[Lifecycle] Failed to save after validity update: {}", e);
            } else {
                eprintln!(
                    "[Lifecycle] Saved {} addresses after validity update",
                    addresses.len(),
                );
            }
        } else {
            eprintln!("[Lifecycle] No validity changes, skipping write");
        }
    }
    /// Check if manager is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    /// Check if notifications are initialized
    pub fn are_notifications_initialized(&self) -> bool {
        *self.notifications_initialized.lock().unwrap()
    }
}
impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for LifecycleManager {
    fn drop(&mut self) {
        if self.is_running() {
            self.shutdown();
        }
    }
}
/// Handle address state change (add/remove/toggle)
///
/// This should be called whenever addresses are modified and will:
/// - Write updated addresses to storage immediately
/// - Check for panel transitions and send notifications
///
/// # Arguments
/// * `addresses` - Current address list after modification
///
/// # Examples
/// ```no_run
/// use amp_android::components::lifecycle::handle_address_change;
///
/// let addresses = get_current_addresses();
/// handle_address_change(&addresses);
/// ```
pub fn handle_address_change(addresses: &[StoredAddress]) {
    eprintln!(
        "[Lifecycle] Address change detected, persisting {} addresses",
        addresses.len(),
    );
    if let Err(e) = write_addresses_to_device(addresses) {
        eprintln!("[Lifecycle] Failed to persist address change: {}", e);
    }
    check_and_send_notifications_standalone(addresses);
}
/// Handle active state toggle
///
/// This should be called when address active state is toggled and will:
/// - Write updated addresses to storage immediately
///
/// # Arguments
/// * `addresses` - Current address list after active toggle
pub fn handle_active_toggle(addresses: &[StoredAddress]) {
    eprintln!(
        "[Lifecycle] Active state toggled, persisting {} addresses",
        addresses.len(),
    );
    if let Err(e) = write_addresses_to_device(addresses) {
        eprintln!("[Lifecycle] Failed to persist active toggle: {}", e);
    }
}
/// Standalone notification check (for use outside LifecycleManager)
///
/// Detects transitions and sends notifications for the provided addresses.
/// Useful for UI-triggered checks when LifecycleManager is not available.
///
/// # Arguments
/// * `addresses` - Addresses to check for transitions
///
/// # Returns
/// Number of notifications sent
fn check_and_send_notifications_standalone(addresses: &[StoredAddress]) -> usize {
    let transitions = detect_transitions(addresses);
    if transitions.is_empty() {
        return 0;
    }
    let mut sent_count = 0;
    for (addr, _prev_bucket, new_bucket) in transitions {
        match new_bucket {
            TimeBucket::Within1Day => {
                notify_one_day(&addr);
                sent_count += 1;
            }
            TimeBucket::Within6Hours => {
                notify_six_hours(&addr);
                sent_count += 1;
            }
            TimeBucket::Now => {
                notify_active(&addr);
                sent_count += 1;
            }
            _ => {}
        }
    }
    sent_count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new();
        assert!(!manager.is_running());
        assert!(!manager.are_notifications_initialized());
    }
    #[test]
    fn test_lifecycle_manager_start_stop() {
        let mut manager = LifecycleManager::new();
        manager.start();
        assert!(manager.is_running());
        assert!(manager.are_notifications_initialized());
        manager.shutdown();
        assert!(!manager.is_running());
    }
    #[test]
    fn test_check_notifications_no_panic() {
        let mut manager = LifecycleManager::new();
        manager.start();
        let count = manager.check_and_send_notifications();
        assert_eq!(count, 0);
    }
}
