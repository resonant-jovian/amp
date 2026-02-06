//! Core data structures for parking restrictions and address management.
//!
//! This module provides the fundamental types used throughout the application:
//!
//! - **Address Types**: [`AdressClean`], [`StoredAddress`] for representing locations
//! - **Parking Data**: [`MiljoeDataClean`], [`ParkeringsDataClean`] for restriction zones
//! - **User Data**: [`LocalData`] for saved addresses with matched parking info
//! - **Time-Based Restrictions**: [`DB`] with Swedish timezone-aware timestamps
//! - **Settings**: [`SettingsData`] for user preferences
//! - **Correlation Results**: [`OutputData`], [`CorrelationResult`] for matching outcomes
//!
//! # Time Handling
//!
//! All time-based operations use [`SWEDISH_TZ`] (Europe/Stockholm) with automatic
//! DST handling. Times are stored in UTC but always interpreted in Swedish timezone.
//!
//! # Examples
//!
//! ## Creating a Time-Based Restriction
//!
//! ```
//! use amp_core::structs::{DB, DBParams};
//!
//! let restriction = DB::from_params(DBParams {
//!     postnummer: Some("21438".to_string()),
//!     adress: "Storgatan 10".to_string(),
//!     gata: Some("Storgatan".to_string()),
//!     gatunummer: Some("10".to_string()),
//!     info: Some("Street cleaning".to_string()),
//!     dag: 15,
//!     tid: "0800-1200".to_string(),
//!     taxa: None,
//!     antal_platser: None,
//!     typ_av_parkering: None,
//!     year: 2024,
//!     month: 1,
//! });
//! assert!(restriction.is_some());
//! ```
//!
//! ## Checking if a Restriction is Active
//!
//! ```
//! use amp_core::structs::DB;
//! use chrono::Utc;
//!
//! # let db = DB::from_dag_tid(
//! #     None, "Test".to_string(), None, None, None,
//! #     15, "0800-1200", None, None, None, 2024, 1
//! # ).unwrap();
//! let now = Utc::now();
//! if db.is_active(now) {
//!     println!("Parking restriction is currently active!");
//! }
//! ```
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::{Europe::Stockholm, Tz};
use rust_decimal::Decimal;
/// Swedish timezone constant for all time operations.
///
/// This is set to `Europe/Stockholm` and automatically handles:
/// - Summer time (CEST, UTC+2) from late March to late October
/// - Winter time (CET, UTC+1) for the rest of the year
///
/// All [`DB`] timestamps are stored in UTC but interpreted in this timezone
/// for display and active status calculations.
pub const SWEDISH_TZ: Tz = Stockholm;
/// Clean address data with coordinates and postal information.
///
/// Represents a single address point loaded from GeoJSON, typically from
/// Swedish municipal address registries (folkbokföringsadress).
///
/// # Fields
///
/// - `coordinates`: `[longitude, latitude]` in WGS84 (EPSG:4326)
/// - `postnummer`: Optional 5-digit Swedish postal code
/// - `adress`: Full address string (e.g., "Storgatan 10")
/// - `gata`: Street name only (e.g., "Storgatan")
/// - `gatunummer`: Street number with optional building code (e.g., "10", "10A")
#[derive(Debug, Clone)]
pub struct AdressClean {
    pub coordinates: [Decimal; 2],
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}
/// Environmental parking restriction data (street cleaning zones).
///
/// Represents a line segment with time-restricted parking, typically for
/// street cleaning (städning). Each instance represents one segment of a
/// LineString or MultiLineString from GeoJSON.
///
/// # Fields
///
/// - `coordinates`: `[[x1, y1], [x2, y2]]` - start and end points of line segment
/// - `info`: Restriction description (e.g., "Parkering förbjuden")
/// - `tid`: Time range string in format "HHMM-HHMM" (e.g., "0800-1200")
/// - `dag`: Day of month when restriction applies (1-31)
#[derive(Debug, Clone)]
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub info: String,
    pub tid: String,
    pub dag: u8,
}
/// Parking zone data with pricing information.
///
/// Represents a paid parking zone segment, typically with hourly rates
/// organized into tiers (Taxa A, B, C, etc.). Each instance represents
/// one segment of a LineString or MultiLineString from GeoJSON.
///
/// # Fields
///
/// - `coordinates`: `[[x1, y1], [x2, y2]]` - start and end points of line segment
/// - `taxa`: Pricing tier (e.g., "Taxa A", "Taxa B", "Taxa C")
/// - `antal_platser`: Number of parking spaces in this segment
/// - `typ_av_parkering`: Parking type (e.g., "Längsgående 6" for parallel parking)
#[derive(Debug, Clone)]
pub struct ParkeringsDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub taxa: String,
    pub antal_platser: u64,
    pub typ_av_parkering: String,
}
/// Correlation result combining address and parking information.
///
/// This is the result of spatial matching between an address and nearby
/// parking restrictions. It contains optional matches for both environmental
/// restrictions (miljö) and parking zones.
///
/// # Examples
///
/// ```
/// use amp_core::structs::OutputData;
///
/// let output = OutputData {
///     postnummer: Some("21438".to_string()),
///     adress: "Storgatan 10".to_string(),
///     gata: "Storgatan".to_string(),
///     gatunummer: "10".to_string(),
///     info: Some("Street cleaning".to_string()),
///     tid: Some("0800-1200".to_string()),
///     dag: Some(15),
///     taxa: Some("Taxa C".to_string()),
///     antal_platser: Some(26),
///     typ_av_parkering: Some("Längsgående 6".to_string()),
/// };
///
/// assert!(output.has_match());
/// assert_eq!(output.dataset_source(), "Both (Miljödata + Parkering)");
/// ```
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
/// User's saved address with matched parking information and active status.
///
/// This represents a user-added address that has been matched against the
/// parking database. It includes validation state and active/inactive status
/// for notifications.
///
/// # Fields
///
/// - `valid`: Whether the address was successfully matched in the database
/// - `active`: Whether notifications are enabled for this address
/// - Other fields: Same as [`OutputData`] but with optional street components
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
/// User-stored address awaiting correlation with parking database.
///
/// This represents a minimal address entry (just the address string) that
/// needs to be fuzzy-matched against the parking database. Equivalent to
/// a user clicking "Add Address" in the UI.
///
/// # Examples
///
/// ```
/// use amp_core::structs::StoredAddress;
///
/// let stored = StoredAddress::new("Kornettsgatan 18C".to_string());
/// // Later, match against database:
/// // let local_data = stored.to_local_data(&static_data);
/// ```
#[derive(Debug, Clone)]
pub struct StoredAddress {
    pub adress: String,
}
impl StoredAddress {
    /// Create a new stored address from an address string.
    ///
    /// This mimics the user typing an address and clicking "Add" in the UI.
    ///
    /// # Arguments
    ///
    /// * `adress` - Full address string (e.g., "Kornettsgatan 18C")
    pub fn new(adress: String) -> Self {
        Self { adress }
    }
    /// Convert to [`LocalData`] by fuzzy-matching against the parking database.
    ///
    /// This performs the address matching step that happens when loading saved
    /// addresses. It uses fuzzy matching to handle variations in formatting.
    ///
    /// # Arguments
    ///
    /// * `static_data` - Array of `(address_string, DB)` tuples from parking database
    ///
    /// # Returns
    ///
    /// `Some(LocalData)` if a match is found, `None` otherwise.
    ///
    /// # Matching Logic
    ///
    /// 1. Parse address into street name and number
    /// 2. Normalize both strings (lowercase, remove non-alphanumeric)
    /// 3. Check if street names match (case-insensitive)
    /// 4. Check if street numbers match (ignoring building codes like U1, U4)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amp_core::structs::{StoredAddress, DB};
    ///
    /// let stored = StoredAddress::new("Kornettsgatan 18C".to_string());
    /// # let static_data: Vec<(String, DB)> = vec![];
    /// let local_data = stored.to_local_data(&static_data);
    /// if let Some(data) = local_data {
    ///     println!("Matched: {}", data.adress);
    /// }
    /// ```
    pub fn to_local_data(&self, static_data: &[(String, DB)]) -> Option<LocalData> {
        println!(
            "[StoredAddress::to_local_data] Attempting to match: '{}'",
            self.adress,
        );
        let (street, number) = Self::parse_address(&self.adress);
        println!(
            "[StoredAddress::to_local_data] Parsed -> street: '{}', number: '{}'",
            street, number,
        );
        for (stored_addr, db) in static_data {
            if Self::fuzzy_match(stored_addr, &self.adress, &street, &number) {
                println!(
                    "[StoredAddress::to_local_data] ✅ MATCH! User: '{}' <-> DB: '{}'",
                    self.adress, stored_addr,
                );
                return Some(LocalData {
                    valid: true,
                    active: false,
                    postnummer: db.postnummer.clone(),
                    adress: self.adress.clone(),
                    gata: db.gata.clone(),
                    gatunummer: db.gatunummer.clone(),
                    info: db.info.clone(),
                    tid: Self::format_time_range(db),
                    dag: Self::extract_day_from_db(db),
                    taxa: db.taxa.clone(),
                    antal_platser: db.antal_platser,
                    typ_av_parkering: db.typ_av_parkering.clone(),
                });
            }
        }
        println!(
            "[StoredAddress::to_local_data] ❌ No match found for: '{}'",
            self.adress,
        );
        None
    }
    /// Parse address into street name and number components.
    ///
    /// # Arguments
    ///
    /// * `address` - Full address string
    ///
    /// # Returns
    ///
    /// Tuple of `(street_name, street_number)`. If no number is found,
    /// returns `(full_address, empty_string)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use amp_core::structs::StoredAddress;
    /// let (street, number) = StoredAddress::parse_address("Kornettsgatan 18C");
    /// assert_eq!(street, "Kornettsgatan");
    /// assert_eq!(number, "18C");
    /// ```
    fn parse_address(address: &str) -> (String, String) {
        let parts: Vec<&str> = address.split_whitespace().collect();
        if let Some(last) = parts.last()
            && last.chars().any(|c| c.is_ascii_digit())
        {
            let street = parts[..parts.len() - 1].join(" ");
            return (street, last.to_string());
        }
        (address.to_string(), String::new())
    }
    /// Fuzzy matching logic for address comparison.
    ///
    /// Matches if:
    /// 1. Street names match (case-insensitive, ignoring diacritics)
    /// 2. Numbers match (ignoring building codes like U1, U4, etc.)
    ///
    /// # Arguments
    ///
    /// * `db_address` - Address from database
    /// * `user_address` - Address entered by user
    /// * `street` - Parsed street name from user address
    /// * `number` - Parsed street number from user address
    fn fuzzy_match(db_address: &str, user_address: &str, street: &str, number: &str) -> bool {
        let normalize = |s: &str| {
            s.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect::<String>()
        };
        let db_norm = normalize(db_address);
        let _user_norm = normalize(user_address);
        let street_norm = normalize(street);
        if !db_norm.contains(&street_norm) {
            return false;
        }
        if !number.is_empty() {
            let number_digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
            if !number_digits.is_empty() {
                return db_norm.contains(&number_digits);
            }
        }
        true
    }
    /// Format DB time range for display.
    ///
    /// Converts Swedish timezone timestamps to "HHMM-HHMM" format.
    fn format_time_range(db: &DB) -> Option<String> {
        let start = db.start_time_swedish();
        let end = db.end_time_swedish();
        Some(format!(
            "{:02}{:02}-{:02}{:02}",
            start.hour(),
            start.minute(),
            end.hour(),
            end.minute(),
        ))
    }
    /// Extract day of month from DB entry.
    fn extract_day_from_db(db: &DB) -> Option<u8> {
        Some(db.start_time_swedish().day() as u8)
    }
}
/// Parameters for creating a [`DB`] entry from day and time strings.
///
/// This struct groups all parameters needed to create a time-based parking
/// restriction, making function calls cleaner than passing 12 individual arguments.
///
/// # Examples
///
/// ```
/// use amp_core::structs::{DB, DBParams};
///
/// let params = DBParams {
///     postnummer: Some("21438".to_string()),
///     adress: "Storgatan 10".to_string(),
///     gata: Some("Storgatan".to_string()),
///     gatunummer: Some("10".to_string()),
///     info: Some("Street cleaning".to_string()),
///     dag: 15,
///     tid: "0800-1200".to_string(),
///     taxa: None,
///     antal_platser: None,
///     typ_av_parkering: None,
///     year: 2024,
///     month: 1,
/// };
///
/// let db = DB::from_params(params);
/// assert!(db.is_some());
/// ```
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
/// Time-aware parking restriction with Swedish timezone support.
///
/// Represents a parking restriction that applies during a specific time window
/// on a specific day of a month. Times are stored in UTC but always interpreted
/// in Swedish timezone (Europe/Stockholm) to handle DST transitions correctly.
///
/// # Time Range and Validation
///
/// - **Valid years**: 2020-2100 (prevents overflow and ensures realistic dates)
/// - **Valid months**: 1-12
/// - **Valid days**: 1-31 (validated by chrono)
/// - **Valid time format**: "HHMM-HHMM" (e.g., "0800-1200")
/// - **Timezone**: All times stored in UTC, displayed in [`SWEDISH_TZ`]
/// - **DST**: Automatic handling of summer/winter time shifts
///
/// # Examples
///
/// ## Creating a Restriction
///
/// ```
/// use amp_core::structs::DB;
///
/// let db = DB::from_dag_tid(
///     Some("22100".to_string()),
///     "Storgatan 10".to_string(),
///     Some("Storgatan".to_string()),
///     Some("10".to_string()),
///     Some("Street cleaning".to_string()),
///     15,                // Day of month
///     "0800-1200",      // Time range
///     None,
///     None,
///     None,
///     2024,             // Year
///     1,                // Month
/// );
/// assert!(db.is_some());
/// ```
///
/// ## Checking Active Status
///
/// ```
/// use amp_core::structs::DB;
/// use chrono::Utc;
///
/// # let db = DB::from_dag_tid(
/// #     None, "Test".to_string(), None, None, None,
/// #     15, "0800-1200", None, None, None, 2024, 1
/// # ).unwrap();
/// let now = Utc::now();
/// if db.is_active(now) {
///     println!("Restriction is active!");
/// }
///
/// if let Some(duration) = db.time_until_end(now) {
///     println!("Ends in {} minutes", duration.num_minutes());
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DB {
    /// Postal code (5-digit Swedish postal code)
    pub postnummer: Option<String>,
    /// Full address string
    pub adress: String,
    /// Street name (gata)
    pub gata: Option<String>,
    /// Street number with optional building code
    pub gatunummer: Option<String>,
    /// Environmental parking restriction info or description
    pub info: Option<String>,
    /// Start time of restriction (stored in UTC, interpreted in Swedish TZ)
    pub start_time: DateTime<Utc>,
    /// End time of restriction (stored in UTC, interpreted in Swedish TZ)
    pub end_time: DateTime<Utc>,
    /// Parking zone/taxa information (e.g., "Taxa C")
    pub taxa: Option<String>,
    /// Number of parking spots in this zone
    pub antal_platser: Option<u64>,
    /// Type of parking (e.g., "Längsgående 6" for parallel parking)
    pub typ_av_parkering: Option<String>,
}
impl DB {
    /// Create a new DB entry from day and time strings (legacy interface).
    ///
    /// # Arguments
    ///
    /// * `postnummer` - Optional 5-digit postal code
    /// * `adress` - Full address string
    /// * `gata` - Optional street name
    /// * `gatunummer` - Optional street number
    /// * `info` - Optional restriction description
    /// * `dag` - Day of month (1-31)
    /// * `tid` - Time range in "HHMM-HHMM" format (e.g., "0800-1200")
    /// * `taxa` - Optional parking zone tier
    /// * `antal_platser` - Optional number of parking spots
    /// * `typ_av_parkering` - Optional parking type
    /// * `year` - Year (must be 2020-2100)
    /// * `month` - Month (1-12)
    ///
    /// # Returns
    ///
    /// - `Some(DB)` if all validations pass and date/time parsing succeeds
    /// - `None` if any validation fails:
    ///   - Invalid year (must be 2020-2100)
    ///   - Invalid month (must be 1-12)
    ///   - Invalid day for the given month
    ///   - Invalid time format (must be "HHMM-HHMM")
    ///   - Time components out of range
    ///
    /// # Note
    ///
    /// Consider using [`from_params`] with [`DBParams`] struct for cleaner code.
    ///
    /// [`from_params`]: Self::from_params
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
    /// Create a new DB entry from [`DBParams`] struct (preferred interface).
    ///
    /// This is the recommended way to create DB entries as it's cleaner than
    /// passing 12 individual arguments.
    ///
    /// # Arguments
    ///
    /// * `params` - [`DBParams`] struct containing all required fields
    ///
    /// # Returns
    ///
    /// - `Some(DB)` if all validations pass
    /// - `None` if validation fails (see [`from_dag_tid`] for details)
    ///
    /// # Overflow Prevention
    ///
    /// Years are validated to be in range 2020-2100 to prevent:
    /// - DateTime overflow (chrono supports years -262144 to 262143)
    /// - Unrealistic dates that could cause calculation errors
    /// - Future-proofing while allowing reasonable historical data
    ///
    /// # Examples
    ///
    /// ```
    /// use amp_core::structs::{DB, DBParams};
    ///
    /// let db = DB::from_params(DBParams {
    ///     postnummer: Some("21438".to_string()),
    ///     adress: "Åhusgatan 1".to_string(),
    ///     gata: Some("Åhusgatan".to_string()),
    ///     gatunummer: Some("1".to_string()),
    ///     info: Some("Parkering förbjuden".to_string()),
    ///     dag: 17,
    ///     tid: "1200-1600".to_string(),
    ///     taxa: Some("Taxa C".to_string()),
    ///     antal_platser: Some(26),
    ///     typ_av_parkering: Some("Längsgående 6".to_string()),
    ///     year: 2024,
    ///     month: 1,
    /// });
    /// assert!(db.is_some());
    /// ```
    ///
    /// [`from_dag_tid`]: Self::from_dag_tid
    pub fn from_params(params: DBParams) -> Option<Self> {
        if !(2020..=2100).contains(&params.year) {
            eprintln!("[DB] Invalid year: {} (must be 2020-2100)", params.year);
            return None;
        }
        if !(1..=12).contains(&params.month) {
            eprintln!("[DB] Invalid month: {} (must be 1-12)", params.month);
            return None;
        }
        let parts: Vec<&str> = params.tid.split('-').collect();
        if parts.len() != 2 {
            eprintln!(
                "[DB] Invalid time format: '{}' (expected HHMM-HHMM)",
                params.tid
            );
            return None;
        }
        let parse_hhmm = |s: &str| -> Option<NaiveTime> {
            let s = s.trim();
            if s.len() != 4 {
                return None;
            }
            let hour: u32 = s[0..2].parse().ok()?;
            let minute: u32 = s[2..4].parse().ok()?;
            NaiveTime::from_hms_opt(hour, minute, 0)
        };
        let start_naive_time = parse_hhmm(parts[0])?;
        let end_naive_time = parse_hhmm(parts[1])?;
        let date = NaiveDate::from_ymd_opt(params.year, params.month, params.dag as u32)?;
        let start_datetime = date.and_time(start_naive_time);
        let end_datetime = date.and_time(end_naive_time);
        let start_time = SWEDISH_TZ
            .from_local_datetime(&start_datetime)
            .single()?
            .with_timezone(&Utc);
        let end_time = SWEDISH_TZ
            .from_local_datetime(&end_datetime)
            .single()?
            .with_timezone(&Utc);
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
    /// Check if the restriction is currently active.
    ///
    /// # Arguments
    ///
    /// * `now` - Current time in UTC (will be compared against stored UTC times)
    ///
    /// # Returns
    ///
    /// `true` if `now` falls within `[start_time, end_time)`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use amp_core::structs::DB;
    /// use chrono::Utc;
    ///
    /// # let db = DB::from_dag_tid(
    /// #     None, "Test".to_string(), None, None, None,
    /// #     15, "0800-1200", None, None, None, 2024, 1
    /// # ).unwrap();
    /// if db.is_active(Utc::now()) {
    ///     println!("Parking restriction is active right now!");
    /// }
    /// ```
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        now >= self.start_time && now < self.end_time
    }
    /// Get duration until restriction starts (if in future).
    ///
    /// # Arguments
    ///
    /// * `now` - Current time in UTC
    ///
    /// # Returns
    ///
    /// - `Some(Duration)` if restriction hasn't started yet
    /// - `None` if restriction has already started or ended
    pub fn time_until_start(&self, now: DateTime<Utc>) -> Option<chrono::Duration> {
        if now < self.start_time {
            Some(self.start_time - now)
        } else {
            None
        }
    }
    /// Get duration until restriction ends (if active or in future).
    ///
    /// # Arguments
    ///
    /// * `now` - Current time in UTC
    ///
    /// # Returns
    ///
    /// - `Some(Duration)` if restriction is active or hasn't started
    /// - `None` if restriction has already ended
    ///
    /// # Examples
    ///
    /// ```
    /// use amp_core::structs::DB;
    /// use chrono::Utc;
    ///
    /// # let db = DB::from_dag_tid(
    /// #     None, "Test".to_string(), None, None, None,
    /// #     15, "0800-1200", None, None, None, 2024, 1
    /// # ).unwrap();
    /// if let Some(duration) = db.time_until_end(Utc::now()) {
    ///     println!("Restriction ends in {} minutes", duration.num_minutes());
    /// }
    /// ```
    pub fn time_until_end(&self, now: DateTime<Utc>) -> Option<chrono::Duration> {
        if now < self.end_time {
            Some(self.end_time - now)
        } else {
            None
        }
    }
    /// Get start time in Swedish timezone for display.
    ///
    /// Converts the internally-stored UTC timestamp to Swedish timezone.
    /// This automatically applies the correct UTC offset based on whether
    /// the date is in summer time (CEST, UTC+2) or winter time (CET, UTC+1).
    ///
    /// # Examples
    ///
    /// ```
    /// use amp_core::structs::DB;
    ///
    /// # let db = DB::from_dag_tid(
    /// #     None, "Test".to_string(), None, None, None,
    /// #     15, "0800-1200", None, None, None, 2024, 1
    /// # ).unwrap();
    /// let swedish_time = db.start_time_swedish();
    /// println!("Restriction starts at {:02}:{:02}",
    ///     swedish_time.hour(), swedish_time.minute());
    /// ```
    pub fn start_time_swedish(&self) -> DateTime<Tz> {
        self.start_time.with_timezone(&SWEDISH_TZ)
    }
    /// Get end time in Swedish timezone for display.
    ///
    /// Converts the internally-stored UTC timestamp to Swedish timezone.
    /// See [`start_time_swedish`] for details on timezone conversion.
    ///
    /// [`start_time_swedish`]: Self::start_time_swedish
    pub fn end_time_swedish(&self) -> DateTime<Tz> {
        self.end_time.with_timezone(&SWEDISH_TZ)
    }
}
/// Result of address-to-parking correlation with distance information.
///
/// This extends [`OutputData`] with optional distance measurements to the
/// matched parking zones, useful for debugging and verification.
#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub address: String,
    pub postnummer: String,
    pub miljo_match: Option<(f64, String)>,
    pub parkering_match: Option<(f64, String)>,
}
impl OutputData {
    /// Check if this address has any parking data matches.
    ///
    /// # Returns
    ///
    /// `true` if either `info` (miljö) or `taxa` (parkering) is present.
    pub fn has_match(&self) -> bool {
        self.info.is_some() || self.taxa.is_some()
    }
    /// Get human-readable description of which datasets matched.
    ///
    /// # Returns
    ///
    /// - `"Both (Miljödata + Parkering)"` if both matched
    /// - `"Miljödata only"` if only environmental restrictions matched
    /// - `"Parkering only"` if only parking zones matched
    /// - `"No match"` if neither matched
    pub fn dataset_source(&self) -> &'static str {
        match (self.info.is_some(), self.taxa.is_some()) {
            (true, true) => "Both (Miljödata + Parkering)",
            (true, false) => "Miljödata only",
            (false, true) => "Parkering only",
            (false, false) => "No match",
        }
    }
}
/// [`OutputData`] extended with distance measurements to matched zones.
///
/// Used for correlation algorithms that track distances to parking zones
/// for debugging and verification purposes.
pub struct OutputDataWithDistance {
    pub data: OutputData,
    pub miljo_distance: Option<f64>,
    pub parkering_distance: Option<f64>,
}
impl OutputDataWithDistance {
    /// Get the closest distance among all matches.
    ///
    /// # Returns
    ///
    /// Minimum of `miljo_distance` and `parkering_distance`, or `None`
    /// if neither distance is available.
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
    /// Get human-readable description of which datasets matched.
    ///
    /// Similar to [`OutputData::dataset_source`] but with different formatting.
    pub fn dataset_source(&self) -> &'static str {
        match (self.miljo_match.is_some(), self.parkering_match.is_some()) {
            (true, true) => "Both (Miljödata & Parkering)",
            (true, false) => "Miljödata only",
            (false, true) => "Parkering only",
            (false, false) => "No match",
        }
    }
}
/// User preferences for notifications, theme, and language.
///
/// This data is persisted in Parquet format and synced with the Android app.
/// Changes to these settings trigger re-generation of the settings Parquet file.
///
/// # Examples
///
/// ```
/// use amp_core::structs::SettingsData;
///
/// let settings = SettingsData {
///     stadning_nu: true,
///     sex_timmar: true,
///     en_dag: false,
///     theme: "Dark".to_string(),
///     language: "English".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsData {
    /// Notify when street cleaning is currently happening
    pub stadning_nu: bool,
    /// Notify 6 hours before street cleaning starts
    pub sex_timmar: bool,
    /// Notify 1 day (24 hours) before street cleaning starts
    pub en_dag: bool,
    /// Theme preference: "Light" or "Dark"
    pub theme: String,
    /// Language: "Svenska", "English", "Espanol", or "Francais"
    pub language: String,
}
impl Default for SettingsData {
    /// Create default settings with Swedish language and light theme.
    ///
    /// Default notification settings:
    /// - `stadning_nu`: `true` (notify during cleaning)
    /// - `sex_timmar`: `true` (notify 6 hours before)
    /// - `en_dag`: `true` (notify 1 day before)
    /// - `theme`: "Light"
    /// - `language`: "Svenska"
    fn default() -> Self {
        Self {
            stadning_nu: true,
            sex_timmar: true,
            en_dag: true,
            theme: "Light".to_string(),
            language: "Svenska".to_string(),
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
    #[test]
    fn test_stored_address_parse() {
        let (street, number) = StoredAddress::parse_address("Kornettsgatan 18C");
        assert_eq!(street, "Kornettsgatan");
        assert_eq!(number, "18C");
    }
}
