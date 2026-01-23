//! Checksum verification for data sources
//! Checks if remote data has changed since last fetch

use chrono::Utc;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataChecksum {
    pub miljo_url: String,
    pub miljo_checksum: String,
    pub parkering_url: String,
    pub parkering_checksum: String,
    pub adresser_url: String,
    pub adresser_checksum: String,
    pub last_checked: String,
}

impl DataChecksum {
    /// Create new checksum record
    pub fn new(miljo_url: String, parkering_url: String, adresser_url: String) -> Self {
        Self {
            miljo_url,
            miljo_checksum: String::new(),
            parkering_url,
            parkering_checksum: String::new(),
            adresser_url,
            adresser_checksum: String::new(),
            last_checked: Utc::now().to_rfc3339(),
        }
    }

    /// Calculate SHA256 checksum of local file
    pub fn calculate_file_checksum(path: &str) -> Result<String, std::io::Error> {
        let data = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Fetch remote URL and calculate checksum
    pub async fn fetch_and_checksum(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response: Response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if any data source has changed
    pub fn has_changed(&self, old_checksum: &DataChecksum) -> bool {
        self.miljo_checksum != old_checksum.miljo_checksum
            || self.parkering_checksum != old_checksum.parkering_checksum
            || self.adresser_checksum != old_checksum.adresser_checksum
    }

    /// Load checksums from file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let checksums: DataChecksum = serde_json::from_str(&content)?;
        Ok(checksums)
    }

    /// Save checksums to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Update all checksums from remote sources
    pub async fn update_from_remote(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.miljo_checksum = Self::fetch_and_checksum(&self.miljo_url).await?;
        self.parkering_checksum = Self::fetch_and_checksum(&self.parkering_url).await?;
        self.adresser_checksum = Self::fetch_and_checksum(&self.adresser_url).await?;
        self.last_checked = Utc::now().to_rfc3339();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_creation() {
        let cs = DataChecksum::new(
            "http://example.com/miljo".to_string(),
            "http://example.com/parkering".to_string(),
            "http://example.com/adresser".to_string(),
        );

        assert!(!cs.miljo_url.is_empty());
        assert!(!cs.last_checked.is_empty());
    }
}
