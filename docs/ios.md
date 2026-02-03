# iOS Implementation

Guide for building the AMP iOS app.

## Overview

iOS application for checking parking restrictions in Malmö, sharing UI code with the Android app via Dioxus.

**Status:** In development

**Platform:** iOS 13+

**Framework:** Dioxus (Rust-to-native)

## Features

Planned features (shared with Android):
- Address search
- Current location detection
- Parking zone restrictions
- Offline operation
- Real-time countdown

## Building

### Prerequisites

**Required:**
- macOS with Xcode 14+
- Rust 1.70+
- Dioxus CLI: `cargo install dioxus-cli`
- iOS SDK

### Setup

```bash
# Install Xcode
xapp-select --install

# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios # For simulator

# Install Dioxus CLI
cargo install dioxus-cli
```

### Build Commands

**Debug build:**
```bash
cd ios
dx build --platform ios
```

**Release build:**
```bash
dx build --platform ios --release
```

**Run in simulator:**
```bash
dx serve --platform ios
```

## Project Structure

```
ios/
├── src/
│   ├── main.rs           # Entry point (shares with Android)
│   ├── ios_bridge.rs     # iOS-specific platform APIs
│   ├── components/       # Shared with Android
│   └── ui/               # Shared UI code
├── assets/              # Icons, data files
├── Cargo.toml
└── Dioxus.toml
```

## Code Sharing with Android

### Shared Modules

Most code is shared between iOS and Android:

**Shared:**
- UI components (`ui/`)
- Business logic (`components/`)
- Data structures
- Correlation logic

**Platform-specific:**
- `android_bridge.rs` - Android APIs
- `ios_bridge.rs` - iOS APIs
- Build configuration
- Asset bundling

### Conditional Compilation

```rust
#[cfg(target_os = "android")]
mod android_bridge;

#[cfg(target_os = "ios")]
mod ios_bridge;

#[cfg(target_os = "android")]
use android_bridge::get_location;

#[cfg(target_os = "ios")]
use ios_bridge::get_location;
```

## Data Embedding

Same as Android - embed Parquet files at build time:

```bash
# Generate correlation data
cd server
cargo run --release -- output --algorithm kdtree --android

# Copy to iOS assets
cp android/assets/data/*.parquet ios/assets/data/

# Build iOS app
cd ../ios
dx build --platform ios --release
```

## Configuration

**Dioxus.toml:**
```toml
[application]
name = "amp-ios"
default_platform = "ios"

[ios]
bundle_identifier = "se.sjoegren.amp"
version = "1.0.0"
build_number = "1"
deployment_target = "13.0"
```

## Testing

### Simulator

```bash
# List available simulators
xcrun simctl list devices

# Run in simulator
cd ios
dx serve --platform ios
```

### Physical Device

```bash
# Build for device
dx build --platform ios --release

# Open in Xcode to deploy
open ios/build/*.xcodeproj
```

### Debug Logging

```bash
# View logs from simulator/device
xcrun simctl spawn booted log stream --predicate 'process == "amp"'
```

## Permissions

**Info.plist additions:**

```xml
<key>NSLocationWhenInUseUsageDescription</key>
<string>AMP needs your location to show nearby parking restrictions</string>

<key>NSLocationAlwaysAndWhenInUseUsageDescription</key>
<string>AMP needs your location to show nearby parking restrictions</string>
```

## Deployment

### TestFlight

1. Create App Store Connect listing
2. Archive build in Xcode
3. Upload to App Store Connect
4. Add to TestFlight
5. Invite testers

### App Store

1. Complete app review information
2. Submit for review
3. Wait for approval
4. Release to App Store

## Known Differences from Android

### Platform APIs

**Location:**
- iOS: CoreLocation framework
- Android: Android Location API

**Notifications:**
- iOS: UserNotifications framework
- Android: Android NotificationManager

**Storage:**
- iOS: Documents directory
- Android: Internal storage

### UI Considerations

**Navigation:**
- iOS: Bottom tab bar convention
- Android: Material Design navigation

**Styling:**
- iOS: Cupertino design language
- Android: Material Design

## Troubleshooting

### Build Fails

**"No iOS SDK found"**
```bash
xcode-select --install
xcodebuild -showsdks
```

**"Rust target not installed"**
```bash
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
```

### Runtime Issues

**Simulator crashes:**
- Check iOS version compatibility
- Verify deployment target in Dioxus.toml
- View crash logs in Console.app

**Location not working:**
- Verify Info.plist permissions
- Check location services enabled in Settings
- Test on physical device (simulator has limitations)

## Development Status

**Completed:**
- Project structure
- Basic Dioxus setup
- Code sharing with Android

**In Progress:**
- iOS-specific bridge implementation
- Location services integration
- Notification support

**Planned:**
- App Store submission
- TestFlight beta testing
- iOS-specific UI refinements

## See Also

- **[Android Implementation](android.md)** - Shared architecture and components
- **[Architecture](architecture.md)** - System design
- **[Building](building.md)** - Build instructions
- **[iOS README](../ios/README.md)** - Component-specific details
- **[Dioxus Docs](https://dioxuslabs.com/)** - Framework documentation
