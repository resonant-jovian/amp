//! Settings state management for Android app
//!
//! Provides persistent storage for user preferences including:
//! - Notification settings (städas nu, 6 hours, 1 day before)
//! - Theme preference (dark/light mode)
//! - Language selection
//! - Debug mode toggle
//!
//! Settings are stored in a JSON file for easy serialization.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
/// Thread-safe settings mutex
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());
#[cfg(target_os = "android")]
const SETTINGS_FILE_NAME: &str = "settings.json";
#[cfg(not(target_os = "android"))]
const SETTINGS_FILE_NAME: &str = "settings.json";
/// Notification timing preferences
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Notify when cleaning is currently happening
    pub stadning_nu: bool,
    /// Notify 6 hours before cleaning
    pub sex_timmar: bool,
    /// Notify 1 day before cleaning
    pub en_dag: bool,
}
impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            stadning_nu: true,
            sex_timmar: true,
            en_dag: false,
        }
    }
}
/// Theme preference
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}
/// Supported languages
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Svenska,
    English,
    Espanol,
    Francais,
}
impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::Svenska => "Svenska",
            Language::English => "English",
            Language::Espanol => "Español",
            Language::Francais => "Français",
        }
    }
}
/// Complete app settings
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub notifications: NotificationSettings,
    pub theme: Theme,
    pub language: Language,
}
/// Get the settings file path
#[cfg(target_os = "android")]
fn get_settings_path() -> Result<PathBuf, String> {
    let mut path =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    path.push(SETTINGS_FILE_NAME);
    Ok(path)
}
#[cfg(not(target_os = "android"))]
fn get_settings_path() -> Result<PathBuf, String> {
    let mut path =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    path.push(SETTINGS_FILE_NAME);
    Ok(path)
}
/// Load settings from persistent storage (thread-safe)
///
/// Returns default settings if file doesn't exist or can't be parsed.
///
/// # Examples
/// ```no_run
/// let settings = load_settings();
/// if settings.notifications.stadning_nu {
///     println!("Städas nu notifications enabled");
/// }
/// ```
pub fn load_settings() -> AppSettings {
    let _lock = SETTINGS_LOCK.lock().unwrap();
    #[cfg(any(target_os = "android", not(target_os = "android")))]
    {
        match get_settings_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(contents) => match serde_json::from_str::<AppSettings>(&contents) {
                            Ok(settings) => {
                                eprintln!("[Settings] Loaded from {:?}", path);
                                return settings;
                            }
                            Err(e) => {
                                eprintln!("[Settings] Failed to parse settings: {}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("[Settings] Failed to read file: {}", e);
                        }
                    }
                } else {
                    eprintln!("[Settings] No settings file found, using defaults");
                }
            }
            Err(e) => {
                eprintln!("[Settings] Failed to get settings path: {}", e);
            }
        }
    }
    AppSettings::default()
}
/// Save settings to persistent storage (thread-safe)
///
/// # Arguments
/// * `settings` - The settings to save
///
/// # Returns
/// - `Ok(())` if successful
/// - `Err(message)` if save failed
///
/// # Examples
/// ```no_run
/// use amp_android::components::settings::{AppSettings, NotificationSettings};
///
/// let mut settings = AppSettings::default();
/// settings.notifications.en_dag = true;
///
/// if let Err(e) = save_settings(&settings) {
///     eprintln!("Failed to save settings: {}", e);
/// }
/// ```
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let _lock = SETTINGS_LOCK.lock().unwrap();
    #[cfg(any(target_os = "android", not(target_os = "android")))]
    {
        let path = get_settings_path()?;
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write settings file: {}", e))?;
        eprintln!("[Settings] Saved to {:?}", path);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(settings.language, Language::Svenska);
        assert!(settings.notifications.stadning_nu);
        assert!(settings.notifications.sex_timmar);
        assert!(!settings.notifications.en_dag);
    }
    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }
}
