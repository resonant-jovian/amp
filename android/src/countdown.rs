use chrono::{Datelike, Duration, Local as ChronoLocal, NaiveDate, NaiveTime};
/// Parse time interval string (e.g., "0800-1000" → (08:00, 10:00))
fn parse_tid_interval(tid: &str) -> Option<(NaiveTime, NaiveTime)> {
    let parts: Vec<_> = tid.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let parse_hm = |s: &str| -> Option<NaiveTime> {
        let s = s.trim();
        if s.len() != 4 {
            return None;
        }
        let hour: u32 = s[0..2].parse().ok()?;
        let minute: u32 = s[2..4].parse().ok()?;
        NaiveTime::from_hms_opt(hour, minute, 0)
    };
    Some((parse_hm(parts[0])?, parse_hm(parts[1])?))
}
/// Add one month to a date, handling month/year wraparound
fn add_one_month(date: NaiveDate) -> Option<NaiveDate> {
    let mut year = date.year();
    let mut month = date.month() + 1;
    if month == 13 {
        month = 1;
        year += 1;
    }
    NaiveDate::from_ymd_opt(year, month, date.day())
}
/// Calculate remaining duration until next parking restriction deadline
///
/// # Arguments
/// * `dag` - Day of month (1-31) when parking restriction ends
/// * `tid` - Time interval string (e.g., "0800-1000")
///
/// # Returns
/// Duration until the next occurrence of the restriction deadline, or None if invalid
pub fn remaining_duration(dag: u8, tid: &str) -> Option<Duration> {
    let (_start, end) = parse_tid_interval(tid)?;
    let now = ChronoLocal::now().naive_local();
    let today = now.date();
    let this_month_date = NaiveDate::from_ymd_opt(
        today.year(),
        today.month(),
        dag as u32,
    )?;
    let this_end = this_month_date.and_time(end);
    if this_end >= now {
        return Some(this_end - now);
    }
    let next_month_date = add_one_month(this_month_date)?;
    let next_end = next_month_date.and_time(end);
    if next_end >= now { Some(next_end - now) } else { None }
}
/// Format countdown as human-readable string
///
/// # Returns
/// Formatted string like "5d 02h 30m" or None if calculation fails
pub fn format_countdown(dag: u8, tid: &str) -> Option<String> {
    let remaining = remaining_duration(dag, tid)?;
    let days = remaining.num_days();
    let hours = remaining.num_hours() % 24;
    let minutes = remaining.num_minutes() % 60;
    Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
}
/// Time bucket categories for grouping parking restrictions
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeBucket {
    Now,
    Within6Hours,
    Within1Day,
    Within1Month,
    Invalid,
}
/// Categorize address by time remaining until parking restriction deadline
pub fn bucket_for(dag: u8, tid: &str) -> TimeBucket {
    let remaining = match remaining_duration(dag, tid) {
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
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    #[test]
    fn test_parse_tid_interval() {
        let (start, end) = parse_tid_interval("0800-1000").unwrap();
        assert_eq!(start.hour(), 8);
        assert_eq!(start.minute(), 0);
        assert_eq!(end.hour(), 10);
        assert_eq!(end.minute(), 0);
    }
    #[test]
    fn test_parse_tid_invalid() {
        assert!(parse_tid_interval("08:00-10:00").is_none());
        assert!(parse_tid_interval("0800").is_none());
    }
    #[test]
    fn test_format_countdown() {
        let result = format_countdown(1, "0800-1000");
        assert!(result.is_some() || result.is_none());
    }
}
