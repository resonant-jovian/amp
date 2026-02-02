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
