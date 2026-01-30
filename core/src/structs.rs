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
/// Result of correlation for a single address
#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub address: String,
    pub postnummer: String,
    pub miljo_match: Option<(f64, String)>,
    pub parkering_match: Option<(f64, String)>,
}
impl CorrelationResult {
    pub fn has_match(&self) -> bool {
        self.miljo_match.is_some() || self.parkering_match.is_some()
    }
    pub fn dataset_source(&self) -> String {
        match (self.miljo_match.is_some(), self.parkering_match.is_some()) {
            (true, true) => "Both (Miljödata + Parkering)".to_string(),
            (true, false) => "Miljödata only".to_string(),
            (false, true) => "Parkering only".to_string(),
            (false, false) => "No match".to_string(),
        }
    }
    pub fn closest_distance(&self) -> Option<f64> {
        match (self.miljo_match.as_ref(), self.parkering_match.as_ref()) {
            (Some((d1, _)), Some((d2, _))) => Some(d1.min(*d2)),
            (Some((d, _)), None) => Some(*d),
            (None, Some((d, _))) => Some(*d),
            (None, None) => None,
        }
    }
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
