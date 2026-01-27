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

AMP correlates street addresses with parking restriction zones using geospatial algorithms. It provides a Rust library, interactive TUI, and mobile apps for checking parking restrictions without internet access.

**Key Features:**
- **Interactive Ratatui TUI** — Real-time correlation with live logs and performance charts
- **Six correlation algorithms** — Distance-based, raycasting, spatial indexing (R-Tree, KD-Tree), overlapping chunks, grid-based
- **Dual dataset support** — Miljödata + parkering zones
- **Visual testing interface** — Browser-based verification with StadsAtlas integration
- **Cross-platform** — Windows, macOS, Linux

## Quick Start

### Interactive TUI

```bash
# Build and launch the interactive Ratatui terminal interface
cargo build --release -p amp_server
./target/release/amp-server
```

**Features:**
- Algorithm multi-selection with checkboxes
- Live log panel with color-coding
- Performance charts (bar + line graphs)
- Theme switcher (Light/Dark/Auto)
- Full keyboard navigation
- Help screen with all shortcuts

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
│   ├── CLI_INTEGRATION_GUIDE.md     # TUI reference
│   ├── architecture.md    # System design
│   ├── algorithms.md      # Algorithm details
│   ├── api-integration.md # Data fetching
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
├── server/                # TUI application crate
│   ├── README.md          # Server guide
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── ui.rs          # Ratatui TUI
│   │   ├── app.rs         # App state
│   │   ├── tui.rs         # Terminal management
│   │   └── classification.rs
│   └── assets/            # Web test assets
│       ├── stadsatlas_interface.html
│       ├── stadsatlas_interface.css
│       ├── stadsatlas_interface.js
│       ├── origo_map.html
│       └── amp_test_*.html (generated)
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

# TUI application
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
├── main.rs              # Entry point - launches TUI
├── ui.rs                # Ratatui TUI - interactive interface
├── app.rs               # App state management
├── tui.rs               # Terminal management
└── classification.rs    # Address classification logic
```

### Application Flow

```
amp-server
    ↓
main.rs
    ↓
ui::App::new()
    ↓
app.run() — Terminal event loop
    ↓
┌─────────────────────────────────┐
│   6 Views:                       │
│   1. Dashboard                   │
│   2. Correlate                   │
│   3. Results                     │
│   4. Benchmark                   │
│   5. Updates                     │
│   6. Help                        │
│                                  │
│   Features:                      │
│   - Algorithm selection          │
│   - Live log panel               │
│   - Performance charts           │
│   - Theme switcher               │
│   - Responsive layouts           │
│   - Keyboard navigation          │
└─────────────────────────────────┘
    ↓
amp-core algorithms & data loading
```

## Documentation

### Getting Started
- **[Quick Start](#quick-start)** — Run the TUI
- **[CLI Integration Guide](docs/CLI_INTEGRATION_GUIDE.md)** — Complete TUI reference
- **[Implementation Complete](docs/IMPLEMENTATION_COMPLETE.md)** — Integration summary

### Architecture & Design
- **[Architecture](docs/architecture.md)** — System design and data flow
- **[Algorithms](docs/algorithms.md)** — How each algorithm works
- **[API Integration](docs/api-integration.md)** — ArcGIS data fetching
- **[Implementation Notes](docs/implementation-notes.md)** — Technical details

### Module Documentation
- **[Core Library](core/README.md)** — Library API and usage
- **[Server/TUI](server/README.md)** — TUI application guide

## Dependencies

Core dependencies:
- `tokio` — Async runtime
- `ratatui` — Terminal UI framework
- `crossterm` — Terminal control
- `rayon` — Parallel processing
- `indicatif` — Progress bars
- `rust_decimal` — High-precision coordinates
- `rstar` — R-tree spatial indexing
- `kiddo` — KD-tree spatial indexing
- `reqwest` / `serde` — HTTP and serialization

See `Cargo.toml` files for complete dependency lists.

## Testing

### Launch TUI

```bash
cargo run --release -p amp_server
```

Inside the TUI:
- Select algorithms with [Space]
- Adjust cutoff with [+/-]
- Run correlation with [Enter]
- Toggle theme with [t]
- View results in Results tab
- Check benchmarks in Benchmark tab
- See logs in live log panel

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

AMP fetches parking zone data from official Malmö Open Data:

- **Miljöparkering** — Environmental parking restrictions
- **Parkeringsavgifter** — Parking fee zones
- **Adresser** — Address coordinates

Data is verified using checksums to detect updates. See [docs/api-integration.md](docs/api-integration.md) for details.

## Performance

### Typical Execution Times

| Operation | Time | Note |
|-----------|------|------|
| TUI startup | < 100ms | Instant |
| Correlate (full dataset) | 30-90s | Real-time in TUI |
| Benchmark (100 addresses) | 30-45s | All 6 algorithms |
| Check updates | 10-20s | Network dependent |

### Memory Usage

- Baseline TUI: 15-20MB
- During correlation: 30-50MB
- Peak (full benchmark): 100-150MB
- Should always stay < 500MB

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Contact

**Albin Sjögren**  
[albin@sjoegren.se](mailto:albin@sjoegren.se)  
Malmö, Sweden

---

### Recent Updates

**✅ Feature/Testing Branch - Pure TUI Interface**

- Removed all CLI command-line arguments
- TUI is now the primary and only interface
- Direct launch: `amp-server` (no commands)
- Integrated correlate, test, benchmark, and updates functionality in TUI
- All asset files working with embedded web tests
- Clean, focused implementation

See [docs/IMPLEMENTATION_COMPLETE.md](docs/IMPLEMENTATION_COMPLETE.md) for details.
