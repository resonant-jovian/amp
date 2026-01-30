# iOS Setup Documentation

## Overview

The iOS implementation shares most business logic with Android through shared Rust modules, with platform-specific code only for notifications, storage, and GPS.

---

## Project Structure

```
amp/
├── core/                        # Shared models
│   └── src/
│       └── models.rs           # ParkingRestriction struct
├── android/                     # Android-specific
│   └── src/
│       ├── main.rs
│       ├── countdown.rs         # → SHARED with iOS
│       ├── matching.rs          # → SHARED with iOS
│       ├── static_data.rs       # → SHARED with iOS
│       ├── components/          # Platform stubs
│       │   ├── notification.rs  # ✗ Android-specific
│       │   ├── storage.rs       # ✗ Android-specific
│       │   └── geo.rs           # ✗ Android-specific
│       └── ui/                  # → SHARED with iOS
│           ├── mod.rs
│           ├── addresses.rs
│           ├── panels.rs
│           └── top_bar.rs
└── ios/                         # iOS-specific
    └── src/
        ├── main.rs
        ├── countdown.rs         # ← Symlink/copy from android
        ├── matching.rs          # ← Symlink/copy from android
        ├── static_data.rs       # ← Symlink/copy from android
        ├── components/          # Platform implementations
        │   ├── mod.rs
        │   ├── notification.rs  # ✓ iOS-specific (UserNotifications)
        │   ├── storage.rs       # ✓ iOS-specific (UserDefaults)
        │   └── geo.rs           # ✓ iOS-specific (CoreLocation)
        └── ui/                  # ← Symlink/copy from android/src/ui
            ├── mod.rs
            ├── addresses.rs
            ├── panels.rs
            └── top_bar.rs
```

---

## Shared Code Strategy

### Option 1: Workspace with Shared Crates (Recommended)

**Best for:** Production deployment

Create shared crates that both platforms depend on:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["core", "shared", "android", "ios"]

# shared/Cargo.toml
[package]
name = "amp-shared"
version = "0.1.0"

[dependencies]
amp-core = { path = "../core" }

# android/Cargo.toml and ios/Cargo.toml
[dependencies]
amp-core = { path = "../core" }
amp-shared = { path = "../shared" }
```

**Move to shared crate:**
- `countdown.rs`
- `matching.rs`
- `static_data.rs`
- `ui/` (entire directory)

### Option 2: Symbolic Links (Development)

**Best for:** Rapid development, keeping code DRY

```bash
# From repository root
cd ios/src

# Link shared business logic
ln -s ../../android/src/countdown.rs countdown.rs
ln -s ../../android/src/matching.rs matching.rs
ln -s ../../android/src/static_data.rs static_data.rs

# Link shared UI
rm -rf ui
ln -s ../../android/src/ui ui
```

**Pros:**
- Single source of truth
- Changes instantly reflected
- No duplication

**Cons:**
- Git may not handle symlinks on all platforms
- Can confuse some IDEs

### Option 3: Build Scripts (Production)

**Best for:** CI/CD pipelines

Create `ios/build.rs`:

```rust
use std::fs;
use std::path::Path;

fn main() {
    let files = vec![
        ("countdown.rs", "../android/src/countdown.rs"),
        ("matching.rs", "../android/src/matching.rs"),
        ("static_data.rs", "../android/src/static_data.rs"),
    ];

    for (dest, src) in files {
        let src_path = Path::new(src);
        let dest_path = Path::new("src").join(dest);
        fs::copy(src_path, dest_path)
            .expect(&format!("Failed to copy {}", src));
    }

    // Copy ui directory recursively
    copy_dir_all("../android/src/ui", "src/ui")
        .expect("Failed to copy ui directory");
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
```

---

## Platform-Specific Code

### iOS Components

Create iOS-specific implementations:

#### `ios/src/components/notification.rs`

```rust
/// iOS notification implementation using UserNotifications framework
use dioxus::prelude::*;

#[cfg(target_os = "ios")]
pub fn request_notification_permission() {
    // Use objc bindings to UserNotifications
    // UNUserNotificationCenter.current().requestAuthorization
}

#[cfg(target_os = "ios")]
pub fn schedule_notification(title: &str, body: &str, time: i64) {
    // UNMutableNotificationContent + UNTimeIntervalNotificationTrigger
}
```

#### `ios/src/components/storage.rs`

```rust
/// iOS storage implementation using UserDefaults
use serde::{Deserialize, Serialize};

#[cfg(target_os = "ios")]
pub fn save_addresses<T: Serialize>(addresses: &T) -> Result<(), String> {
    // Use objc bindings to NSUserDefaults
    // UserDefaults.standard.set(data, forKey: "addresses")
    Ok(())
}

#[cfg(target_os = "ios")]
pub fn load_addresses<T: for<'de> Deserialize<'de>>() -> Result<T, String> {
    // UserDefaults.standard.data(forKey: "addresses")
    Err("Not implemented".to_string())
}
```

#### `ios/src/components/geo.rs`

```rust
/// iOS geolocation using CoreLocation
use dioxus::prelude::*;

#[cfg(target_os = "ios")]
pub fn request_location_permission() {
    // CLLocationManager.requestWhenInUseAuthorization()
}

#[cfg(target_os = "ios")]
pub fn get_current_location() -> (f64, f64) {
    // CLLocationManager.location
    (0.0, 0.0)
}
```

### Android Components (Already Platform-Specific)

```rust
// android/src/components/notification.rs - Uses Android Notifications API
// android/src/components/storage.rs - Uses Android SharedPreferences  
// android/src/components/geo.rs - Uses Android Location Services
```

---

## iOS Main Entry Point

`ios/src/main.rs`:

```rust
use dioxus::prelude::*;
pub mod components;  // iOS-specific
pub mod countdown;   // Shared
pub mod matching;    // Shared
pub mod static_data; // Shared
pub mod ui;          // Shared

use ui::App;

fn main() {
    launch(App);
}
```

---

## iOS Build Configuration

`ios/Cargo.toml`:

```toml
[package]
name = "amp-ios"
version = "0.1.0"
edition = "2021"

[dependencies]
amp-core = { path = "../core" }
dioxus = { version = "0.6", features = ["mobile"] }
dioxus-std = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"

# iOS-specific
objc = "0.2"          # For Objective-C interop
block = "0.1"         # For block callbacks
cocoa = "0.25"        # Cocoa bindings
core-foundation = "0.9"  # CoreFoundation

[lib]
crate-type = ["staticlib", "cdylib", "rlib"]

[[bin]]
name = "amp-ios"
path = "src/main.rs"
```

---

## Building for iOS

### Prerequisites

```bash
# Install iOS targets
rustup target add aarch64-apple-ios      # iOS devices
rustup target add x86_64-apple-ios       # Simulator (Intel)
rustup target add aarch64-apple-ios-sim  # Simulator (Apple Silicon)

# Install cargo-bundle
cargo install cargo-bundle
```

### Build Commands

```bash
# Build for iOS device
cd ios
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator (Apple Silicon Mac)
cargo build --release --target aarch64-apple-ios-sim

# Build for iOS simulator (Intel Mac)
cargo build --release --target x86_64-apple-ios
```

### Xcode Integration

1. Create Xcode project
2. Add Rust static library as dependency
3. Configure build phases to compile Rust
4. Link iOS framework bindings

---

## Testing iOS Build

```bash
# Run on simulator
cd ios
dioxus serve --platform ios

# Or with cargo
cargo run --target aarch64-apple-ios-sim
```

---

## Code Sharing Summary

### ✅ Shared Between iOS & Android

| Module | Description | Lines |
|--------|-------------|-------|
| `countdown.rs` | Time calculation logic | ~150 |
| `matching.rs` | Address matching algorithms | ~100 |
| `static_data.rs` | Parquet data loading | ~50 |
| `ui/mod.rs` | Main app state and UI | ~250 |
| `ui/addresses.rs` | Address list UI | ~100 |
| `ui/panels.rs` | Parking restriction panels | ~300 |
| `ui/top_bar.rs` | Top bar UI component | ~180 |
| **Total Shared** | | **~1,130 lines** |

### ⚠️ Platform-Specific

| Platform | Module | iOS Implementation | Android Implementation |
|----------|--------|--------------------|------------------------|
| **Notifications** | `components/notification.rs` | UserNotifications | Android Notifications |
| **Storage** | `components/storage.rs` | UserDefaults | SharedPreferences |
| **GPS** | `components/geo.rs` | CoreLocation | Android Location |
| **Total Platform-Specific** | | **~200 lines** | **~200 lines** |

**Code Reuse:** ~85% shared between platforms

---

## Migration Checklist

### Phase 1: Setup Shared Structure ✓

- [x] Create `ios/` directory
- [x] Copy `Cargo.toml` configuration
- [x] Create `main.rs` entry point
- [ ] Choose sharing strategy (workspace/symlinks/build-script)
- [ ] Set up shared code linkage

### Phase 2: Implement Platform Components

- [ ] Create `ios/src/components/notification.rs`
- [ ] Create `ios/src/components/storage.rs`
- [ ] Create `ios/src/components/geo.rs`
- [ ] Add objc/cocoa dependencies
- [ ] Implement iOS-specific APIs

### Phase 3: Build & Test

- [ ] Install iOS Rust targets
- [ ] Configure Xcode project
- [ ] Build for simulator
- [ ] Build for device
- [ ] Test on iOS simulator
- [ ] Test on physical device

### Phase 4: CI/CD

- [ ] Add iOS build to GitHub Actions
- [ ] Configure code signing
- [ ] Set up TestFlight deployment
- [ ] Add App Store Connect integration

---

## Recommended Approach

**For Production:**

1. Create `shared/` crate with all shared logic
2. Move common code to `shared/src/`
3. Both `android` and `ios` depend on `shared`
4. Platform-specific code stays in respective crates
5. Clean separation, easy to maintain

**Directory after refactor:**

```
amp/
├── core/              # Data models
├── shared/            # NEW: Shared business logic
│   └── src/
│       ├── countdown.rs
│       ├── matching.rs
│       ├── static_data.rs
│       └── ui/
├── android/           # Android-specific only
│   └── src/
│       ├── main.rs
│       └── components/
└── ios/               # iOS-specific only
    └── src/
        ├── main.rs
        └── components/
```

---

## Next Steps

1. **Choose sharing strategy** (recommended: shared crate)
2. **Create shared crate** if using workspace approach
3. **Implement iOS platform components**
4. **Test iOS build locally**
5. **Add iOS to CI pipeline**

---

## Related Documentation

- **[Android README](../android/README.md)** - Android build instructions
- **[Core Models](../core/src/models.rs)** - Shared data structures
- **[Scripts Guide](../scripts/README.md)** - Build automation
- **[CI Workflow](../.github/workflows/ci.yml)** - GitHub Actions

---

**Status:** iOS structure prepared, awaiting platform component implementation

**Last Updated:** January 30, 2026
