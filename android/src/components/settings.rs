//! Settings state management for Android app
//!
//! Provides persistent storage for user preferences using Parquet format:
//! - Notification settings (städas nu, 6 hours, 1 day before)
//! - Theme preference (dark/light mode)
//! - Language selection
//!
//! Settings are stored in a Parquet file (settings.parquet) for efficient binary storage.
use amp_core::parquet::{build_settings_parquet, read_settings_parquet};
use amp_core::structs::SettingsData;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
/// Thread-safe settings mutex
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());
#[cfg(target_os = "android")]
const SETTINGS_FILE_NAME: &str = "settings.parquet";
#[cfg(not(target_os = "android"))]
const SETTINGS_FILE_NAME: &str = "settings.parquet";
/// Notification timing preferences
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}
impl Theme {
    fn to_string(&self) -> String {
        match self {
            Theme::Light => "Light".to_string(),
            Theme::Dark => "Dark".to_string(),
        }
    }
    fn from_string(s: &str) -> Self {
        match s {
            "Dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }
}
/// Supported languages
#[derive(Clone, Debug, PartialEq, Default)]
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
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
    fn from_string(s: &str) -> Self {
        match s {
            "English" => Language::English,
            "Espanol" => Language::Espanol,
            "Francais" => Language::Francais,
            _ => Language::Svenska,
        }
    }
}
/// Complete app settings
#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppSettings {
    pub notifications: NotificationSettings,
    pub theme: Theme,
    pub language: Language,
}
/// Convert AppSettings to SettingsData for Parquet storage
fn to_settings_data(settings: &AppSettings) -> SettingsData {
    SettingsData {
        stadning_nu: settings.notifications.stadning_nu,
        sex_timmar: settings.notifications.sex_timmar,
        en_dag: settings.notifications.en_dag,
        theme: settings.theme.to_string(),
        language: settings.language.to_string(),
    }
}
/// Convert SettingsData from Parquet to AppSettings
fn from_settings_data(data: SettingsData) -> AppSettings {
    AppSettings {
        notifications: NotificationSettings {
            stadning_nu: data.stadning_nu,
            sex_timmar: data.sex_timmar,
            en_dag: data.en_dag,
        },
        theme: Theme::from_string(&data.theme),
        language: Language::from_string(&data.language),
    }
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
                    match File::open(&path) {
                        Ok(file) => match read_settings_parquet(file) {
                            Ok(settings_vec) => {
                                if let Some(settings_data) = settings_vec.first() {
                                    eprintln!("[Settings] Loaded from {:?}", path);
                                    return from_settings_data(settings_data.clone());
                                } else {
                                    eprintln!("[Settings] Parquet file empty, using defaults");
                                }
                            }
                            Err(e) => {
                                eprintln!("[Settings] Failed to parse parquet: {}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("[Settings] Failed to open file: {}", e);
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
        let settings_data = to_settings_data(settings);
        let buffer = build_settings_parquet(vec![settings_data])
            .map_err(|e| format!("Failed to build parquet: {}", e))?;
        fs::write(&path, buffer).map_err(|e| format!("Failed to write settings file: {}", e))?;
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
        assert_eq!(settings.theme, Theme::Light);
        assert_eq!(settings.language, Language::Svenska);
        assert!(settings.notifications.stadning_nu);
        assert!(settings.notifications.sex_timmar);
        assert!(!settings.notifications.en_dag);
    }
    #[test]
    fn test_settings_conversion_roundtrip() {
        let original = AppSettings {
            notifications: NotificationSettings {
                stadning_nu: false,
                sex_timmar: true,
                en_dag: true,
            },
            theme: Theme::Dark,
            language: Language::English,
        };
        let settings_data = to_settings_data(&original);
        let restored = from_settings_data(settings_data);
        assert_eq!(original, restored);
    }
    #[test]
    fn test_settings_parquet_roundtrip() {
        let settings = AppSettings {
            notifications: NotificationSettings {
                stadning_nu: true,
                sex_timmar: false,
                en_dag: true,
            },
            theme: Theme::Dark,
            language: Language::Espanol,
        };
        let result = save_settings(&settings);
        assert!(result.is_ok(), "Save should succeed");
        let loaded = load_settings();
        assert_eq!(settings, loaded, "Loaded settings should match saved");
    }
}
