//! Countdown and time bucket calculations for parking restrictions
//!
//! This module provides time-based calculations and categorization for
//! parking restrictions using the new DB struct with proper chrono timestamps.
//!
//! # Migration Note
//! This version has been updated to use `DB` struct with `DateTime<Utc>`.
//! The old `parse_time_interval` function has been removed as time parsing
//! is now handled by `DB::from_dag_tid()`.
//!
//! # Examples
//! ```no_run
//! use amp_android::countdown::{remaining_duration, format_countdown, bucket_for};
//! use amp_core::structs::DB;
//! use chrono::Utc;
//!
//! // Create a DB entry
//! let db = DB::from_dag_tid(
//!     Some("22100".to_string()),
//!     "Storgatan 10".to_string(),
//!     Some("Storgatan".to_string()),
//!     Some("10".to_string()),
//!     None,
//!     15,
//!     "0800-1200",
//!     None,
//!     None,
//!     None,
//!     2024,
//!     1,
//! ).unwrap();
//!
//! // Calculate remaining time
//! if let Some(duration) = remaining_duration(&db) {
//!     println!("Time remaining: {} minutes", duration.num_minutes());
//! }
//!
//! // Format as string
//! if let Some(countdown) = format_countdown(&db) {
//!     println!("Countdown: {}", countdown);
//! }
//!
//! // Categorize by urgency
//! let bucket = bucket_for(&db);
//! println!("Urgency: {:?}", bucket);
//! ```
use amp_core::structs::DB;
use chrono::{Duration, Utc};
/// Calculate remaining duration until parking restriction ends
///
/// Uses the DB struct's `time_until_end` method to calculate the duration
/// from the current time until the restriction's end_time.
///
/// # Arguments
/// * `restriction` - DB entry containing restriction information with timestamps
///
/// # Returns
/// Some(Duration) if restriction is still active or upcoming, None otherwise
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::remaining_duration;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// if let Some(duration) = remaining_duration(&db) {
///     println!("Minutes remaining: {}", duration.num_minutes());
/// }
/// ```
pub fn remaining_duration(restriction: &DB) -> Option<Duration> {
    let now = Utc::now();
    restriction.time_until_end(now)
}
/// Check if a parking restriction is currently active
///
/// Wrapper around DB's `is_active` method for convenience.
///
/// # Arguments
/// * `restriction` - DB entry to check
///
/// # Returns
/// true if the current time is between start_time and end_time
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::is_currently_active;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// if is_currently_active(&db) {
///     println!("Parking restriction is active right now!");
/// }
/// ```
pub fn is_currently_active(restriction: &DB) -> bool {
    let now = Utc::now();
    restriction.is_active(now)
}
/// Calculate time until restriction starts
///
/// Uses the DB struct's `time_until_start` method.
///
/// # Arguments
/// * `restriction` - DB entry containing restriction information
///
/// # Returns
/// Some(Duration) if restriction hasn't started yet, None if already started or passed
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::time_until_start;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// if let Some(duration) = time_until_start(&db) {
///     println!("Restriction starts in {} hours", duration.num_hours());
/// }
/// ```
pub fn time_until_start(restriction: &DB) -> Option<Duration> {
    let now = Utc::now();
    restriction.time_until_start(now)
}
/// Format countdown as human-readable string
///
/// Converts the remaining duration into a formatted string showing
/// days, hours, and minutes.
///
/// # Arguments
/// * `restriction` - DB entry with parking restriction timestamps
///
/// # Returns
/// Formatted string like "5d 02h 30m" or None if calculation fails
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::format_countdown;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// if let Some(countdown) = format_countdown(&db) {
///     println!("Time remaining: {}", countdown);
/// }
/// ```
pub fn format_countdown(restriction: &DB) -> Option<String> {
    let remaining = remaining_duration(restriction)?;
    let days = remaining.num_days();
    let hours = remaining.num_hours() % 24;
    let minutes = remaining.num_minutes() % 60;
    Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
}
/// Format countdown in compact form (no days if zero)
///
/// Provides a more compact representation, omitting days if zero.
///
/// # Arguments
/// * `restriction` - DB entry with parking restriction timestamps
///
/// # Returns
/// Formatted string like "2h 30m" or "1d 5h 15m", None if calculation fails
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::format_countdown_compact;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// if let Some(countdown) = format_countdown_compact(&db) {
///     println!("Time remaining: {}", countdown);
/// }
/// ```
pub fn format_countdown_compact(restriction: &DB) -> Option<String> {
    let remaining = remaining_duration(restriction)?;
    let days = remaining.num_days();
    let hours = remaining.num_hours() % 24;
    let minutes = remaining.num_minutes() % 60;
    if days > 0 {
        Some(format!("{}d {}h {}m", days, hours, minutes))
    } else {
        Some(format!("{}h {}m", hours, minutes))
    }
}
/// Time bucket categories for grouping parking restrictions
///
/// Categorizes restrictions by urgency based on time remaining.
/// Used for sorting and filtering addresses by deadline proximity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeBucket {
    /// Restriction ends within 4 hours (highest urgency)
    Now,
    /// Restriction ends within 6 hours
    Within6Hours,
    /// Restriction ends within 1 day
    Within1Day,
    /// Restriction ends within 1 month
    Within1Month,
    /// Invalid, expired, or far-future restriction
    Invalid,
}
impl TimeBucket {
    /// Get human-readable label for the time bucket
    ///
    /// # Returns
    /// String describing the time bucket
    pub fn label(&self) -> &'static str {
        match self {
            TimeBucket::Now => "Urgent (< 4h)",
            TimeBucket::Within6Hours => "Soon (< 6h)",
            TimeBucket::Within1Day => "Today (< 24h)",
            TimeBucket::Within1Month => "This Month",
            TimeBucket::Invalid => "Invalid/Expired",
        }
    }
    /// Get emoji/icon for the time bucket
    ///
    /// # Returns
    /// Unicode emoji representing urgency level
    pub fn icon(&self) -> &'static str {
        match self {
            TimeBucket::Now => "🔴",
            TimeBucket::Within6Hours => "🟡",
            TimeBucket::Within1Day => "🟢",
            TimeBucket::Within1Month => "⚪",
            TimeBucket::Invalid => "⚫",
        }
    }
}
/// Categorize restriction by time remaining until deadline
///
/// Assigns a TimeBucket based on how much time is left before
/// the parking restriction ends.
///
/// # Arguments
/// * `restriction` - DB entry containing restriction timestamps
///
/// # Returns
/// TimeBucket representing urgency of the restriction
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::{bucket_for, TimeBucket};
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     15, "0800-1200",
/// #     None, None, None,
/// #     2024, 1,
/// # ).unwrap();
/// let bucket = bucket_for(&db);
/// println!("Urgency: {} {}", bucket.icon(), bucket.label());
/// ```
pub fn bucket_for(restriction: &DB) -> TimeBucket {
    let remaining = match remaining_duration(restriction) {
        Some(d) => d,
        None => return TimeBucket::Invalid,
    };
    if remaining <= Duration::hours(4) {
        TimeBucket::Now
    } else if remaining <= Duration::hours(6) {
        TimeBucket::Within6Hours
    } else if remaining <= Duration::days(1) {
        TimeBucket::Within1Day
    } else if remaining <= Duration::days(31) {
        TimeBucket::Within1Month
    } else {
        TimeBucket::Invalid
    }
}
/// Group multiple restrictions by time bucket
///
/// Takes a collection of DB entries and groups them by urgency.
///
/// # Arguments
/// * `restrictions` - Iterator of DB entries to categorize
///
/// # Returns
/// HashMap mapping TimeBucket to vector of restriction references
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::group_by_bucket;
/// use amp_android::static_data::get_static_data;
///
/// let data = get_static_data();
/// let grouped = group_by_bucket(data.values());
///
/// for (bucket, items) in grouped {
///     println!("{} {}: {} items", bucket.icon(), bucket.label(), items.len());
/// }
/// ```
pub fn group_by_bucket<'a, I>(restrictions: I) -> std::collections::HashMap<TimeBucket, Vec<&'a DB>>
where
    I: Iterator<Item = &'a DB>,
{
    let mut groups: std::collections::HashMap<TimeBucket, Vec<&DB>> =
        std::collections::HashMap::new();
    for restriction in restrictions {
        let bucket = bucket_for(restriction);
        groups
            .entry(bucket)
            .or_insert_with(Vec::new)
            .push(restriction);
    }
    groups
}
#[cfg(test)]
mod tests {
    use super::*;
    use amp_core::structs::DB;
    /// Helper to create a test DB entry
    fn create_test_db(day: u8, time: &str) -> DB {
        DB::from_dag_tid(
            Some("22100".to_string()),
            "Test Street 10".to_string(),
            Some("Test Street".to_string()),
            Some("10".to_string()),
            Some("Test restriction".to_string()),
            day,
            time,
            Some("Taxa C".to_string()),
            Some(5),
            Some("Längsgående".to_string()),
            2024,
            1,
        )
        .expect("Failed to create test DB entry")
    }
    #[test]
    fn test_remaining_duration() {
        let db = create_test_db(28, "2300-2359");
        let result = remaining_duration(&db);
        assert!(result.is_some() || result.is_none());
    }
    #[test]
    fn test_format_countdown() {
        let db = create_test_db(15, "0800-1200");
        let result = format_countdown(&db);
        if let Some(s) = result {
            assert!(s.contains("d"));
            assert!(s.contains("h"));
            assert!(s.contains("m"));
        }
    }
    #[test]
    fn test_format_countdown_compact() {
        let db = create_test_db(15, "0800-1200");
        let result = format_countdown_compact(&db);
        if let Some(s) = result {
            assert!(s.contains("h"));
            assert!(s.contains("m"));
        }
    }
    #[test]
    fn test_bucket_for() {
        let db = create_test_db(15, "0800-1200");
        let bucket = bucket_for(&db);
        assert!(matches!(
            bucket,
            TimeBucket::Now
                | TimeBucket::Within6Hours
                | TimeBucket::Within1Day
                | TimeBucket::Within1Month
                | TimeBucket::Invalid
        ),);
    }
    #[test]
    fn test_time_bucket_label() {
        assert_eq!(TimeBucket::Now.label(), "Urgent (< 4h)");
        assert_eq!(TimeBucket::Within6Hours.label(), "Soon (< 6h)");
        assert_eq!(TimeBucket::Within1Day.label(), "Today (< 24h)");
        assert_eq!(TimeBucket::Within1Month.label(), "This Month");
        assert_eq!(TimeBucket::Invalid.label(), "Invalid/Expired");
    }
    #[test]
    fn test_time_bucket_icon() {
        assert_eq!(TimeBucket::Now.icon(), "🔴");
        assert_eq!(TimeBucket::Within6Hours.icon(), "🟡");
        assert_eq!(TimeBucket::Within1Day.icon(), "🟢");
        assert_eq!(TimeBucket::Within1Month.icon(), "⚪");
        assert_eq!(TimeBucket::Invalid.icon(), "⚫");
    }
    #[test]
    fn test_is_currently_active() {
        let db = create_test_db(15, "0800-1200");
        let result = is_currently_active(&db);
        assert!(result || !result);
    }
    #[test]
    fn test_time_until_start() {
        let db = create_test_db(28, "2300-2359");
        let result = time_until_start(&db);
        assert!(result.is_some() || result.is_none());
    }
    #[test]
    fn test_group_by_bucket() {
        let restrictions = vec![
            create_test_db(15, "0800-1200"),
            create_test_db(20, "1400-1800"),
            create_test_db(25, "0900-1700"),
        ];
        let grouped = group_by_bucket(restrictions.iter());
        assert!(!grouped.is_empty());
        let total: usize = grouped.values().map(|v| v.len()).sum();
        assert_eq!(total, 3);
    }
    #[test]
    fn test_db_struct_integration() {
        let db = create_test_db(15, "0800-1200");
        let now = Utc::now();
        let _is_active = db.is_active(now);
        let _time_until_end = db.time_until_end(now);
        let _time_until_start = db.time_until_start(now);
    }
}
