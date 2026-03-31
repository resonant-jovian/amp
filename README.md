<div align="center">

<h1>amp</h1>

[![Sponsor](https://img.shields.io/badge/Sponsor-resonant--jovian-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/resonant-jovian)
[![Support on thanks.dev](https://img.shields.io/badge/thanks.dev-Support-green?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0id2hpdGUiIGQ9Ik0xMiAyMWMtNS41IDAtMTAtMy41LTEwLTkgMC00IDItNy41IDYtMTAgMS41IDIuNSAzIDQuNSA0IDQuNSAxLTEuNSAyLjUtMy41IDQtNC41IDQuNSAyLjUgNiA2IDYgMTAgMCA1LjUtNC41IDktMTAgOXoiLz48L3N2Zz4=)](https://thanks.dev/u/gh/resonant-jovian)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-3366cc?style=for-the-badge&logo=gnu&logoColor=white)](https://www.gnu.org/licenses/gpl-3.0)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-3366cc?style=for-the-badge&logo=rust&logoColor=white)](https://releases.rs/docs/1.70.0/)
[![Edition](https://img.shields.io/badge/Edition-2024-3366cc?style=for-the-badge&logo=rust&logoColor=white)](https://doc.rust-lang.org/edition-guide/)

[![CI](https://img.shields.io/github/actions/workflow/status/resonant-jovian/amp/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=ci)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml)
[![Correlation](https://img.shields.io/github/actions/workflow/status/resonant-jovian/amp/correlation-tests.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=correlation)](https://github.com/resonant-jovian/amp/actions/workflows/correlation-tests.yml)
[![Benchmark](https://img.shields.io/github/actions/workflow/status/resonant-jovian/amp/server-benchmark.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=benchmark)](https://github.com/resonant-jovian/amp/actions/workflows/server-benchmark.yml)
[![Android](https://img.shields.io/github/actions/workflow/status/resonant-jovian/amp/android-test.yml?style=for-the-badge&logo=android&logoColor=white&label=android)](https://github.com/resonant-jovian/amp/actions/workflows/android-test.yml)
[![iOS](https://img.shields.io/github/actions/workflow/status/resonant-jovian/amp/ios-test.yml?style=for-the-badge&logo=apple&logoColor=white&label=ios)](https://github.com/resonant-jovian/amp/actions/workflows/ios-test.yml)

</div>


**Address-to-Miljözone Parking** — Geospatial correlation library for Swedish environmental parking zones in Malmö.

## Overview

AMP correlates street addresses with parking restriction zones using multiple geospatial algorithms. It consists of:

- **[Core Library](core/)** — Rust library with correlation algorithms and data structures
- **[CLI Tool](server/)** — Testing, benchmarking, and correlation command-line interface  
- **[Android App](android/)** — Offline parking lookup app with smart notifications
- **[iOS App](ios/)** — iOS version sharing UI code with Android
- **[Build Scripts](scripts/)** — Automation for building and deployment

## Quick Start

```bash
# Visual testing (opens browser tabs)
cargo run --release -- test

# Run correlation on addresses
cargo run --release -p amp_server -- correlate

# Benchmark algorithms
cargo run --release -p amp_server -- benchmark
```

## Documentation

### Core Concepts
- **[Architecture](docs/architecture.md)** — System design and data flow
- **[Algorithms](docs/algorithms.md)** — How geospatial correlation works
- **[Data Format](docs/data-format.md)** — Parquet storage structure

### Development
- **[Building](docs/building.md)** — Build instructions for all components
- **[Testing](docs/testing.md)** — Visual and automated testing guide
- **[API Integration](docs/api-integration.md)** — Fetching data from Malmö Open Data

### Android Notifications
- **[Notification System](docs/android-notifications.md)** — Complete notification implementation guide
- **[Kotlin Integration](docs/android-kotlin-integration.md)** — JNI bridge and Android setup

### Component Documentation
- **[Core Library](core/README.md)** — API reference and usage examples
- **[CLI Tool](server/README.md)** — Command reference and options
- **[Android App](android/README.md)** — Android-specific build and architecture
- **[iOS App](ios/README.md)** — iOS setup and code sharing
- **[Scripts](scripts/README.md)** — Automation scripts reference

## Features

### Core Library
- 🗺️ **Multiple algorithms**: KD-Tree, R-Tree, Quadtree, Grid-based
- 🚀 **Optimized**: Pre-computed correlations, O(1) lookups
- 📦 **Parquet storage**: Efficient binary format
- 🧪 **Visual testing**: Compare against official StadsAtlas

### Android App
- 📱 **Offline-first**: No internet required after initial data load
- ⏰ **Real-time countdowns**: Know exactly when restrictions apply
- 🔔 **Smart notifications**: Three-tier alert system (1 day, 6 hours, active)
- 💾 **Persistent storage**: Saves addresses locally
- 🎯 **Fuzzy matching**: Handles typos and partial addresses
- ✅ **Validity checking**: Accounts for date-dependent restrictions

## Project Structure

```
amp/
├── core/              # Rust library (algorithms, data structures)
│   ├── src/
│   │   ├── api.rs                    # Malmö Open Data API client
│   │   ├── parquet.rs                # Parquet read/write operations
│   │   ├── structs.rs                # Core data structures
│   │   ├── correlation_algorithms/   # Algorithm implementations
│   │   └── benchmark.rs              # Performance testing
│   └── README.md
├── server/            # CLI tool for testing and correlation
│   ├── src/main.rs
│   └── README.md
├── android/           # Android app (Dioxus)
│   ├── src/
│   │   ├── main.rs                   # App entry point
│   │   ├── ui/                       # UI components
│   │   ├── components/               # Business logic
│   │   │   ├── notifications.rs      # Notification system
│   │   │   ├── transitions.rs        # Panel transition detection
│   │   │   └── lifecycle.rs          # Background tasks
│   │   ├── storage.rs                # Data persistence
│   │   └── android_bridge.rs         # JNI bridge
│   ├── tests/                        # Integration tests
│   └── README.md
├── ios/               # iOS app (shares UI with Android)
│   ├── src/
│   └── README.md
├── scripts/           # Build and automation scripts
│   └── README.md
└── docs/              # General documentation
    ├── android-notifications.md      # Notification system guide
    └── android-kotlin-integration.md # Kotlin/JNI setup
```

## Building

### Prerequisites
- **Rust 1.70+** — [Install](https://rustup.rs)
- **Dioxus CLI** — `cargo install dioxus-cli` (for mobile apps)
- **Android SDK + Java 21** — For Android builds
- **Xcode** — For iOS builds (macOS only)

### Build Commands

```bash
# Core library and CLI
cargo build --release

# Run tests
cargo test --release

# Android APK
./scripts/build.sh

# iOS app
cd ios && dx build --release
```

See **[Building Guide](docs/building.md)** for detailed instructions.

## Testing

### Visual Testing
Compare algorithm results against official Malmö StadsAtlas:

```bash
# Default: 10 windows, KD-Tree algorithm
cargo run -- test

# Custom parameters
cargo run -- test --algorithm rtree --cutoff 100 --windows 15
```

### Automated Tests
```bash
# All tests
cargo test --release

# Android notification tests
cd android && cargo test --lib notifications
cd android && cargo test --lib transitions
cd android && cargo test --test notification_integration_tests
```

See **[Testing Guide](docs/testing.md)** for details.

## Android Notification System

The Android app includes a comprehensive notification system:

- **Three notification channels**:
  - 🔴 **Active Now**: Urgent alerts with sound + vibration + heads-up
  - 🟠 **6 Hours**: High-priority warnings with sound + vibration
  - 🟡 **1 Day**: Silent reminders in notification tray

- **Smart transition detection**: Only notifies when entering new time panels
- **User control**: Respects notification preferences in settings
- **No duplicates**: State tracking prevents repeat notifications

See **[Notification System Guide](docs/android-notifications.md)** for implementation details.

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden
