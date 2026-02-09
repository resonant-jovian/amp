# Notification Settings UI

Implementation of notification toggle switches in the settings dropdown panel for controlling street cleaning notification preferences.

## Overview

The Aviseringar (Notifications) section in the settings dropdown now includes three toggle switches that allow users to control when they receive notifications about street cleaning events. Each toggle directly modifies the persistent settings stored in the `settings.parquet` file.

## Toggle Switches

### 1. Städas nu (Currently Active)
- **Label**: "Städas nu"
- **Description**: "Avisera när gatustädning pågår"
- **Default**: `true` (enabled)
- **Function**: Controls whether notifications fire when street cleaning is currently active
- **Backend check**: `notifications.rs::notify_active()` checks `settings.notifications.stadning_nu`

### 2. 6 timmar (6 Hours Before)
- **Label**: "6 timmar"
- **Description**: "Avisera 6 timmar före gatustädning"
- **Default**: `true` (enabled)
- **Function**: Controls whether notifications fire 6 hours before street cleaning starts
- **Backend check**: `notifications.rs::notify_six_hours()` checks `settings.notifications.sex_timmar`

### 3. 1 dag (1 Day Before)
- **Label**: "1 dag"
- **Description**: "Avisera 1 dag före gatustädning"
- **Default**: `false` (disabled)
- **Function**: Controls whether notifications fire 1 day (24 hours) before street cleaning starts
- **Backend check**: `notifications.rs::notify_one_day()` checks `settings.notifications.en_dag`

## Architecture

### State Management

The notification settings are managed through a Dioxus signal in `SettingsDropdown`:

```rust
let mut settings = use_signal(load_settings);
```

This signal is initialized with settings loaded from `settings.parquet` and is updated in real-time when users toggle switches.

### Toggle Callback Pattern

Each toggle follows the same pattern:

```rust
let on_toggle_stadning_nu = move |_| {
    let mut current = settings();              // 1. Read current settings
    current.notifications.stadning_nu = !current.notifications.stadning_nu;  // 2. Flip boolean
    save_settings(&current);                   // 3. Persist to disk
    settings.set(current);                     // 4. Update UI signal
};
```

### Persistence Layer

Settings are persisted using the Parquet format through two functions in `android/src/components/settings.rs`:

#### `load_settings() -> AppSettings`
- Reads `settings.parquet` from app storage directory
- Returns default settings if file doesn't exist
- Thread-safe using `SETTINGS_LOCK` mutex

#### `save_settings(settings: &AppSettings)`
- Converts `AppSettings` to `SettingsData`
- Calls `build_settings_parquet()` to create byte buffer
- Writes bytes to `settings.parquet`
- Thread-safe using `SETTINGS_LOCK` mutex
- Creates parent directories if they don't exist

### Data Structures

#### AppSettings (Rust)
```rust
pub struct AppSettings {
    pub notifications: NotificationSettings,
    pub theme: Theme,
    pub language: Language,
}

pub struct NotificationSettings {
    pub stadning_nu: bool,   // Notify during cleaning
    pub sex_timmar: bool,    // Notify 6 hours before
    pub en_dag: bool,        // Notify 1 day before
}
```

#### SettingsData (Parquet Schema)
```rust
pub struct SettingsData {
    pub stadning_nu: bool,
    pub sex_timmar: bool,
    pub en_dag: bool,
    pub theme: String,      // "Light" or "Dark"
    pub language: String,   // "Svenska", "English", etc.
}
```

## UI Components

### RSX Structure

Each toggle uses the same CSS classes as the Debug toggle for visual consistency:

```rust
div { class: "settings-toggle-item",
    div { class: "settings-item-text",
        div { class: "settings-item-label", "Städas nu" }
        div { class: "settings-item-description",
            "Avisera när gatustädning pågår"
        }
    }
    label { class: "settings-toggle-switch",
        input {
            r#type: "checkbox",
            checked: settings().notifications.stadning_nu,
            onchange: on_toggle_stadning_nu,
        }
        div { class: "settings-switch-container",
            div {
                class: "settings-switch-thumb",
                "data-active": if settings().notifications.stadning_nu { "true" } else { "false" },
                div { class: "settings-led" }
            }
        }
    }
}
```

### CSS Classes

- `settings-toggle-item`: Container for each toggle row
- `settings-item-text`: Text content wrapper (label + description)
- `settings-item-label`: Toggle title text
- `settings-item-description`: Toggle description text
- `settings-toggle-switch`: Toggle switch container
- `settings-switch-container`: Visual switch wrapper
- `settings-switch-thumb`: Sliding thumb element
- `settings-led`: Green indicator LED
- `data-active`: Attribute that controls active/inactive styling

## Integration with Notification System

The notification system in `android/src/components/notifications.rs` checks these settings before firing notifications:

```rust
pub fn notify_active(local_data: &[LocalData], settings: &AppSettings) {
    if !settings.notifications.stadning_nu {
        return;  // User disabled "städas nu" notifications
    }
    // ... fire notification if active
}

pub fn notify_six_hours(local_data: &[LocalData], settings: &AppSettings) {
    if !settings.notifications.sex_timmar {
        return;  // User disabled "6 timmar" notifications
    }
    // ... fire notification if within 6-hour window
}

pub fn notify_one_day(local_data: &[LocalData], settings: &AppSettings) {
    if !settings.notifications.en_dag {
        return;  // User disabled "1 dag" notifications
    }
    // ... fire notification if within 24-hour window
}
```

## File Changes

### Modified Files

#### `android/src/components/settings.rs`
- Added `save_settings()` function for persisting settings
- Added `to_settings_data()` helper for converting `AppSettings` to `SettingsData`
- Added `build_settings_parquet` import from `amp_core::parquet`
- Added roundtrip test for settings serialization

#### `android/src/ui/settings_dropdown.rs`
- Renamed `_settings` signal to `settings` (now actively used)
- Added `save_settings` import
- Added three toggle callback handlers:
  - `on_toggle_stadning_nu`
  - `on_toggle_sex_timmar`
  - `on_toggle_en_dag`
- Populated Aviseringar section body with three toggle switch components

### Unchanged Files

- `android/src/components/notifications.rs` - Check logic already correct
- `android/src/android_bridge.rs` - No changes needed
- `core/src/parquet.rs` - Already has `build_settings_parquet()` function
- `core/src/structs.rs` - `SettingsData` struct already defined

## Storage Location

### Android
Settings file path: `/data/local/tmp/amp_storage/settings.parquet`
Alternatively uses `APP_FILES_DIR` environment variable if set.

### Desktop/Testing
Settings file path: `./settings.parquet` (current directory)

## Thread Safety

Both `load_settings()` and `save_settings()` use the `SETTINGS_LOCK` mutex to prevent race conditions when multiple components access settings simultaneously:

```rust
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

pub fn save_settings(settings: &AppSettings) {
    let _lock = SETTINGS_LOCK.lock().unwrap();
    // ... write operations
}
```

## Testing

### Unit Tests

Added to `android/src/components/settings.rs`:

```rust
#[test]
fn test_settings_roundtrip() {
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
```

### Manual Testing

1. Launch app
2. Open settings dropdown (tap gear icon)
3. Expand "Aviseringar" section
4. Toggle each switch on/off
5. Verify LED indicator changes color
6. Close and reopen settings
7. Verify toggle states persist

## Future Enhancements

- Add theme toggle to "Inställningar" section
- Add language selector to "Inställningar" section
- Add settings export/import functionality
- Add settings reset button
- Add notification time customization (e.g., "3 hours" instead of fixed "6 hours")

## Related Documentation

- [Android Notifications](./android-notifications.md) - Notification system implementation
- [Android Persistent State](./android-persistent-state.md) - Parquet-based state management
- [Data Format](./data-format.md) - Parquet schema definitions
