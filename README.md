```
                                .         .                          
         .8.                   ,8.       ,8.          8 888888888o   
        .888.                 ,888.     ,888.         8 8888    `88. 
       :88888.               .`8888.   .`8888.        8 8888     `88 
      . `88888.             ,8.`8888. ,8.`8888.       8 8888     ,88 
     .8. `88888.           ,8'8.`8888,8^8.`8888.      8 8888.   ,88' 
    .8`8. `88888.         ,8' `8.`8888' `8.`8888.     8 888888888P'  
   .8' `8. `88888.       ,8'   `8.`88'   `8.`8888.    8 8888         
  .8'   `8. `88888.     ,8'     `8.`'     `8.`8888.   8 8888         
 .888888888. `88888.   ,8'       `8        `8.`8888.  8 8888         
.8'       `8. `88888. ,8'         `         `8.`8888. 8 8888         

```

**Address-to-Miljozone Parking** — Geospatial correlation library matching addresses to environmental parking zones in Malmö, Sweden.

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%2B-orange)](https://www.rust-lang.org/)

## Overview

AMP correlates street addresses with parking restriction zones using geospatial algorithms. It provides a Rust library, CLI tool, and mobile apps for checking parking restrictions without internet access.

**Key Features:**
- Six correlation algorithms (distance-based, raycasting, spatial indexing, grid-based)
- Dual dataset support (miljödata + parkering zones)
- CLI with testing mode, benchmarking, and data update checks
- Android and iOS apps built with Dioxus
- Visual testing interface with StadsAtlas integration

## Quick Start

### Testing Mode (Visual Verification)

Open browser windows to visually verify correlation accuracy against official StadsAtlas:

```bash
# Default: 10 windows, KD-Tree algorithm, 50m threshold
cargo run --release -- test

# Custom parameters
cargo run -- test --algorithm rtree --cutoff 100 --windows 15
```

**What each window shows:**
- **Tab 1:** Official Malmö StadsAtlas with parking zones
- **Tab 2:** Correlation result details (address, distance, zone info)

Manually verify that Tab 2 matches what you see in Tab 1's StadsAtlas.

See [docs/cli-usage.md](docs/cli-usage.md) for complete testing guide.

### CLI - Standard Commands

```bash
# Build
cargo build --release -p amp_server

# Run correlation (default: KD-Tree, 50m threshold)
./target/release/amp-server correlate

# Custom algorithm and distance threshold
./target/release/amp-server correlate --algorithm rtree --cutoff 75

# Benchmark algorithms interactively
./target/release/amp-server benchmark --sample-size 500

# Check if data has been updated
./target/release/amp-server check-updates
```

### Library Usage

```rust
use amp_core::api::api_miljo_only;
use amp_core::correlation_algorithms::{RTreeSpatialAlgo, CorrelationAlgo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (addresses, zones) = api_miljo_only()?;
    let algo = RTreeSpatialAlgo::new(&zones);
    
    for addr in addresses.iter().take(10) {
        if let Some((idx, dist)) = algo.correlate(addr, &zones) {
            println!("{}: {:.2}m to zone {}", addr.adress, dist, idx);
        }
    }
    Ok(())
}
```

## Documentation

### Getting Started
- **[CLI Usage](docs/cli-usage.md)** — Complete command reference
- **[Testing Guide](docs/testing.md)** — Visual and unit testing procedures

### Architecture & Design
- **[Architecture](docs/architecture.md)** — System design and data flow
- **[Algorithms](docs/algorithms.md)** — How each algorithm works
- **[API Integration](docs/api-integration.md)** — ArcGIS data fetching

### Module Guides
- **[Core Library](core/README.md)** — Library API and usage
- **[Server/CLI](server/README.md)** — CLI tool guide

## Project Structure

```
amp/
├── README.md              # This file
├── docs/                  # Documentation
│   ├── cli-usage.md       # CLI command reference
│   ├── testing.md         # Testing procedures
│   ├── architecture.md    # System design
│   ├── algorithms.md      # Algorithm details
│   └── api-integration.md # Data fetching
├── core/                  # Rust library crate
│   ├── README.md          # Library guide
│   └── src/
├── server/                # CLI tool crate
│   ├── README.md          # Server guide
│   └── src/
├── android/               # Android app (Dioxus)
├── ios/                   # iOS app (Dioxus)
└── build.sh              # Build script
```

## Building

### Prerequisites
- Rust 1.70+ ([rustup](https://rustup.rs))
- For mobile: Dioxus CLI (`cargo install dioxus-cli`)

### Build Commands

```bash
# Library and CLI
cargo build --release -p amp_core
cargo build --release -p amp_server

# Run tests
cargo test --release

# Android
cd android && dx build --release

# iOS
cd ios && dx build --release
```

## Dependencies

Core dependencies:
- `rust_decimal` — High-precision coordinates
- `rayon` — Parallel processing
- `tokio` — Async runtime
- `reqwest` — HTTP client
- `rstar` — R-tree spatial indexing
- `kiddo` — KD-tree spatial indexing
- `dioxus` — UI framework (mobile)

See `Cargo.toml` files for complete lists.

## Data Sources

AMP fetches parking zone data from official Malmö Open Data:
- **Miljöparkering** — Environmental parking restrictions
- **Parkeringsavgifter** — Parking fee zones
- **Adresser** — Address coordinates

Data is verified using checksums. Run `check-updates` to detect new data.

## Testing

### Visual Testing (Browser)

Test correlation accuracy by comparing results against official StadsAtlas:

```bash
# Quick test (5 windows)
cargo run -- test --windows 5

# Compare algorithms
cargo run -- test --algorithm kdtree --windows 10
cargo run -- test --algorithm rtree --windows 10

# Validate distance thresholds
cargo run -- test --cutoff 25 --windows 5
cargo run -- test --cutoff 50 --windows 5
cargo run -- test --cutoff 100 --windows 5
```

See [docs/testing.md](docs/testing.md) for detailed testing guide.

### Unit & Integration Tests

```bash
# All tests
cargo test --release

# Specific algorithm
cargo test --lib correlation_algorithms::rtree

# Benchmarks
cargo bench
```

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden

## Related Documentation

For detailed information, see:
- [CLI Usage Guide](docs/cli-usage.md) — All commands and parameters
- [Testing Strategies](docs/testing.md) — Visual, unit, and integration testing
- [Architecture Overview](docs/architecture.md) — System design
- [Algorithm Comparison](docs/algorithms.md) — How each algorithm works
- [API Integration](docs/api-integration.md) — Data fetching from ArcGIS
- [Core Library](core/README.md) — Library API documentation
- [Server Tool](server/README.md) — CLI tool documentation
