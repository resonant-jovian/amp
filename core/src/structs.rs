use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct AdressClean {
    pub coordinates: [Decimal; 2],
    pub postnummer: String,
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

/// Result of correlation for a single address
#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub address: String,
    pub postnummer: String,
    pub miljo_match: Option<(f64, String)>,     // (distance, info)
    pub parkering_match: Option<(f64, String)>, // (distance, info)
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
            (Some((d1, _)), Some((d2, _))) => Some(d1.min(d2)),
            (Some((d, _)), None) => Some(*d),
            (None, Some((d, _))) => Some(*d),
            (None, None) => None,
        }
    }
}
