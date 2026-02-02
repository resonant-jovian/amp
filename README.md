# AMP

**Address-to-Miljözone Parking** — Geospatial correlation library matching Swedish addresses to environmental parking zones in Malmö.

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%2B-orange)](https://www.rust-lang.org/)

## What is AMP?

AMP correlates street addresses with parking restriction zones using geospatial algorithms. It provides:
- **Rust library** (`amp_core`) for correlation algorithms
- **CLI tool** (`amp_server`) for testing and benchmarking
- **Mobile apps** for offline parking zone lookup (Android/iOS)

## Quick Start

```bash
# Run visual testing (opens browser tabs)
cargo run --release -- test

# Correlate addresses
cargo run --release -p amp_server -- correlate

# Benchmark algorithms
cargo run --release -p amp_server -- benchmark
```

## Project Structure

```
amp/
├── core/           # Rust library (correlation algorithms)
├── server/         # CLI tool (testing, benchmarking)
├── android/        # Android app (Dioxus)
├── ios/            # iOS app (Dioxus)
├── scripts/        # Build and deployment scripts
└── docs/           # Documentation
```

## Documentation

### Getting Started
- [Architecture](docs/architecture.md) — System design and data flow
- [Algorithms](docs/algorithms.md) — How correlation algorithms work
- [CLI Usage](docs/cli-usage.md) — Command reference
- [Testing Guide](docs/testing.md) — Visual and automated testing

### Development
- [Core Library](core/README.md) — Library API and usage
- [CLI Tool](server/README.md) — Server/CLI documentation
- [Android App](android/README.md) — Android build and deployment
- [iOS App](ios/README.md) — iOS setup and code sharing
- [Build Scripts](scripts/README.md) — Automation scripts

### Technical Details
- [API Integration](docs/api-integration.md) — Data fetching from Malmö Open Data
- [Data Format](docs/data-format.md) — Parquet storage and structure

## Building

### Prerequisites
- Rust 1.70+ ([install](https://rustup.rs))
- Dioxus CLI: `cargo install dioxus-cli` (for mobile apps)
- Android SDK + Java 21 (for Android)
- Xcode (for iOS)

### Build Commands

```bash
# Library and CLI
cargo build --release

# Run tests
cargo test --release

# Android APK
./scripts/build.sh

# iOS
cd ios && dx build --release
```

## Testing

### Visual Testing
Test correlation accuracy by comparing results against official Malmö StadsAtlas:

```bash
# Default: 10 windows, KD-Tree algorithm
cargo run -- test

# Custom parameters
cargo run -- test --algorithm rtree --cutoff 100 --windows 15
```

See [Testing Guide](docs/testing.md) for details.

### Automated Tests
```bash
cargo test --release
```

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden
