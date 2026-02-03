# Android Implementation

Complete guide for building and deploying the AMP Android app.

## Overview

Native Android application for checking parking restrictions in Malmö, built with Dioxus (Rust-to-native UI framework).

**Features:**
- Address search with autocomplete
- Current location detection
- Parking zone restrictions display
- Offline operation (embedded Parquet data)
- Day/time extraction for quick lookups

## Quick Start

```bash
# Prerequisites
cargo install dioxus-cli
export ANDROID_NDK_ROOT=/path/to/ndk

# Build and run
cd android
dx build --platform android --release
```

**APK Location:** `android/dist/amp.apk`

## Architecture

### Project Structure

```
android/
├── src/
│   ├── main.rs              # Entry point, launches Dioxus app
│   ├── android_bridge.rs    # Android platform APIs
│   ├── components/          # Business logic
│   │   ├── address_utils.rs # Address parsing
│   │   ├── countdown.rs     # Time remaining calculations
│   │   ├── file.rs          # Asset loading
│   │   ├── geo.rs           # Geolocation
│   │   ├── matching.rs      # Zone correlation
│   │   ├── notification.rs  # Notifications
│   │   ├── static_data.rs   # Embedded data access
│   │   └── storage.rs       # Local persistence
│   └── ui/                  # UI components
│       ├── mod.rs           # App component
│       ├── addresses.rs     # Address list/search
│       ├── confirm_dialog.rs
│       ├── info_dialog.rs
│       ├── panels.rs        # Main content panels
│       ├── settings_dropdown.rs
│       └── top_bar.rs       # App header
├── assets/
│   ├── data/
│   │   ├── db.parquet       # Correlation results
│   │   ├── adress_info.parquet
│   │   ├── local.parquet    # Extracted day/time data
│   │   └── parking_db.parquet
│   ├── icon/                # App icons (various DPIs)
│   └── style.css            # UI styling
├── Cargo.toml
└── Dioxus.toml              # Dioxus configuration
```

### Data Flow

```
[User Input] → [Address Search]
                     ↓
            [Match in Parquet Data]
                     ↓
              [Extract Zone Info]
                     ↓
         [Display Restrictions + Timer]
```

### Key Components

**`components/matching.rs`** - Core correlation logic:
- Reads embedded Parquet files
- Matches addresses to zones
- Returns restriction information

**`components/countdown.rs`** - Time calculations:
- Parses time restrictions (e.g., "06:00-18:00")
- Calculates time remaining
- Handles day-of-week logic

**`components/geo.rs`** - Location services:
- Gets current GPS coordinates
- Reverse geocodes to address
- Handles permission requests

**`ui/addresses.rs`** - Search interface:
- Autocomplete dropdown
- Recent addresses
- Address validation

## Building

### Prerequisites

**Required:**
- Rust 1.70+
- Dioxus CLI: `cargo install dioxus-cli`
- Android NDK
- Java 21+

**Setup Android NDK:**
```bash
# Download from https://developer.android.com/ndk
export ANDROID_NDK_ROOT=/path/to/android-ndk-r26b
```

### Build Commands

**Debug build:**
```bash
cd android
dx build --platform android
```

**Release build:**
```bash
dx build --platform android --release
```

**Build and install to device:**
```bash
dx build --platform android --release
adb install dist/amp.apk
```

### Build Configuration

**Dioxus.toml:**
```toml
[application]
name = "amp"
default_platform = "android"

[android]
min_sdk_version = 28
target_sdk_version = 34

[android.manifest]
package = "se.sjoegren.amp"
version_name = "1.0.0"
version_code = 1

[[android.manifest.uses_permission]]
name = "android.permission.ACCESS_FINE_LOCATION"

[[android.manifest.uses_permission]]
name = "android.permission.POST_NOTIFICATIONS"
```

## Data Embedding

### Generating Embedded Data

Parking data is embedded at build time for offline operation:

```bash
# 1. Run correlation with CLI
cd server
cargo run --release -- output --algorithm kdtree --cutoff 100 --android

# 2. Files generated in android/assets/data/:
#    - db.parquet (full correlation results)
#    - local.parquet (extracted day/time for fast lookup)
#    - adress_info.parquet (address metadata)
#    - parking_db.parquet (parking zone info)

# 3. Rebuild Android app
cd ../android
dx build --platform android --release
```

### Data Format

**db.parquet** - Full correlation results:
```
postnummer, adress, gata, gatunummer, info, tid, dag, taxa, antal_platser, typ_av_parkering
```

**local.parquet** - Optimized for app (extracted fields):
```
adress, info, days[], time_ranges[]
```

See **[Data Format](data-format.md)** for complete schema.

### Updating Data

```bash
# Check for data updates
cd server
cargo run --release -- check-updates

# If updates available, regenerate
cargo run --release -- output --android
cd ../android
dx build --release
```

## Testing

### On Emulator

```bash
# Start Android emulator
emulator -avd Pixel_5_API_34

# Build and install
dx build --platform android
adb install dist/amp.apk
```

### On Physical Device

```bash
# Enable USB debugging on device
# Settings → About → Tap "Build number" 7 times
# Settings → Developer options → USB debugging

# Connect device
adb devices

# Install
dx build --platform android --release
adb install -r dist/amp.apk
```

### Debug Logging

```bash
# View app logs
adb logcat | grep amp

# View Rust panic logs
adb logcat | grep RUST_BACKTRACE

# Clear logs
adb logcat -c
```

## Permissions

**Required:**
- None (app works offline)

**Optional:**
- `ACCESS_FINE_LOCATION` - For current location feature
- `POST_NOTIFICATIONS` - For parking expiry notifications

## Troubleshooting

### Build Fails

**"NDK not found"**
```bash
export ANDROID_NDK_ROOT=/path/to/ndk
```

**"Java version mismatch"**
```bash
# Install Java 21
# Set JAVA_HOME
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk
```

**"dx: command not found"**
```bash
cargo install dioxus-cli --force
```

### Runtime Issues

**App crashes on launch:**
```bash
adb logcat | grep FATAL
# Check if data files are embedded
# Rebuild with fresh correlation data
```

**Location not working:**
- Check permission granted in app settings
- Test on physical device (emulator may have location issues)
- Verify GPS is enabled

**Search not finding addresses:**
- Verify `adress_info.parquet` exists in assets
- Check correlation was run with correct data
- View logs: `adb logcat | grep matching`

### Performance Issues

**Slow startup:**
- Parquet files may be too large
- Reduce dataset size in correlation step
- Check device storage space

**Search lag:**
- Optimize autocomplete filtering
- Reduce address list size
- Use background thread for search

## Deployment

### Google Play Store

**1. Generate signing key:**
```bash
keytool -genkey -v -keystore amp-release.keystore \
  -alias amp -keyalg RSA -keysize 2048 -validity 10000
```

**2. Sign APK:**
```bash
dx build --platform android --release
jarsigner -verbose -keystore amp-release.keystore \
  android/dist/amp.apk amp
```

**3. Upload to Play Console:**
- Create app listing
- Upload signed APK
- Fill out store listing
- Submit for review

### Direct Distribution

**Share APK directly:**
```bash
cp android/dist/amp.apk ~/amp-v1.0.0.apk
```

**Installation:**
- Users must enable "Install from Unknown Sources"
- Open APK file to install

## Known Limitations

- **No live data updates** - Data embedded at build time
- **Malmö only** - Hardcoded for Malmö datasets
- **Swedish language** - UI and data in Swedish only
- **Android 9+** - Minimum SDK 28 (Android 9.0)

## Performance

**Typical metrics:**
- APK size: ~8-12 MB (with embedded data)
- Cold start: <2s on mid-range device
- Search response: <100ms
- Memory usage: ~40-60 MB

## See Also

- **[Architecture](architecture.md)** - System design
- **[Building](building.md)** - Build instructions for all components
- **[Data Format](data-format.md)** - Parquet schema
- **[Testing](testing.md)** - Testing strategy
- **[Android README](../android/README.md)** - Component-specific details
