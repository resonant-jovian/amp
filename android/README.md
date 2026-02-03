# Android App

Native Android application for checking parking restrictions in Malmö.

## Overview

**Built with:** Dioxus (Rust-to-native UI framework)

**Features:**
- Address search and management
- Current location detection
- Parking zone restrictions display
- Offline operation (embedded data)
- **Persistent state with automatic backup**
- **Background notifications**
- **Validity checking for date-dependent restrictions**

## Building

### Prerequisites

```bash
# Install Dioxus CLI
cargo install dioxus-cli

# Install Android NDK
# See: https://developer.android.com/ndk/guides
```

### Build Commands

```bash
# Debug build
cd android
dx build --platform android

# Release build
dx build --platform android --release

# Build and install to device
dx build --platform android --release
../adb-install.sh
```

### APK Location

```
android/dist/
└── amp-android.apk
```

## Project Structure

```
android/
├── src/
│   ├── main.rs              # Entry point
│   ├── android_bridge.rs    # JNI bridge for Android APIs
│   ├── components/          # Business logic
│   │   ├── storage.rs       # Persistent state (Parquet)
│   │   ├── lifecycle.rs     # Background task management
│   │   ├── validity.rs      # Date validation
│   │   ├── matching.rs      # Address matching
│   │   ├── notification.rs  # Push notifications
│   │   └── ...
│   └── ui/                  # UI components (#[component])
│       ├── mod.rs           # App entry
│       ├── addresses.rs     # Address list
│       └── panels.rs        # Info panels
├── assets/                  # Icons, images, styles
├── Cargo.toml
└── Dioxus.toml             # Dioxus config
```

## Persistent State System

The app implements a robust persistent state system using Parquet file format.

### Storage Architecture

**Files:**
- `local.parquet` — Active user data
- `local.parquet.backup` — Previous version (auto-rotation)

**When data is written:**
- Address added/removed
- Active state toggled
- Valid state changed (day 29/30 in February)
- App exit/crash
- Once per day (midnight)

**When data is read:**
- App start
- Once per day (midnight)

**Backup Strategy:**
On write:
1. Delete old backup
2. Rename current → backup
3. Write new data

This ensures data safety even if app crashes during write.

### Validity Checking

Parking restrictions with day-of-month requirements (e.g., "15th of month") need special handling:

- **Day 29**: Invalid in non-leap-year February
- **Day 30**: Invalid in all February
- **Day 31**: Invalid in months with 30 days

The system automatically:
- Checks validity on app start
- Re-checks daily at midnight
- Updates `valid` flag on affected addresses
- Persists changes to storage

### Background Service

For continuous operation and notifications:

**Required:**
- Foreground service (Android)
- Boot receiver (RECEIVE_BOOT_COMPLETED)
- Battery optimization exemption

**See:** [`docs/android-persistent-state.md`](../docs/android-persistent-state.md) for complete implementation guide.

## Implementation

**Entry Point:** `src/main.rs`

```rust
use dioxus::prelude::*;

pub mod components;
pub mod ui;

use ui::App;

fn main() {
    launch(App);
}
```

**Dependencies:**
- `dioxus` — UI framework
- `amp_core` — Correlation engine, Parquet I/O
- `chrono` — Date/time handling
- `parquet` — Binary data format

See `Cargo.toml` for complete list.

## Configuration

**Dioxus.toml:**
```toml
[application]
name = "amp-android"
default_platform = "android"

[android]
package = "se.sjoegren.amp"
label = "AMP Parking"
icon = "assets/icon.png"
```

## Usage

1. **Install APK** on Android device
2. **Grant permissions** (location, notifications, battery)
3. **Add addresses** to track
4. **Toggle active** for addresses you want notifications for
5. **Background service** handles:
   - Daily validity checks
   - Notification scheduling
   - Data persistence

## Features

### Address Management

- Add/remove addresses
- Toggle active state (enable/disable notifications)
- Automatic validity checking
- Persistent across app restarts

### Validity Tracking

- Detects when restrictions become invalid (e.g., February 30th)
- Auto-updates daily
- Visual indicators in UI

### Offline Mode

All data embedded in APK — no internet required after install.

### Restriction Display

Shows:
- Zone name
- Time restrictions (e.g., "06:00-18:00")
- Day restrictions (e.g., "15th of month")
- Valid status (green/red indicator)

## Development

### Run in Simulator

```bash
# Requires Android emulator
dx serve --platform android
```

### Debug on Device

```bash
# Connect device via USB
adb devices

# Build and install
dx build --platform android
adb install android/dist/amp-android.apk

# View logs
adb logcat | grep amp
```

### Hot Reload

```bash
# Development mode with live updates
dx serve --platform android --hot-reload
```

## Data Embedding

Parking zone data embedded at build time:

1. Fetch latest data from Malmö API
2. Run correlation
3. Save results as Parquet
4. Embed in APK assets

**Update data:**
```bash
# Re-run correlation and rebuild
cd ../server
amp-server correlate --algorithm rtree
cp results.parquet ../android/assets/
cd ../android
dx build --platform android --release
```

## Testing

```bash
# Unit tests
cargo test -p amp-android

# Component tests
cargo test -p amp-android storage
cargo test -p amp-android lifecycle
cargo test -p amp-android validity

# Integration tests (requires device/emulator)
dx test --platform android
```

### Manual Testing Checklist

**Storage:**
- [ ] Add address → restart app → verify address persisted
- [ ] Remove address → restart app → verify removed
- [ ] Toggle active → restart app → verify state saved
- [ ] Delete `local.parquet` → restart → verify recovery from backup

**Validity:**
- [ ] Add address with day 30 in February → verify marked invalid
- [ ] Same address in March → verify marked valid
- [ ] Leap year February 29 → verify marked valid

**Background:**
- [ ] Force kill app → restart → verify no data loss
- [ ] Reboot device → verify service restarts

## Deployment

### Google Play Store

1. Sign APK with release key
2. Create Play Console listing
3. Upload APK
4. Submit for review

See: [Android Publishing Guide](https://developer.android.com/studio/publish)

### Direct Distribution

```bash
# Share APK file
cp android/dist/amp-android.apk ~/Downloads/

# Users: Enable "Install from Unknown Sources"
```

## Permissions

**Required:**
- `FOREGROUND_SERVICE` — Background notifications
- `RECEIVE_BOOT_COMPLETED` — Auto-start on boot
- `POST_NOTIFICATIONS` — Send notifications

**Optional:**
- `ACCESS_FINE_LOCATION` — Current location feature
- Battery optimization exemption — Reliable background operation

## Troubleshooting

**"Build failed"**
```bash
# Check NDK installation
echo $ANDROID_NDK_ROOT

# Reinstall Dioxus CLI
cargo install dioxus-cli --force
```

**"App crashes on launch"**
```bash
# View crash logs
adb logcat | grep FATAL

# Check device compatibility (min Android 5.0)
```

**"Data not persisting"**
```bash
# Check logs for storage errors
adb logcat | grep Storage

# Verify file permissions
adb shell ls -l /data/data/se.sjoegren.amp/files/
```

**"Notifications not working"**
```bash
# Check if foreground service running
adb shell dumpsys activity services | grep BackgroundService

# Verify permissions granted
adb shell dumpsys package se.sjoegren.amp | grep permission
```

**"Background service killed"**
- Disable battery optimization for the app
- Check manufacturer-specific battery settings (Xiaomi, Huawei, etc.)
- Use `START_STICKY` in service to auto-restart

## Related Documentation

- [**Persistent State Guide**](../docs/android-persistent-state.md) — Complete implementation details
- [Architecture](../docs/architecture.md) — System design
- [core/](../core/) — Core library
- [Dioxus Docs](https://dioxuslabs.com/) — Framework documentation

## Contributing

### Code Organization

- **All logic in `components/`** — Business logic, state management
- **All UI in `ui/`** — Components marked with `#[component]`
- **Documentation in `docs/`** — `.md` files only

### Documentation Standards

- Use `///` for inline documentation
- Add examples in doc comments
- Update README when adding features
- Document in `docs/` for complex features

See [Rust Best Practices](https://canonical.github.io/rust-best-practices/) for coding standards.
