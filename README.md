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

# AMP

**Address-to-Miljozone Parking** — Geospatial correlation library matching addresses to environmental parking zones in Malmö, Sweden.

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%2B-orange)](https://www.rust-lang.org/)

## Overview

AMP correlates street addresses with parking restriction zones using geospatial algorithms. It provides a Rust library, interactive TUI, CLI tool, and mobile apps for checking parking restrictions without internet access.

**Key Features:**
- **Interactive Ratatui TUI** — Real-time correlation with live logs and performance charts
- **Six correlation algorithms** — Distance-based, raycasting, spatial indexing (R-Tree, KD-Tree), overlapping chunks, grid-based
- **Dual dataset support** — Miljödata + parkering zones
- **CLI tool** — Testing mode, benchmarking, data update checks
- **Visual testing** — Browser-based verification with StadsAtlas integration
- **Cross-platform** — Windows, macOS, Linux

## Quick Start

### Interactive TUI

```bash
# Launch the interactive Ratatui terminal interface
cargo run --release -p amp_server -- tui

# Features:
# - Algorithm multi-selection with checkboxes
# - Live log panel with color-coding
# - Performance charts (bar + line graphs)
# - Theme switcher (Light/Dark/Auto)
# - Full keyboard navigation
# - Help screen with all shortcuts
```

**Keyboard Shortcuts:**
- `[1-6]` / `[←→]` — Navigate tabs (Dashboard, Correlate, Results, Benchmark, Updates, Help)
- `[Space]` — Select/deselect algorithm
- `[a]` / `[c]` — Select all / Clear algorithms
- `[↑↓]` — Scroll content
- `[+/-]` — Adjust distance cutoff
- `[t]` — Toggle theme (Light/Dark/Auto)
- `[Enter]` — Run operation
- `[?]` — Show help screen
- `[Ctrl+C]` / `[q]` — Quit

See [docs/CLI_INTEGRATION_GUIDE.md](docs/CLI_INTEGRATION_GUIDE.md) for complete TUI guide.

### CLI - Testing Mode

Visually verify correlation accuracy by comparing results against official StadsAtlas:

```bash
# Build
cargo build --release -p amp_server

# Open 10 browser windows with random addresses
./target/release/amp-server test

# Custom algorithm and distance threshold
./target/release/amp-server test --algorithm rtree --cutoff 100 --windows 15
```

**What happens:**
- Runs correlation with selected algorithm
- Finds matching addresses (with zone info)
- Opens N browser windows with test pages
- Each page: address, zones, embedded Origo map
- 4 tabs per window: Address Search, Instructions, Data, Debug Console
- Manually verify zone matches against official Malmö StadsAtlas

See [docs/CLI_INTEGRATION_GUIDE.md](docs/CLI_INTEGRATION_GUIDE.md#3-test-mode-cli) for detailed testing guide.

### CLI - Correlation

```bash
# Run correlation with KD-Tree algorithm (default)
cargo run --release -p amp_server -- correlate

# Custom algorithm and distance threshold  
cargo run -- correlate --algorithm rtree --cutoff 75

# Available algorithms:
# - distance-based
# - raycasting
# - overlapping-chunks
# - rtree
# - kdtree (default)
# - grid
```

**Output includes:**
- Dataset information
- Progress bars for each dataset
- Result statistics (match counts, percentages)
- 10 random matching addresses
- Top 10 addresses with largest distances
- Threshold compliance verification

See [docs/CLI_INTEGRATION_GUIDE.md#2-correlate-cli](docs/CLI_INTEGRATION_GUIDE.md#2-correlate-cli) for complete reference.

### CLI - Benchmarking

```bash
# Interactive algorithm selection (default: all selected)
cargo run -- benchmark --sample-size 100

# Custom sample size and cutoff
cargo run -- benchmark --sample-size 500 --cutoff 75
```

**Features:**
- Interactive Y/n prompts for each algorithm (default: yes)
- Progress bars during benchmarking
- Comparative results table
- Time per address statistics

See [docs/CLI_INTEGRATION_GUIDE.md#4-benchmark-cli](docs/CLI_INTEGRATION_GUIDE.md#4-benchmark-cli) for examples.

### CLI - Check Updates

```bash
# Check for data updates from Mälmö open data portal
cargo run -- check-updates

# Custom checksum file
cargo run -- check-updates --checksum-file my-checksums.json
```

Detects changes in environmental parking and street address data.

### Library

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

See [core/README.md](core/README.md) for library documentation.

## Project Structure

```
amp/
├── README.md              # This file
├── docs/                  # Documentation
│   ├── IMPLEMENTATION_COMPLETE.md  # Integration summary
│   ├── CLI_INTEGRATION_GUIDE.md     # Complete CLI reference
│   ├── architecture.md    # System design
│   ├── algorithms.md      # Algorithm details
│   ├── api-integration.md # Data fetching
│   ├── testing.md         # Testing strategies
│   └── implementation-notes.md  # Technical details
├── core/                  # Rust library crate
│   ├── README.md          # Library guide
│   └── src/
│       ├── lib.rs
│       ├── api.rs
│       ├── structs.rs
│       ├── correlation_algorithms/
│       ├── benchmark.rs
│       ├── checksum.rs
│       └── correlation_tests.rs
├── server/                # CLI tool + TUI crate
│   ├── README.md          # Server guide
│   ├── src/
│   │   ├── main.rs       # Command router
│   │   ├── cli.rs        # CLI handlers (NEW)
│   │   ├── ui.rs         # Ratatui TUI
│   │   ├── app.rs        # App state
│   │   ├── tui.rs        # Terminal management
│   │   └── classification.rs
│   └── assets/        # UI templates
│       ├── stadsatlas_interface.html
│       ├── stadsatlas_interface.css
│       ├── stadsatlas_interface.js
│       └── origo_map.html
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
# Core library
cargo build --release -p amp_core

# CLI server (includes TUI and all commands)
cargo build --release -p amp_server

# Run tests
cargo test --release

# Android
cd android && dx build --release

# iOS
cd ios && dx build --release
```

## Architecture

### Module Organization

```
server/src/
├── main.rs              # Command router - directs to CLI or TUI
├── cli.rs               # CLI handlers - all command implementations (NEW)
├── ui.rs                # Ratatui TUI - interactive interface
├── app.rs               # App state management
├── tui.rs               # Terminal management
└── classification.rs    # Address classification logic
```

### Command Routing

```
amp-server [COMMAND] [OPTIONS]
    ↓
main.rs (CLI::parse())
    ↓
Route to handler:
├─ correlate → cli::run_correlation()
├─ tui → ui::App::new() + app.run()
├─ test → cli::run_test_mode()
├─ benchmark → cli::run_benchmark()
└─ check-updates → cli::check_updates()
```

## Documentation

### Getting Started
- **[Quick Start](#quick-start)** — Run TUI, testing, or CLI
- **[CLI Integration Guide](docs/CLI_INTEGRATION_GUIDE.md)** — Complete command reference with examples
- **[Implementation Complete](docs/IMPLEMENTATION_COMPLETE.md)** — Integration summary

### Architecture & Design
- **[Architecture](docs/architecture.md)** — System design and data flow
- **[Algorithms](docs/algorithms.md)** — How each algorithm works
- **[API Integration](docs/api-integration.md)** — ArcGIS data fetching
- **[Implementation Notes](docs/implementation-notes.md)** — Technical details

### Module Documentation
- **[Core Library](core/README.md)** — Library API and usage
- **[Server/CLI](server/README.md)** — CLI and TUI tool guide

## Dependencies

Core dependencies:
- `tokio` — Async runtime
- `clap` — CLI argument parsing
- `ratatui` — Terminal UI framework (TUI)
- `crossterm` — Terminal control
- `rayon` — Parallel processing
- `indicatif` — Progress bars
- `rust_decimal` — High-precision coordinates
- `rstar` — R-tree spatial indexing
- `kiddo` — KD-tree spatial indexing
- `reqwest` / `serde` — HTTP and serialization

See `Cargo.toml` files for complete dependency lists. **No new dependencies were added for TUI integration.**

## Testing

### Manual Testing

```bash
# Interactive TUI
cargo run --release -p amp_server -- tui

# Test CLI correlation
cargo run -- correlate --algorithm kdtree --cutoff 50

# Test mode (opens browser windows)
cargo run -- test --windows 1

# Benchmark
cargo run -- benchmark --sample-size 100

# Check for updates
cargo run -- check-updates
```

See [docs/CLI_INTEGRATION_GUIDE.md#testing-sequence](docs/CLI_INTEGRATION_GUIDE.md#testing-sequence) for comprehensive testing procedure.

### Unit Tests

```bash
# Run all tests
cargo test --release

# Run specific algorithm tests
cargo test --lib correlation_algorithms::rtree_spatial

# Run benchmarks
cargo bench
```

## Data Sources

AMP fetches parking zone data from official Mälmö Open Data:

- **Miljöparkering** — Environmental parking restrictions
- **Parkeringsavgifter** — Parking fee zones
- **Adresser** — Address coordinates

Data is verified using checksums to detect updates. See [docs/api-integration.md](docs/api-integration.md) for details.

## Performance

### Typical Execution Times

| Operation | Time | Note |
|-----------|------|------|
| TUI startup | < 100ms | Instant |
| Correlate (full dataset) | 30-90s | Depends on algorithm |
| Benchmark (100 addresses) | 30-45s | All 6 algorithms |
| Test mode (1 window) | 60-120s | Including browser startup |
| Check updates | 10-20s | Network dependent |

### Memory Usage

- Typical: 30-50MB
- Peak (full benchmark): ~100-150MB
- Should always stay < 500MB

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden

---

### Recent Updates

**✅ Feature/Testing Branch - Ratatui TUI + CLI Integration**

- Added interactive Ratatui terminal interface (`tui` command)
- Restored full CLI functionality (correlate, test, benchmark, check-updates)
- Separated concerns: `main.rs` (router) + `cli.rs` (handlers) + `ui.rs` (TUI)
- All asset files working with browser test mode
- 0 new dependencies (all from workspace)
- Comprehensive documentation added

See [docs/IMPLEMENTATION_COMPLETE.md](docs/IMPLEMENTATION_COMPLETE.md) for details.
