# iOS App

Native iOS application for checking parking restrictions in Malmö.

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

# Install Xcode (macOS only)
# Download from App Store

# Install iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios  # For simulator
```

### Build Commands

```bash
# Debug build
cd ios
dx build --platform ios

# Release build
dx build --platform ios --release

# Build for simulator
dx build --platform ios --simulator
```

### App Location

```
ios/dist/
└── AMP.app
```

## Project Structure

```
ios/
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
name = "amp-ios"
default_platform = "ios"

[ios]
bundle_identifier = "se.sjoegren.amp"
app_name = "AMP Parking"
icon = "assets/icon.png"
```

## Usage

1. **Install app** on iOS device
2. **Grant location permissions** (optional)
3. **Enter address** or use current location
4. **View restrictions:** Zone info, time windows, days

## Features

### Address Search

Type address to check parking restrictions.

### Current Location

Use GPS to find restrictions at current position.

### Offline Mode

All data embedded in app bundle — no internet required after install.

### Restriction Display

Shows:
- Zone name
- Time restrictions (e.g., "06:00-18:00")
- Day restrictions (weekdays/weekends)
- Distance to zone boundary

## Development

### Run in Simulator

```bash
# Launch iOS simulator
open -a Simulator

# Run app
dx serve --platform ios
```

### Debug on Device

```bash
# Connect device via USB
# Trust computer on device

# Build and install (requires Apple Developer account)
dx build --platform ios --release
# Use Xcode to deploy
```

### Hot Reload

```bash
# Development mode with live updates
dx serve --platform ios --hot-reload
```

## Data Embedding

Parking zone data embedded at build time:

1. Fetch latest data from Malmö API
2. Run correlation
3. Save results as Parquet
4. Embed in app bundle

**Update data:**
```bash
# Re-run correlation and rebuild
cd ../server
amp-server correlate --algorithm rtree
cp results.parquet ../ios/assets/
cd ../ios
dx build --platform ios --release
```

## Testing

```bash
# Unit tests
cargo test -p amp-ios

# Integration tests (requires simulator/device)
dx test --platform ios
```

## Deployment

### App Store

1. **Join Apple Developer Program** ($99/year)
2. **Create App Store Connect listing**
3. **Build release version**
4. **Upload with Xcode or Transporter**
5. **Submit for review**

See: [App Store Review Guidelines](https://developer.apple.com/app-store/review/guidelines/)

### TestFlight (Beta)

```bash
# Build release
dx build --platform ios --release

# Upload to App Store Connect
# Invite testers via TestFlight
```

### Enterprise Distribution

Requires Apple Developer Enterprise Program.

## Permissions

**Info.plist entries:**

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>Used to check parking restrictions at your current location</string>
```

**Required:**
- None (offline operation)

**Optional:**
- Location ("When In Use") — For current location feature

## Troubleshooting

**"Build failed"**
```bash
# Check Xcode installation
xcode-select --print-path

# Install iOS targets
rustup target add aarch64-apple-ios

# Reinstall Dioxus CLI
cargo install dioxus-cli --force
```

**"Code signing error"**
- Requires Apple Developer account
- Set up signing in Xcode: Open `ios/AMP.xcodeproj`

**"App crashes on launch"**
```bash
# View crash logs
# Xcode: Window → Devices and Simulators → View Device Logs

# Check minimum iOS version (iOS 12.0+)
```

**"Data outdated"**
```bash
# Update embedded data and rebuild
amp-server correlate
cp results.parquet ios/assets/
dx build --release
```

## Minimum Requirements

- **iOS:** 12.0+
- **Xcode:** 13.0+
- **macOS:** 11.0+ (for development)

## Related Documentation

- [Architecture](../docs/architecture.md) — System design
- [core/](../core/) — Core library
- [Dioxus Docs](https://dioxuslabs.com/) — Framework documentation
