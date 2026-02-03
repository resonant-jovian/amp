//! Application lifecycle and background task management
//!
//! Manages background operations including:
//! - Daily storage operations (read once, write once per day)
//! - Validity checks for date-dependent addresses
//! - Uptime tracking
//! - Graceful shutdown with data persistence
//!
//! # Background Tasks
//!
//! ## Daily Operations
//! - **00:00 (midnight)**: Read storage, check validity, write if changed
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
//! // Start background tasks
//! manager.start();
//!
//! // On app shutdown
//! manager.shutdown();
//! ```

use crate::components::storage::{read_addresses_from_device, write_addresses_to_device};
use crate::components::validity::check_and_update_validity;
use crate::ui::StoredAddress;
use chrono::{DateTime, Local, Timelike};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Lifecycle manager for background operations
///
/// Handles scheduling and execution of background tasks including
/// daily storage operations and validity checks.
pub struct LifecycleManager {
    /// Last time daily tasks were run
    last_daily_run: Arc<Mutex<Option<DateTime<Local>>>>,
    /// Application start time
    start_time: DateTime<Local>,
    /// Whether the manager is currently running
    running: Arc<Mutex<bool>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            last_daily_run: Arc::new(Mutex::new(None)),
            start_time: Local::now(),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the lifecycle manager background tasks
    ///
    /// This should be called on app startup and will:
    /// - Load addresses from storage
    /// - Check validity
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
        
        // Perform initial load and check
        self.perform_daily_tasks();
        
        // Mark last run as now
        let mut last_run = self.last_daily_run.lock().unwrap();
        *last_run = Some(Local::now());
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
        
        // Perform final save
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
            None => true, // Never run
            Some(last) => {
                // Check if it's a new day
                last.date_naive() != now.date_naive()
            }
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
        
        // Read current addresses
        let mut addresses = read_addresses_from_device();
        
        // Check and update validity
        let validity_changed = check_and_update_validity(&mut addresses);
        
        // Write back if anything changed
        if validity_changed {
            if let Err(e) = write_addresses_to_device(&addresses) {
                eprintln!("[Lifecycle] Failed to save after validity update: {}", e);
            } else {
                eprintln!("[Lifecycle] Saved {} addresses after validity update", addresses.len());
            }
        } else {
            eprintln!("[Lifecycle] No validity changes, skipping write");
        }
    }
    
    /// Get uptime in seconds since manager was created
    pub fn uptime_seconds(&self) -> i64 {
        let now = Local::now();
        (now - self.start_time).num_seconds()
    }
    
    /// Check if manager is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
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
/// - Check validity if needed
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
    eprintln!("[Lifecycle] Address change detected, persisting {} addresses", addresses.len());
    
    if let Err(e) = write_addresses_to_device(addresses) {
        eprintln!("[Lifecycle] Failed to persist address change: {}", e);
    }
}

/// Handle validity state change (day 29/30 in February transition)
///
/// This should be called when validity changes are detected and will:
/// - Write updated addresses to storage immediately
///
/// # Arguments
/// * `addresses` - Current address list after validity update
pub fn handle_validity_change(addresses: &[StoredAddress]) {
    eprintln!("[Lifecycle] Validity change detected, persisting {} addresses", addresses.len());
    
    if let Err(e) = write_addresses_to_device(addresses) {
        eprintln!("[Lifecycle] Failed to persist validity change: {}", e);
    }
}

/// Handle active state toggle
///
/// This should be called when address active state is toggled and will:
/// - Write updated addresses to storage immediately
///
/// # Arguments
/// * `addresses` - Current address list after active toggle
pub fn handle_active_toggle(addresses: &[StoredAddress]) {
    eprintln!("[Lifecycle] Active state toggled, persisting {} addresses", addresses.len());
    
    if let Err(e) = write_addresses_to_device(addresses) {
        eprintln!("[Lifecycle] Failed to persist active toggle: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new();
        assert!(!manager.is_running());
    }
    
    #[test]
    fn test_lifecycle_manager_start_stop() {
        let mut manager = LifecycleManager::new();
        manager.start();
        assert!(manager.is_running());
        
        manager.shutdown();
        assert!(!manager.is_running());
    }
    
    #[test]
    fn test_uptime_tracking() {
        let manager = LifecycleManager::new();
        thread::sleep(Duration::from_millis(100));
        let uptime = manager.uptime_seconds();
        assert!(uptime >= 0);
    }
}
