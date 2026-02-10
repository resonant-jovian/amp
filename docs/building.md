# Building AMP

Comprehensive build instructions for all AMP components.

## Prerequisites

### All Platforms

- **Rust 1.70+** — [Install via rustup](https://rustup.rs)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Dioxus CLI** (for mobile apps)
  ```bash
  cargo install dioxus-cli
  ```

### Android-Specific

- **Android SDK** — Install via [Android Studio](https://developer.android.com/studio) or `sdkmanager`
- **Java 21** — [OpenJDK 21](https://adoptium.net/) or Oracle JDK 21
- **Android NDK** — Install via Android Studio SDK Manager

**Environment variables:**
```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
export JAVA_HOME="/path/to/java-21"
```

### iOS-Specific (macOS only)

- **Xcode 15+** — Install from App Store
- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```

## Core Library

Build the `amp_core` library:

```bash
cd core
cargo build --release
```

**Run tests:**
```bash
cargo test --release
```

**Generate documentation:**
```bash
cargo doc --no-deps --open
```

See [Core Library README](../core/README.md) for API details.

## CLI Tool

Build the `amp_server` CLI:

```bash
cd server
cargo build --release
```

**The binary will be at:** `target/release/amp_server`

**Install globally:**
```bash
cargo install --path server
```

**Usage:**
```bash
amp_server --help
amp_server test
amp_server correlate
amp_server benchmark
```

See [CLI Documentation](../server/README.md) for command reference.

## Android App

### Method 1: Using Build Script (Recommended)

The easiest way to build and install:

```bash
./scripts/build.sh
```

This script:
1. Builds Android APK in release mode
2. Signs the APK
3. Installs to connected device via ADB
4. Launches the app

### Method 2: Manual Build

```bash
cd android
dx build --release --platform android
```

**Output:** `target/dx/android/release/apk/amp.apk`

**Install to device:**
```bash
adb install -r target/dx/android/release/apk/amp.apk
```

### Development Build

For faster iteration during development:

```bash
cd android
dx serve --platform android
```

This enables hot-reload on device.

### Troubleshooting

**Error: "Android SDK not found"**
```bash
export ANDROID_HOME="$HOME/Android/Sdk"
```

**Error: "Java version mismatch"**
```bash
export JAVA_HOME="/path/to/java-21"
```

**Error: "NDK not found"**
```bash
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
```

See [Android README](../android/README.md) for architecture details.

## iOS App

### Build for Simulator

```bash
cd ios
dx build --platform ios
```

### Build for Device

```bash
cd ios
dx build --release --platform ios
```

**Note:** Device builds require:
- Apple Developer account
- Provisioning profile configured
- Code signing identity

### Run on Simulator

```bash
cd ios
dx serve --platform ios
```

### Troubleshooting

**Error: "Xcode not found"**
```bash
sudo xcode-select --switch /Applications/Xcode.app
```

**Error: "Provisioning profile expired"**
- Open project in Xcode
- Go to Signing & Capabilities
- Select your team and provisioning profile

See [iOS README](../ios/README.md) for setup details.

## Build Scripts

Automation scripts in `scripts/` directory:

### `build.sh`
Complete Android build pipeline:
```bash
./scripts/build.sh
```

### `adb-install.sh`
Install APK to connected device:
```bash
./scripts/adb-install.sh
```

### `fmt_fix_clippy.sh`
Format code and run linter:
```bash
./scripts/fmt_fix_clippy.sh
```

### `test.sh`
Run visual testing:
```bash
./scripts/test.sh
```

See [Scripts README](../scripts/README.md) for all available scripts.

## Build Optimization

### Release Builds

Always use `--release` for production:
```bash
cargo build --release
```

**Performance difference:**
- Debug: ~10x slower, larger binaries
- Release: Optimized, smaller binaries

### Link-Time Optimization (LTO)

Add to `Cargo.toml` for smaller binaries:

```toml
[profile.release]
lto = "thin"  # or "fat" for maximum optimization
codegen-units = 1
strip = true  # Remove debug symbols
```

**Trade-off:** Longer compile times, smaller binaries.

### Android APK Size

Reduce APK size:

1. **Enable ProGuard/R8:**
   - Configured in `android/Dioxus.toml`

2. **Remove unused architectures:**
   - Target only ARM64 for modern devices
   - Configured in build scripts

3. **Compress assets:**
   - Parquet files already compressed
   - Use zstd compression for additional savings

## Continuous Integration

GitHub Actions workflow (`.github/workflows/build.yml`):

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test --release
      - run: cargo clippy -- -D warnings
```

## Related Documentation

- **[Architecture](architecture.md)** — System overview
- **[Testing](testing.md)** — Testing guide
- **[Core Library](../core/README.md)** — Library API
- **[Android App](../android/README.md)** — Android architecture
- **[iOS App](../ios/README.md)** — iOS setup
- **[Scripts](../scripts/README.md)** — Build automation
