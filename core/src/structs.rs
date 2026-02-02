use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

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

/// Database struct for Android component
/// Uses chrono timestamps to represent time intervals within a month
/// Time is counted from second 0 of the month (start of month)
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
    /// Start time of restriction (seconds from month start)
    pub start_time: DateTime<Utc>,
    /// End time of restriction (seconds from month start)
    pub end_time: DateTime<Utc>,
    /// Parking zone/taxa information
    pub taxa: Option<String>,
    /// Number of parking spots
    pub antal_platser: Option<u64>,
    /// Type of parking (e.g., "Längsgående 6")
    pub typ_av_parkering: Option<String>,
}

impl DB {
    /// Create a new DB entry from day and time strings
    /// 
    /// # Arguments
    /// * `dag` - Day of month (1-31)
    /// * `tid` - Time range string (e.g., "0800-1200")
    /// * `year` - Year for the timestamp
    /// * `month` - Month for the timestamp (1-12)
    /// 
    /// # Returns
    /// DB instance with calculated start_time and end_time from month start
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
        use chrono::{NaiveDate, NaiveTime};
        
        // Parse time string (e.g., "0800-1200" -> 08:00 to 12:00)
        let parts: Vec<&str> = tid.split('-').collect();
        if parts.len() != 2 {
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
        
        // Create date for the specific day
        let date = NaiveDate::from_ymd_opt(year, month, dag as u32)?;
        
        // Combine date with times
        let start_datetime = date.and_time(start_naive_time);
        let end_datetime = date.and_time(end_naive_time);
        
        // Convert to UTC
        let start_time = DateTime::<Utc>::from_naive_utc_and_offset(start_datetime, Utc);
        let end_time = DateTime::<Utc>::from_naive_utc_and_offset(end_datetime, Utc);
        
        Some(DB {
            postnummer,
            adress,
            gata,
            gatunummer,
            info,
            start_time,
            end_time,
            taxa,
            antal_platser,
            typ_av_parkering,
        })
    }
    
    /// Check if the restriction is currently active
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
            17, // day
            "1200-1600", // time
            Some("Taxa C".to_string()),
            Some(26),
            Some("Längsgående 6".to_string()),
            2024,
            1, // January
        );
        
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
        ).unwrap();
        
        // Test with a time during restriction
        let during = DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            Utc,
        );
        assert!(db.is_active(during));
        
        // Test with a time before restriction
        let before = DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_opt(6, 0, 0)
                .unwrap(),
            Utc,
        );
        assert!(!db.is_active(before));
    }
}
