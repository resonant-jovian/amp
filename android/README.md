# Amp Android Crate

Android mobile application for parking restriction lookup in Malmö, Sweden.

## Overview

The Android crate provides a native mobile app built with [Dioxus](https://dioxuslabs.com/) and Rust. It offers a user-friendly interface for managing parking restrictions at saved addresses with real-time countdowns and validity checking.

## Features

✅ **Address Management**
- Add/remove addresses with fuzzy matching
- Toggle address visibility (active/inactive)
- Duplicate detection (case-insensitive)
- Persistent storage using Parquet files

✅ **Intelligent Matching**
- Fuzzy search with Levenshtein distance (handles typos)
- Case-insensitive matching
- Substring matching for partial addresses
- Pre-computed correlations for O(1) lookups

✅ **Time Management**
- Real-time countdown to parking expiry
- Time-categorized panels:
  - 🔴 **Active Now**: Currently restricted
  - 🟠 **6 Hours**: Active within 6 hours
  - 🟡 **1 Day**: Active within 24 hours
  - 🟢 **1 Month**: Active within 30 days
  - 🔵 **>1 Month**: Active beyond 30 days
  - ⚪ **Invalid**: Validation failed

✅ **Validity Checking**
- Handles date-dependent restrictions (day 1-31)
- Accounts for month lengths (Feb 28/29, etc.)
- Automatic daily validation updates
- Swedish timezone support

✅ **Android Integration**
- Native performance with Rust
- JNI bridge for system services
- Lifecycle-aware background tasks
- Internal storage access
- Notification support (planned)

✅ **Developer Tools**
- Debug mode with example addresses
- Read-only test data
- Extensive logging

## Architecture

```
┌─────────────────────────────────────┐
│   Dioxus UI (ui/)                │  ← User interface
├─────────────────────────────────────┤
│   Business Logic (components/)  │  ← App logic
├─────────────────────────────────────┤
│   Android Bridge (JNI)          │  ← Native integration
├─────────────────────────────────────┤
│   Core Library (amp_core)       │  ← Correlation engine
└─────────────────────────────────────┘
```

### Module Structure

#### UI Layer (`ui/`)
- **mod.rs**: Main app component, state management, fuzzy matching
- **top_bar.rs**: Search bar and debug controls
- **addresses.rs**: Saved address list
- **panels.rs**: Time-categorized restriction displays
- **confirm_dialog.rs**: Confirmation dialogs
- **info_dialog.rs**: Parking detail dialogs
- **settings_dropdown.rs**: Settings menu

#### Business Logic (`components/`)
- **storage.rs**: Persistent Parquet-based storage (33KB)
- **static_data.rs**: Embedded parking database (13KB)
- **matching.rs**: Address validation and lookup (12KB)
- **lifecycle.rs**: Android lifecycle management (7KB)
- **settings.rs**: User preferences (7KB)
- **validity.rs**: Date-dependent validation (6KB)
- **countdown.rs**: Real-time timers (10KB)
- **debug.rs**: Debug utilities (8KB)
- **address_utils.rs**: String normalization
- **geo.rs**: GPS location (stub)
- **notification.rs**: Push notifications (stub)

#### Platform Integration
- **android_bridge.rs**: JNI bindings (4KB)
- **android_utils.rs**: File system access (2.5KB)

## Quick Start

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Android NDK
# Via Android Studio SDK Manager or:
sdk install android-ndk 26.0.10792818

# Install Dioxus CLI
cargo install dioxus-cli

# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### Building

```bash
# Build for Android
cd android
dx build --platform android --release

# Or use cargo-apk
cargo apk build --release
```

### Running

```bash
# Run on connected device/emulator
dx serve --platform android

# Or with cargo-apk
cargo apk run
```

## Usage Examples

### Adding an Address

```rust
use amp_android::ui::StoredAddress;

let address = StoredAddress::new(
    "Storgatan".to_string(),
    "10".to_string(),
    "22100".to_string(),
);

if address.valid {
    println!("Valid address with parking data!");
    if let Some(ref entry) = address.matched_entry {
        println!("Restriction: {:?}", entry.info);
    }
}
```

### Loading Storage

```rust
use amp_android::storage;

// Load saved addresses
let addresses = storage::read_addresses_from_device();
println!("Loaded {} addresses", addresses.len());

// Modify and save
let mut addresses = addresses;
addresses.push(new_address);
storage::write_addresses_to_device(&addresses)?;
```

### Matching Addresses

```rust
use amp_android::components::matching::{match_address, MatchResult};

match match_address("Storgatan", "10", "22100") {
    MatchResult::Valid(entry) => {
        println!("Found: {}", entry.adress);
        if entry.is_active(Utc::now()) {
            println!("Restriction is active now!");
        }
    },
    MatchResult::Invalid => {
        eprintln!("Address not found");
    },
}
```

### Checking Validity

```rust
use amp_android::components::validity::check_and_update_validity;

let mut addresses = get_addresses();
if check_and_update_validity(&mut addresses) {
    println!("Some addresses changed validity");
    save_addresses(&addresses);
}
```

## Data Format

### StoredAddress

```rust
pub struct StoredAddress {
    pub id: usize,              // UUID-based unique ID
    pub street: String,         // "Storgatan"
    pub street_number: String,  // "10" or "10A"
    pub postal_code: String,    // "22100" or "221 00"
    pub valid: bool,            // Matches database?
    pub active: bool,           // Show in panels?
    pub matched_entry: Option<DB>, // Parking data
}
```

### Storage Files

```
/data/data/com.example.amp/files/
├── local.parquet         # Main address storage
├── local.parquet.backup  # Previous version
└── settings.parquet      # User preferences
```

### Embedded Data

```
android/assets/data/
└── db.parquet            # Pre-computed correlations
```

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Cold start | 0.5-1.0s | Parquet load + index build |
| Address add | 10-50ms | Fuzzy match + persist + render |
| Toggle active | 5-10ms | Update + persist + render |
| Address search | 10-50ms | Fuzzy matching with Levenshtein |
| Correlation | 0.01-0.05ms | O(1) HashMap lookup |
| Validity check | 1-5ms | Daily, checks all addresses |
| Panel update | <5ms | Reactive, automatic |

**Memory Usage:** ~15-30 MB total (including UI)

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib storage
cargo test --lib matching
cargo test --lib validity

# Run with logging
RUST_LOG=debug cargo test
```

### Debug Mode

1. Launch the app
2. Tap the debug button in the top bar
3. Example addresses load (read-only)
4. Test UI without modifying user data
5. Tap debug button again to exit

## Configuration

### Settings (Planned)

```rust
pub struct AppSettings {
    pub notifications: NotificationSettings,
    pub theme: Theme,           // Light/Dark
    pub language: Language,     // Svenska/English/etc.
}
```

### Notification Settings

```rust
pub struct NotificationSettings {
    pub stadning_nu: bool,   // Notify when active
    pub sex_timmar: bool,    // 6 hours before
    pub en_dag: bool,        // 1 day before
}
```

## Troubleshooting

### Storage Issues

If addresses aren't persisting:

```bash
# Check app permissions
adb shell run-as com.example.amp ls -la /data/data/com.example.amp/files/

# Clear storage and restart
adb shell run-as com.example.amp rm -rf /data/data/com.example.amp/files/*.parquet
```

### Build Issues

```bash
# Clean build
cargo clean
rm -rf target/

# Verify NDK
echo $ANDROID_NDK_HOME
echo $ANDROID_HOME

# Update toolchain
rustup update
cargo install dioxus-cli --force
```

### Logging

```bash
# View Android logs
adb logcat | grep amp

# Or use log level filter
adb logcat *:E amp:D
```

## Contributing

See the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Code Style

- Follow Rust naming conventions
- Add rustdoc comments to public items
- Include examples in documentation
- Write tests for new features
- Run `cargo fmt` and `cargo clippy`

### Commit Messages

Follow conventional commits:

```
feat(android): Add GPS location support
fix(storage): Handle corrupted parquet files
docs(android): Update README with examples
test(matching): Add fuzzy match test cases
```

## License

GPL-3.0 - See [LICENSE](../LICENSE) for details.

## See Also

- [Core Library](../core/README.md) - Parking correlation engine
- [Server](../server/README.md) - Data processing and API
- [iOS App](../ios/README.md) - iOS mobile app
- [Dioxus Documentation](https://dioxuslabs.com/docs/0.7/guide/en/)
