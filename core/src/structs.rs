use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::{Europe::Stockholm, Tz};
use rust_decimal::Decimal;
/// Swedish timezone constant for all time operations
pub const SWEDISH_TZ: Tz = Stockholm;
#[derive(Debug, Clone)]
pub struct AdressClean {
    pub coordinates: [Decimal; 2],
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}
#[derive(Debug, Clone)]
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub info: String,
    pub tid: String,
    pub dag: u8,
}
#[derive(Debug, Clone)]
pub struct ParkeringsDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub taxa: String,
    pub antal_platser: u64,
    pub typ_av_parkering: String,
}
#[derive(Debug, Clone)]
pub struct OutputData {
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
    pub info: Option<String>,
    pub tid: Option<String>,
    pub dag: Option<u8>,
    pub taxa: Option<String>,
    pub antal_platser: Option<u64>,
    pub typ_av_parkering: Option<String>,
}
#[derive(Debug, Clone)]
pub struct LocalData {
    pub valid: bool,
    pub active: bool,
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: Option<String>,
    pub gatunummer: Option<String>,
    pub info: Option<String>,
    pub tid: Option<String>,
    pub dag: Option<u8>,
    pub taxa: Option<String>,
    pub antal_platser: Option<u64>,
    pub typ_av_parkering: Option<String>,
}
/// Parameters for creating a DB entry from day and time strings
#[derive(Debug, Clone)]
pub struct DBParams {
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: Option<String>,
    pub gatunummer: Option<String>,
    pub info: Option<String>,
    pub dag: u8,
    pub tid: String,
    pub taxa: Option<String>,
    pub antal_platser: Option<u64>,
    pub typ_av_parkering: Option<String>,
    pub year: i32,
    pub month: u32,
}
/// Database struct for Android component
/// Uses chrono timestamps to represent time intervals within a month
/// Time is always in Swedish timezone (Europe/Stockholm) which handles DST automatically
///
/// # Time Range and Validation
///
/// - Valid years: 2020-2100 (prevents overflow and ensures realistic dates)
/// - Valid months: 1-12
/// - Valid days: 1-31 (validated by chrono)
/// - Valid time format: "HHMM-HHMM" (e.g., "0800-1200")
/// - All times are stored in Swedish timezone (Europe/Stockholm)
/// - Handles summer/winter time shifts automatically
///
/// # Examples
/// ```
/// use amp_core::structs::DB;
/// let db = DB::from_dag_tid(
///     Some("22100".to_string()),
///     "Storgatan 10".to_string(),
///     Some("Storgatan".to_string()),
///     Some("10".to_string()),
///     None,
///     15,
///     "0800-1200",
///     None,
///     None,
///     None,
///     2024,
///     1,
/// );
/// assert!(db.is_some());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DB {
    /// Postal code
    pub postnummer: Option<String>,
    /// Full address string
    pub adress: String,
    /// Street name (gata)
    pub gata: Option<String>,
    /// Street number
    pub gatunummer: Option<String>,
    /// Environmental parking restriction info
    pub info: Option<String>,
    /// Start time of restriction (in Swedish timezone)
    pub start_time: DateTime<Utc>,
    /// End time of restriction (in Swedish timezone)
    pub end_time: DateTime<Utc>,
    /// Parking zone/taxa information
    pub taxa: Option<String>,
    /// Number of parking spots
    pub antal_platser: Option<u64>,
    /// Type of parking (e.g., "Längsgående 6")
    pub typ_av_parkering: Option<String>,
}
impl DB {
    /// Create a new DB entry from day and time strings (legacy interface)
    ///
    /// # Arguments
    /// All the individual parameters needed to create a DB entry
    ///
    /// # Returns
    /// - `Some(DB)` if all validations pass and date/time parsing succeeds
    /// - `None` if any validation fails:
    ///   - Invalid year (must be 2020-2100)
    ///   - Invalid month (must be 1-12)
    ///   - Invalid day for the given month
    ///   - Invalid time format (must be "HHMM-HHMM")
    ///   - Time components out of range
    ///
    /// # Deprecated
    /// Consider using `from_params` with DBParams struct for cleaner code
    #[allow(clippy::too_many_arguments)]
    pub fn from_dag_tid(
        postnummer: Option<String>,
        adress: String,
        gata: Option<String>,
        gatunummer: Option<String>,
        info: Option<String>,
        dag: u8,
        tid: &str,
        taxa: Option<String>,
        antal_platser: Option<u64>,
        typ_av_parkering: Option<String>,
        year: i32,
        month: u32,
    ) -> Option<Self> {
        Self::from_params(DBParams {
            postnummer,
            adress,
            gata,
            gatunummer,
            info,
            dag,
            tid: tid.to_string(),
            taxa,
            antal_platser,
            typ_av_parkering,
            year,
            month,
        })
    }
    /// Create a new DB entry from DBParams struct (preferred interface)
    ///
    /// # Arguments
    /// * `params` - DBParams struct containing all required fields
    ///
    /// # Returns
    /// - `Some(DB)` if all validations pass
    /// - `None` if validation fails (see `from_dag_tid` for details)
    ///
    /// # Overflow Prevention
    /// Years are validated to be in range 2020-2100 to prevent:
    /// - DateTime overflow (chrono supports years -262144 to 262143)
    /// - Unrealistic dates that could cause calculation errors
    /// - Future-proofing while allowing reasonable historical data
    pub fn from_params(params: DBParams) -> Option<Self> {
        eprintln!(
            "[DB::from_params] Input: year={}, month={}, dag={}, tid='{}', adress='{}'",
            params.year, params.month, params.dag, params.tid, params.adress
        );
        
        if !(2020..=2100).contains(&params.year) {
            eprintln!("[DB] Invalid year: {} (must be 2020-2100)", params.year);
            return None;
        }
        if !(1..=12).contains(&params.month) {
            eprintln!("[DB] Invalid month: {} (must be 1-12)", params.month);
            return None;
        }
        
        let parts: Vec<&str> = params.tid.split('-').collect();
        eprintln!("[DB::from_params] Split tid into {} parts: {:?}", parts.len(), parts);
        
        if parts.len() != 2 {
            eprintln!(
                "[DB] Invalid time format: '{}' (expected HHMM-HHMM, got {} parts)",
                params.tid,
                parts.len()
            );
            return None;
        }
        
        let parse_hhmm = |s: &str| -> Option<NaiveTime> {
            let s = s.trim();
            eprintln!("[DB::from_params] Parsing time component: '{}' (length={})", s, s.len());
            
            if s.len() != 4 {
                eprintln!("[DB::from_params] Time component length != 4: '{}'", s);
                return None;
            }
            
            let hour_str = &s[0..2];
            let minute_str = &s[2..4];
            eprintln!("[DB::from_params] Parsing hour='{}' minute='{}'", hour_str, minute_str);
            
            let hour: u32 = match hour_str.parse() {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[DB::from_params] Failed to parse hour '{}': {}", hour_str, e);
                    return None;
                }
            };
            
            let minute: u32 = match minute_str.parse() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[DB::from_params] Failed to parse minute '{}': {}", minute_str, e);
                    return None;
                }
            };
            
            eprintln!("[DB::from_params] Parsed hour={} minute={}", hour, minute);
            
            match NaiveTime::from_hms_opt(hour, minute, 0) {
                Some(time) => {
                    eprintln!("[DB::from_params] Created NaiveTime: {:?}", time);
                    Some(time)
                }
                None => {
                    eprintln!("[DB::from_params] Invalid time components: hour={} minute={}", hour, minute);
                    None
                }
            }
        };
        
        let start_naive_time = match parse_hhmm(parts[0]) {
            Some(t) => t,
            None => {
                eprintln!("[DB::from_params] Failed to parse start time from '{}'", parts[0]);
                return None;
            }
        };
        
        let end_naive_time = match parse_hhmm(parts[1]) {
            Some(t) => t,
            None => {
                eprintln!("[DB::from_params] Failed to parse end time from '{}'", parts[1]);
                return None;
            }
        };
        
        eprintln!(
            "[DB::from_params] Creating date from year={} month={} day={}",
            params.year, params.month, params.dag
        );
        
        let date = match NaiveDate::from_ymd_opt(params.year, params.month, params.dag as u32) {
            Some(d) => {
                eprintln!("[DB::from_params] Created date: {:?}", d);
                d
            }
            None => {
                eprintln!(
                    "[DB::from_params] Invalid date: year={} month={} day={}",
                    params.year, params.month, params.dag
                );
                return None;
            }
        };
        
        let start_datetime = date.and_time(start_naive_time);
        let end_datetime = date.and_time(end_naive_time);
        
        eprintln!(
            "[DB::from_params] Created naive datetimes: start={:?} end={:?}",
            start_datetime, end_datetime
        );
        
        let start_time = match SWEDISH_TZ.from_local_datetime(&start_datetime).single() {
            Some(dt) => {
                let utc = dt.with_timezone(&Utc);
                eprintln!("[DB::from_params] Converted start time to UTC: {:?}", utc);
                utc
            }
            None => {
                eprintln!(
                    "[DB::from_params] Failed to convert start datetime to Swedish timezone: {:?}",
                    start_datetime
                );
                return None;
            }
        };
        
        let end_time = match SWEDISH_TZ.from_local_datetime(&end_datetime).single() {
            Some(dt) => {
                let utc = dt.with_timezone(&Utc);
                eprintln!("[DB::from_params] Converted end time to UTC: {:?}", utc);
                utc
            }
            None => {
                eprintln!(
                    "[DB::from_params] Failed to convert end datetime to Swedish timezone: {:?}",
                    end_datetime
                );
                return None;
            }
        };
        
        eprintln!(
            "[DB::from_params] SUCCESS! Created DB entry with times: start={:?} end={:?}",
            start_time, end_time
        );
        
        Some(DB {
            postnummer: params.postnummer,
            adress: params.adress,
            gata: params.gata,
            gatunummer: params.gatunummer,
            info: params.info,
            start_time,
            end_time,
            taxa: params.taxa,
            antal_platser: params.antal_platser,
            typ_av_parkering: params.typ_av_parkering,
        })
    }
    /// Check if the restriction is currently active
    ///
    /// # Arguments
    /// * `now` - Current time in UTC (will be converted to Swedish timezone for comparison)
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        now >= self.start_time && now < self.end_time
    }
    /// Get duration until restriction starts (if in future)
    pub fn time_until_start(&self, now: DateTime<Utc>) -> Option<chrono::Duration> {
        if now < self.start_time {
            Some(self.start_time - now)
        } else {
            None
        }
    }
    /// Get duration until restriction ends (if active or in future)
    pub fn time_until_end(&self, now: DateTime<Utc>) -> Option<chrono::Duration> {
        if now < self.end_time {
            Some(self.end_time - now)
        } else {
            None
        }
    }
    /// Get start time in Swedish timezone for display
    pub fn start_time_swedish(&self) -> DateTime<Tz> {
        self.start_time.with_timezone(&SWEDISH_TZ)
    }
    /// Get end time in Swedish timezone for display
    pub fn end_time_swedish(&self) -> DateTime<Tz> {
        self.end_time.with_timezone(&SWEDISH_TZ)
    }
}
/// Result of correlation for a single address
#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub address: String,
    pub postnummer: String,
    pub miljo_match: Option<(f64, String)>,
    pub parkering_match: Option<(f64, String)>,
}
impl OutputData {
    /// Check if this address has any matches
    pub fn has_match(&self) -> bool {
        self.info.is_some() || self.taxa.is_some()
    }
    /// Get source description
    pub fn dataset_source(&self) -> &'static str {
        match (self.info.is_some(), self.taxa.is_some()) {
            (true, true) => "Both (Miljödata + Parkering)",
            (true, false) => "Miljödata only",
            (false, true) => "Parkering only",
            (false, false) => "No match",
        }
    }
}
pub struct OutputDataWithDistance {
    pub data: OutputData,
    pub miljo_distance: Option<f64>,
    pub parkering_distance: Option<f64>,
}
impl OutputDataWithDistance {
    pub fn closest_distance(&self) -> Option<f64> {
        match (self.miljo_distance, self.parkering_distance) {
            (Some(m), Some(p)) => Some(m.min(p)),
            (Some(m), None) => Some(m),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        }
    }
}
impl CorrelationResult {
    /// Get source description
    pub fn dataset_source(&self) -> &'static str {
        match (self.miljo_match.is_some(), self.parkering_match.is_some()) {
            (true, true) => "Both (Miljödata & Parkering)",
            (true, false) => "Miljödata only",
            (false, true) => "Parkering only",
            (false, false) => "No match",
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db_from_dag_tid() {
        let db = DB::from_dag_tid(
            Some("21438".to_string()),
            "Åhusgatan1".to_string(),
            Some("Åhusgatan".to_string()),
            Some("1".to_string()),
            Some("Parkering förbjuden".to_string()),
            17,
            "1200-1600",
            Some("Taxa C".to_string()),
            Some(26),
            Some("Längsgående 6".to_string()),
            2024,
            1,
        );
        assert!(db.is_some());
        let db = db.unwrap();
        assert_eq!(db.adress, "Åhusgatan1");
    }
    #[test]
    fn test_db_from_params() {
        let db = DB::from_params(DBParams {
            postnummer: Some("21438".to_string()),
            adress: "Åhusgatan1".to_string(),
            gata: Some("Åhusgatan".to_string()),
            gatunummer: Some("1".to_string()),
            info: Some("Parkering förbjuden".to_string()),
            dag: 17,
            tid: "1200-1600".to_string(),
            taxa: Some("Taxa C".to_string()),
            antal_platser: Some(26),
            typ_av_parkering: Some("Längsgående 6".to_string()),
            year: 2024,
            month: 1,
        });
        assert!(db.is_some());
        let db = db.unwrap();
        assert_eq!(db.adress, "Åhusgatan1");
    }
    #[test]
    fn test_db_is_active() {
        let db = DB::from_dag_tid(
            None,
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2024,
            1,
        )
        .unwrap();
        let during = DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            Utc,
        );
        assert!(db.is_active(during));
        let before = DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_opt(6, 0, 0)
                .unwrap(),
            Utc,
        );
        assert!(!db.is_active(before));
    }
    #[test]
    fn test_year_validation() {
        let db = DB::from_dag_tid(
            None,
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2019,
            1,
        );
        assert!(db.is_none());
        let db = DB::from_dag_tid(
            None,
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2101,
            1,
        );
        assert!(db.is_none());
        let db = DB::from_dag_tid(
            None,
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2024,
            1,
        );
        assert!(db.is_some());
    }
    #[test]
    fn test_swedish_timezone() {
        let db = DB::from_dag_tid(
            None,
            "Test".to_string(),
            None,
            None,
            None,
            15,
            "0800-1200",
            None,
            None,
            None,
            2024,
            1,
        )
        .unwrap();
        let swedish_time = db.start_time_swedish();
        assert_eq!(swedish_time.timezone(), SWEDISH_TZ);
    }
}
