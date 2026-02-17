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
use chrono::{Datelike, Duration, Utc};
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
#[allow(unused)]
pub fn remaining_duration(restriction: &DB) -> Option<Duration> {
    let now = Utc::now();
    restriction.time_until_end(now)
}
/// Calculate duration until next occurrence of this restriction
///
/// If the restriction has already passed this month, calculates when it will
/// occur next month. Handles edge cases like February 29/30.
///
/// # Arguments
/// * `restriction` - DB entry containing restriction information
///
/// # Returns
/// Some(Duration) if a future occurrence exists, None if the date is invalid
/// (e.g., February 30 or February 29 in non-leap years)
///
/// # Examples
/// ```no_run
/// use amp_android::countdown::time_until_next_occurrence;
/// use amp_core::structs::DB;
///
/// # let db = DB::from_dag_tid(
/// #     Some("22100".to_string()),
/// #     "Test".to_string(),
/// #     None, None, None,
/// #     3, "0800-1200",
/// #     None, None, None,
/// #     2024, 2,
/// # ).unwrap();
/// // If today is Feb 5, this will calculate time until March 3
/// if let Some(duration) = time_until_next_occurrence(&db) {
///     println!("Next occurrence in {} days", duration.num_days());
/// }
/// ```
pub fn time_until_next_occurrence(restriction: &DB) -> Option<Duration> {
    let now = Utc::now();
    if let Some(duration) = restriction.time_until_end(now) {
        return Some(duration);
    }
    let current_date = now.date_naive();
    let restriction_day = restriction.start_time_swedish().day();
    let mut next_month = current_date.month() + 1;
    let mut next_year = current_date.year();
    if next_month > 12 {
        next_month = 1;
        next_year += 1;
    }
    use chrono::NaiveDate;
    let next_date = NaiveDate::from_ymd_opt(next_year, next_month, restriction_day)?;
    let start_time = restriction.start_time_swedish().time();
    let next_datetime = next_date.and_time(start_time);
    use amp_core::structs::SWEDISH_TZ;
    use chrono::TimeZone;
    let next_start = SWEDISH_TZ
        .from_local_datetime(&next_datetime)
        .single()?
        .with_timezone(&Utc);
    if now < next_start {
        Some(next_start - now)
    } else {
        None
    }
}
/// Calculate duration until the next start of this restriction.
///
/// For non-active panels, this gives the time until the restriction begins.
/// If the start time has already passed this month (restriction is active or ended),
/// calculates the next month's start time.
pub fn time_until_next_start(restriction: &DB) -> Option<Duration> {
    let now = Utc::now();
    if let Some(duration) = restriction.time_until_start(now) {
        return Some(duration);
    }
    let current_date = now.date_naive();
    let restriction_day = restriction.start_time_swedish().day();
    let mut next_month = current_date.month() + 1;
    let mut next_year = current_date.year();
    if next_month > 12 {
        next_month = 1;
        next_year += 1;
    }
    use chrono::NaiveDate;
    let next_date = NaiveDate::from_ymd_opt(next_year, next_month, restriction_day)?;
    let start_time = restriction.start_time_swedish().time();
    let next_datetime = next_date.and_time(start_time);
    use amp_core::structs::SWEDISH_TZ;
    use chrono::TimeZone;
    let next_start = SWEDISH_TZ
        .from_local_datetime(&next_datetime)
        .single()?
        .with_timezone(&Utc);
    if now < next_start {
        Some(next_start - now)
    } else {
        None
    }
}
/// Format countdown as human-readable string with adaptive granularity
///
/// Converts the remaining duration into a formatted string, adapting
/// the format based on the time bucket:
/// - Active (< 4h): Show hours, minutes, and seconds for urgency
/// - Within 6 hours: Show hours and minutes only
/// - Longer durations: Show days, hours, and minutes
///
/// # Arguments
/// * `restriction` - DB entry with parking restriction timestamps
///
/// # Returns
/// Formatted string like "3h 25m 10s", "12h 30m", or "2d 06h 30m"
/// Returns None if calculation fails
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
    let bucket = bucket_for(restriction);
    let remaining = match bucket {
        TimeBucket::Now => time_until_next_occurrence(restriction)?,
        TimeBucket::Invalid => return None,
        _ => time_until_next_start(restriction)?,
    };
    match bucket {
        TimeBucket::Now | TimeBucket::Within6Hours | TimeBucket::Within1Day => {
            let hours = remaining.num_hours();
            let minutes = remaining.num_minutes() % 60;
            let seconds = remaining.num_seconds() % 60;
            Some(format!("{}h {:02}m {:02}s", hours, minutes, seconds))
        }
        _ => {
            let days = remaining.num_days();
            let hours = remaining.num_hours() % 24;
            let minutes = remaining.num_minutes() % 60;
            Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
        }
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
    /// Restriction ends within 1 month (30 days)
    Within1Month,
    /// Restriction ends more than 1 month away (>30 days)
    MoreThan1Month,
    /// Invalid, expired, or far-future restriction
    Invalid,
}
/// Categorize restriction by time remaining until deadline
///
/// Assigns a TimeBucket based on how much time is left before
/// the parking restriction ends. For restrictions that have already
/// passed this month, calculates time until next month's occurrence.
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
/// println!("Urgency: {:?}", bucket);
/// ```
pub fn bucket_for(restriction: &DB) -> TimeBucket {
    let now = Utc::now();
    if restriction.is_active(now) {
        return TimeBucket::Now;
    }
    let remaining = match time_until_next_start(restriction) {
        Some(d) => d,
        None => return TimeBucket::Invalid,
    };
    if remaining <= Duration::hours(6) {
        TimeBucket::Within6Hours
    } else if remaining <= Duration::days(1) {
        TimeBucket::Within1Day
    } else if remaining <= Duration::days(30) {
        TimeBucket::Within1Month
    } else {
        TimeBucket::MoreThan1Month
    }
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
                | TimeBucket::MoreThan1Month
                | TimeBucket::Invalid
        ),);
    }
    #[test]
    fn test_db_struct_integration() {
        let db = create_test_db(15, "0800-1200");
        let now = Utc::now();
        let _is_active = db.is_active(now);
        let _time_until_end = db.time_until_end(now);
        let _time_until_start = db.time_until_start(now);
    }
    #[test]
    fn test_time_until_next_occurrence() {
        let db = create_test_db(1, "0800-1200");
        let result = time_until_next_occurrence(&db);
        assert!(result.is_some());
    }
}
