//! Classification data management for AMP testing interface
//! Stores human-reviewed classification data to local JSON files in Documents folder

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

/// Get the classification data directory (Documents/amp_classifications)
fn _get_classification_dir() -> Result<PathBuf, String> {
    let _home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let _docs_dir = _home.join("Documents");

    let _class_dir = _docs_dir.join("amp_classifications");

    // Create directory if it doesn't exist
    if !_class_dir.exists() {
        fs::create_dir_all(&_class_dir)
            .map_err(|e| format!("Failed to create classifications directory: {}", e))?;
    }

    Ok(_class_dir)
}

/// Get path to classification JSON file for a category
fn _get_classification_file(_category: &str) -> Result<PathBuf, String> {
    let _dir = _get_classification_dir()?;
    Ok(_dir.join(format!("amp_stadsatlas_{}.json", _category)))
}

/// Classification entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ClassificationEntry {
    pub id: String,
    pub timestamp: String,
    pub address: String,
    pub postal_code: String,
    pub source: String,
    pub matches_html: String,
}

/// Request body for classification
#[derive(Debug, Serialize, Deserialize)]
pub struct _ClassificationRequest {
    pub category: String,
    pub data: _ClassificationData,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct _ClassificationData {
    pub address: String,
    pub postal_code: String,
    pub source: String,
    pub matches_html: String,
}

/// Response body for classification operations
#[derive(Debug, Serialize, Deserialize)]
pub struct _ClassificationResponse {
    pub success: bool,
    pub message: String,
    pub id: Option<String>,
}

/// Add classification entry to JSON file
pub fn _add_classification(_req: &_ClassificationRequest) -> Result<String, String> {
    // Validate category
    if _req.category != "notMatching" && _req.category != "invalid" {
        return Err(format!("Invalid category: {}", _req.category));
    }

    let _file_path = _get_classification_file(&_req.category)?;

    // Generate unique ID
    let _id = format!(
        "{}-{}-{}",
        _req.category,
        chrono::Local::now().timestamp_millis(),
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("xxx")
    );

    // Create entry
    let _entry = _ClassificationEntry {
        id: _id.clone(),
        timestamp: _req.timestamp.clone(),
        address: _req.data.address.clone(),
        postal_code: _req.data.postal_code.clone(),
        source: _req.data.source.clone(),
        matches_html: _req.data.matches_html.clone(),
    };

    // Load or create JSON structure
    let mut _json: Value = if _file_path.exists() {
        let _content = fs::read_to_string(&_file_path)
            .map_err(|e| format!("Failed to read classification file: {}", e))?;
        serde_json::from_str(&_content).unwrap_or_else(|_| json!({ "entries": [] }))
    } else {
        json!({ "entries": [] })
    };

    // Ensure entries array exists
    if !_json["entries"].is_array() {
        _json["entries"] = json!([])
    }

    // Add new entry
    if let Some(_entries) = _json["entries"].as_array_mut() {
        _entries.push(
            serde_json::to_value(&_entry)
                .map_err(|e| format!("Failed to serialize entry: {}", e))?,
        );
    }

    // Write back to file
    let _json_str = serde_json::to_string_pretty(&_json)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(&_file_path, _json_str)
        .map_err(|e| format!("Failed to write classification file: {}", e))?;

    Ok(_id)
}

/// Undo the last classification for a category and address
pub fn _undo_classification(_category: &str, _address: &str) -> Result<String, String> {
    // Validate category
    if _category != "notMatching" && _category != "invalid" {
        return Err(format!("Invalid category: {}", _category));
    }

    let _file_path = _get_classification_file(_category)?;

    // If file doesn't exist, nothing to undo
    if !_file_path.exists() {
        return Err(format!(
            "No classifications found for category: {}",
            _category
        ));
    }

    // Load JSON
    let _content = fs::read_to_string(&_file_path)
        .map_err(|e| format!("Failed to read classification file: {}", e))?;

    let mut _json: Value = serde_json::from_str(&_content)
        .map_err(|e| format!("Failed to parse classification file: {}", e))?;

    // Get entries array
    if let Some(_entries) = _json["entries"].as_array_mut() {
        // Find last entry matching this address
        if let Some(_last_index) = _entries.iter().rposition(|e| {
            e.get("address")
                .and_then(|addr| addr.as_str())
                .map(|addr| addr == _address)
                .unwrap_or(false)
        }) {
            _entries.remove(_last_index);

            // Write back to file
            let _json_str = serde_json::to_string_pretty(&_json)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

            fs::write(&_file_path, _json_str)
                .map_err(|e| format!("Failed to write classification file: {}", e))?;

            return Ok(format!(
                "Undid last classification for '{}' in category '{}'",
                _address, _category
            ));
        }
    }

    Err(format!(
        "No classifications found for '{}' in category '{}'",
        _address, _category
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_classification_dir() {
        let _result = _get_classification_dir();
        assert!(_result.is_ok());
        let _dir = _result.unwrap();
        assert!(_dir.ends_with("amp_classifications"));
    }
}
