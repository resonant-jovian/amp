# Android App

Native Android application for checking parking restrictions in Malmö.

## Overview

**Built with:** Dioxus (Rust-to-native UI framework)

**Features:**
- Address search
- Current location detection
- Parking zone restrictions display
- Offline operation (embedded data)

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
│   ├── main.rs           # Entry point
│   ├── components/       # UI components
│   └── ui/               # App layout
├── assets/              # Icons, images
├── Cargo.toml
└── Dioxus.toml          # Dioxus config
```

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
- `amp_core` — Correlation engine

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
2. **Grant location permissions** (optional)
3. **Enter address** or use current location
4. **View restrictions:** Zone info, time windows, days

## Features

### Address Search

Type address to check parking restrictions.

### Current Location

Use GPS to find restrictions at current position.

### Offline Mode

All data embedded in APK — no internet required after install.

### Restriction Display

Shows:
- Zone name
- Time restrictions (e.g., "06:00-18:00")
- Day restrictions (weekdays/weekends)
- Distance to zone boundary

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

# Integration tests (requires device/emulator)
dx test --platform android
```

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
- None (offline operation)

**Optional:**
- `ACCESS_FINE_LOCATION` — For current location feature

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

**"Data outdated"**
```bash
# Update embedded data and rebuild
amp-server correlate
cp results.parquet android/assets/
dx build --release
```

## Related Documentation

- [Architecture](../docs/architecture.md) — System design
- [core/](../core/) — Core library
- [Dioxus Docs](https://dioxuslabs.com/) — Framework documentation
