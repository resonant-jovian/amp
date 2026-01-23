# Core Library

Geospatial correlation engine for matching addresses to parking zones.

## Overview

The `amp_core` library provides:
- Six correlation algorithms
- ArcGIS API integration
- Data structures for addresses and parking zones
- Benchmarking framework
- Data verification with checksums

## Quick Start

Add to `Cargo.toml`:
```toml
[dependencies]
amp_core = { path = "../amp/core" }
tokio = { version = "1", features = ["full"] }
```

Basic usage:
```rust
use amp_core::api::api_miljo_only;
use amp_core::correlation_algorithms::{RTreeSpatialAlgo, CorrelationAlgo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch data from Malmö Open Data
    let (addresses, zones) = api_miljo_only()?;
    
    // Create R-Tree spatial index
    let algo = RTreeSpatialAlgo::new(&zones);
    
    // Correlate addresses
    for addr in addresses.iter().take(10) {
        if let Some((idx, dist)) = algo.correlate(addr, &zones) {
            println!("{}: {:.2}m to zone {}", addr.adress, dist, zones[idx].info);
        }
    }
    
    Ok(())
}
```

## Module Structure

```
core/src/
├── lib.rs                     # Public API
├── api.rs                     # ArcGIS data fetching
├── structs.rs                 # Data types
├── correlation_algorithms/    # Algorithm implementations
│   ├── mod.rs
│   ├── distance_based.rs      # O(n×m) brute-force
│   ├── raycasting.rs          # Ray intersection
│   ├── overlapping_chunks.rs  # Spatial grid
│   ├── rtree_spatial.rs       # R-tree index
│   ├── kdtree_spatial.rs      # KD-tree index
│   └── grid_nearest.rs        # Fixed grid
├── benchmark.rs               # Performance testing
├── checksum.rs                # Data verification
├── parquet.rs                 # Result storage
└── correlation_tests.rs       # Integration tests
```

## Data Types

### AdressClean

```rust
pub struct AdressClean {
    pub coordinates: [Decimal; 2],  // [lat, lon]
    pub postnummer: String,          // Postal code
    pub adress: String,              // Full address
    pub gata: String,                // Street name
    pub gatunummer: String,          // Street number
}
```

### MiljoeDataClean

```rust
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],  // Line segment [start, end]
    pub info: String,                     // Zone description
    pub tid: String,                      // Time restrictions
    pub dag: u8,                          // Day bitmask
}
```

### CorrelationResult

```rust
pub struct CorrelationResult {
    pub address: String,
    pub postnummer: String,
    pub miljo_match: Option<(f64, String)>,     // (distance, info)
    pub parkering_match: Option<(f64, String)>,
}
```

## Algorithms

All algorithms implement the `CorrelationAlgo` trait:

```rust
pub trait CorrelationAlgo {
    fn correlate(
        &self,
        address: &AdressClean,
        zones: &[MiljoeDataClean]
    ) -> Option<(usize, f64)>;  // (zone_index, distance)
}
```

### Distance-Based

```rust
use amp_core::correlation_algorithms::{DistanceBasedAlgo, CorrelationAlgo};

let algo = DistanceBasedAlgo;
let result = algo.correlate(&address, &zones);
```

**Use:** Small datasets (<1000 zones)

### R-Tree (Recommended)

```rust
use amp_core::correlation_algorithms::{RTreeSpatialAlgo, CorrelationAlgo};

let algo = RTreeSpatialAlgo::new(&zones);
let result = algo.correlate(&address, &zones);
```

**Use:** General purpose, production

### Overlapping Chunks

```rust
use amp_core::correlation_algorithms::{OverlappingChunksAlgo, CorrelationAlgo};

let algo = OverlappingChunksAlgo::new(&zones);
let result = algo.correlate(&address, &zones);
```

**Use:** Large datasets (>10K zones)

See [../docs/algorithms.md](../docs/algorithms.md) for complete algorithm comparison.

## API Functions

### Fetch All Datasets

```rust
use amp_core::api::api;

let (addresses, miljo_zones, parkering_zones) = api().await?;
```

### Fetch Miljödata Only

```rust
use amp_core::api::api_miljo_only;

let (addresses, zones) = api_miljo_only()?;
```

**Data Sources:**
- Miljöparkering (Environmental Parking)
- Parkeringsavgifter (Parking Fees)
- Adresser (Addresses)

See [../docs/api-integration.md](../docs/api-integration.md) for API details.

## Benchmarking

```rust
use amp_core::benchmark::Benchmarker;

let benchmarker = Benchmarker::new(addresses, zones);
let results = benchmarker.benchmark_all(Some(1000));

Benchmarker::print_results(&results);
```

**Output:**
```
Algorithm            Total Time    Avg/Address    Matches
──────────────────────────────────────────────────────
R-Tree              1.15s         2.30ms         423
```

## Data Verification

```rust
use amp_core::checksum::DataChecksum;

let mut checksums = DataChecksum::new(
    miljo_url.to_string(),
    parkering_url.to_string(),
    adress_url.to_string()
);

checksums.update_from_remote().await?;

if let Some(old) = DataChecksum::load_from_file("checksums.json").ok() {
    if checksums.has_changed(&old) {
        println!("Data updated!");
    }
}

checksums.save_to_file("checksums.json")?;
```

## Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test correlation_tests

# Benchmarks
cargo bench

# Documentation tests
cargo test --doc
```

See [../docs/testing.md](../docs/testing.md) for test strategy.

## Dependencies

Key dependencies:
- `rust_decimal` — High-precision coordinates
- `rayon` — Parallel processing
- `tokio` — Async runtime
- `reqwest` — HTTP client
- `serde` — Serialization
- `rstar` — R-tree spatial indexing
- `kiddo` — KD-tree spatial indexing
- `geojson` — GeoJSON parsing

See `Cargo.toml` for complete list.

## Examples

### Dual Dataset Correlation

```rust
use amp_core::api::api;
use amp_core::correlation_algorithms::{RTreeSpatialAlgo, CorrelationAlgo};
use amp_core::structs::CorrelationResult;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (addresses, miljo, parkering) = api().await?;
    
    let miljo_algo = RTreeSpatialAlgo::new(&miljo);
    let parkering_algo = RTreeSpatialAlgo::new(&parkering);
    
    for addr in addresses.iter().take(10) {
        let miljo_match = miljo_algo.correlate(addr, &miljo)
            .map(|(idx, dist)| (dist, miljo[idx].info.clone()));
        
        let parkering_match = parkering_algo.correlate(addr, &parkering)
            .map(|(idx, dist)| (dist, parkering[idx].info.clone()));
        
        let result = CorrelationResult {
            address: addr.adress.clone(),
            postnummer: addr.postnummer.clone(),
            miljo_match,
            parkering_match,
        };
        
        if result.has_match() {
            println!("{}: {}", result.address, result.dataset_source());
        }
    }
    
    Ok(())
}
```

### Custom Algorithm Implementation

```rust
use amp_core::correlation_algorithms::CorrelationAlgo;
use amp_core::structs::{AdressClean, MiljoeDataClean};

struct MyAlgorithm;

impl CorrelationAlgo for MyAlgorithm {
    fn correlate(
        &self,
        address: &AdressClean,
        zones: &[MiljoeDataClean]
    ) -> Option<(usize, f64)> {
        // Your implementation
        None
    }
}
```

## Performance Tips

**Choose right algorithm:**
- Small dataset: `DistanceBasedAlgo`
- General use: `RTreeSpatialAlgo`
- Large dataset: `OverlappingChunksAlgo`

**Parallel processing:**
```rust
use rayon::prelude::*;

let results: Vec<_> = addresses
    .par_iter()  // Parallel iterator
    .map(|addr| algo.correlate(addr, &zones))
    .collect();
```

**Pre-build indexes:**
```rust
// Build once, reuse many times
let algo = RTreeSpatialAlgo::new(&zones);

for address in addresses {
    algo.correlate(&address, &zones);  // Fast lookup
}
```

## Related Documentation

- [Algorithms](../docs/algorithms.md) — Algorithm comparison
- [API Integration](../docs/api-integration.md) — Data fetching
- [Architecture](../docs/architecture.md) — System design
- [Testing](../docs/testing.md) — Test strategy
