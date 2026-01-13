use crate::error::{AMPError, Result};
use crate::models::{CleaningEvent, CleaningSchedule};
use chrono::{Datelike, Timelike};
use std::collections::HashMap;

pub struct CorrelationAnalyzer {
    min_confidence: f64,
}

impl CorrelationAnalyzer {
    pub fn new(min_confidence: f64) -> Self {
        Self { min_confidence }
    }

    pub fn analyze(&self, events: Vec<CleaningEvent>) -> Result<HashMap<String, CleaningSchedule>> {
        let mut by_address: HashMap<String, Vec<CleaningEvent>> = HashMap::new();

        for event in events {
            by_address
                .entry(event.address.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }

        let mut schedules = HashMap::new();

        for (address, mut address_events) in by_address {
            address_events.sort_by_key(|e| e.timestamp);

            let cleaning_events: Vec<_> = address_events
                .iter()
                .filter(|e| e.is_active)
                .collect();

            if cleaning_events.len() < 2 {
                continue;
            }

            if let Ok(schedule) = self.analyze_address(&address, cleaning_events) {
                if schedule.confidence >= self.min_confidence {
                    schedules.insert(address, schedule);
                }
            }
        }

        Ok(schedules)
    }

    fn analyze_address(
        &self,
        address: &str,
        events: Vec<&CleaningEvent>,
    ) -> Result<CleaningSchedule> {
        let timestamps: Vec<_> = events.iter().map(|e| e.timestamp).collect();

        let mut intervals = Vec::new();
        for i in 1..timestamps.len() {
            let duration = timestamps[i] - timestamps[i - 1];
            let hours = duration.num_seconds() as f64 / 3600.0;
            intervals.push(hours);
        }

        if intervals.is_empty() {
            return Err(AMPError::CorrelationFailed(
                "No intervals calculated".to_string(),
            ));
        }

        let interval_hours = self.find_dominant_interval(&intervals)?;
        let confidence = self.calculate_confidence(&intervals, interval_hours);
        let day_of_week = self.get_dominant_day(&events);
        let time_of_day = self.get_dominant_time(&events);

        let last_cleaning = *timestamps.last().unwrap();
        let next_cleaning = last_cleaning + chrono::Duration::hours(interval_hours.round() as i64);

        Ok(CleaningSchedule {
            address: address.to_string(),
            coordinate: events[0].coordinate,
            next_cleaning,
            frequency_hours: interval_hours,
            confidence,
            day_of_week,
            time_of_day,
            last_cleaning,
            sample_size: events.len(),
        })
    }

    fn find_dominant_interval(&self, intervals: &[f64]) -> Result<f64> {
        if intervals.is_empty() {
            return Err(AMPError::CorrelationFailed(
                "No intervals to analyze".to_string(),
            ));
        }

        let min = intervals.iter().copied().fold(f64::INFINITY, f64::min);
        let max = intervals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bin_width = (max - min) / 20.0;

        if bin_width == 0.0 {
            return Ok(intervals[0]);
        }

        let mut bins = vec![0; 20];
        for &interval in intervals {
            let bin_idx = ((interval - min) / bin_width).floor() as usize;
            if bin_idx < 20 {
                bins[bin_idx] += 1;
            }
        }

        let dominant_bin = bins
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .ok_or_else(|| AMPError::CorrelationFailed("Empty histogram".to_string()))?
            .0;

        let interval = min + (dominant_bin as f64 + 0.5) * bin_width;
        Ok(interval)
    }

    fn calculate_confidence(&self, intervals: &[f64], expected: f64) -> f64 {
        if intervals.len() < 2 {
            return 0.0;
        }

        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let variance = intervals
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / intervals.len() as f64;
        let std_dev = variance.sqrt();

        let cv = std_dev / expected;
        (1.0 - cv).max(0.0).min(1.0)
    }

    fn get_dominant_day(&self, events: &[&CleaningEvent]) -> String {
        let mut day_counts = [0; 7];
        for event in events {
            let weekday = event.timestamp.weekday();
            let day_idx = weekday.number_from_monday() as usize;
            if day_idx > 0 {
                day_counts[day_idx - 1] += 1;
            }
        }

        let dominant_idx = day_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let days = [
            "Monday", "Tuesday", "Wednesday", "Thursday",
            "Friday", "Saturday", "Sunday",
        ];
        days[dominant_idx].to_string()
    }

    fn get_dominant_time(&self, events: &[&CleaningEvent]) -> String {
        let mut hour_counts = [0; 24];
        for event in events {
            let hour = event.timestamp.hour() as usize;
            hour_counts[hour] += 1;
        }

        let dominant_hour = hour_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(hour, _)| hour)
            .unwrap_or(0);

        format!(
            "{:02}:00-{:02}:00",
            dominant_hour,
            (dominant_hour + 1) % 24
        )
    }
}
