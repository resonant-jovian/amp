use crate::models::{AlertLevel, GpsCoordinate};
use std::collections::HashMap;

/// Analyzes cleaning schedule patterns and finds matches
pub struct CorrelationAnalyzer {
    cache: HashMap<String, Vec<(i32, f64)>>,
}

impl CorrelationAnalyzer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Find matching cleaning schedules for a coordinate
    pub fn find_matches(
        &mut self,
        _coordinate: GpsCoordinate,
        all_schedules: &[(&str, i32)], // (street_name, days_until)
    ) -> Vec<(String, AlertLevel, f64)> {
        all_schedules
            .iter()
            .map(|(street, days)| {
                let alert = match days {
                    0 => AlertLevel::Cleaning,
                    1..=6 => AlertLevel::SixHours,
                    7..=24 => AlertLevel::TwentyFours,
                    _ => AlertLevel::None,
                };

                let confidence = self.calculate_confidence(*days);

                (street.to_string(), alert, confidence)
            })
            .filter(|(_, alert, _)| *alert != AlertLevel::None)
            .collect()
    }

    /// Calculate confidence score (0.0 to 1.0)
    fn calculate_confidence(&self, days_until: i32) -> f64 {
        match days_until {
            0 => 1.0,           // Cleaning now = 100% confidence
            1 => 0.95,
            2..=6 => 0.90,
            7..=14 => 0.70,
            15..=30 => 0.40,
            _ => 0.0,
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CorrelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_analysis() {
        let mut analyzer = CorrelationAnalyzer::new();
        let coord = GpsCoordinate::new(55.6050, 13.0038);

        let schedules = vec![
            ("Storgatan", 0),
            ("Nörrvägen", 6),
            ("Södergatan", 24),
            ("Västra vägen", 30),
        ];

        let matches = analyzer.find_matches(coord, &schedules);

        // Should find 3 matches (0 days, 6 days, 24 days)
        assert_eq!(matches.len(), 3);

        // First match should be highest alert
        assert_eq!(matches[0].1, AlertLevel::Cleaning);
    }
}
