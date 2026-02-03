# AMP

**Address-to-Miljözone Parking** — Geospatial correlation library for Swedish environmental parking zones in Malmö.

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%2B-orange)](https://www.rust-lang.org/)

## Overview

AMP correlates street addresses with parking restriction zones using multiple geospatial algorithms. It consists of:

- **[Core Library](core/)** — Rust library with correlation algorithms and data structures
- **[CLI Tool](server/)** — Testing, benchmarking, and correlation command-line interface  
- **[Android App](android/)** — Offline parking lookup app built with Dioxus
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

### Component Documentation
- **[Core Library](core/README.md)** — API reference and usage examples
- **[CLI Tool](server/README.md)** — Command reference and options
- **[Android App](android/README.md)** — Android-specific build and architecture
- **[iOS App](ios/README.md)** — iOS setup and code sharing
- **[Scripts](scripts/README.md)** — Automation scripts reference

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
│   │   └── storage.rs                # Data persistence
│   └── README.md
├── ios/               # iOS app (shares UI with Android)
│   ├── src/
│   └── README.md
├── scripts/           # Build and automation scripts
│   └── README.md
└── docs/              # General documentation
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
cargo test --release
```

See **[Testing Guide](docs/testing.md)** for details.

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden
