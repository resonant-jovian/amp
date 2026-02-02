# Architecture

AMP follows a modular architecture with clear separation between correlation logic, CLI tooling, and mobile apps.

## System Overview

```
┌────────────────────────────────┐
│     Malmö Open Data APIs      │
│  (Adresser, Miljöparkering)  │
└──────────────┬─────────────────┘
               │
               │ HTTP/GeoJSON
               │
       ┌───────┴───────┐
       │   amp_core    │
       │ (Rust Library)│
       └─────┬───┬──────┘
            │    │
    ┌───────┼────┼───────┐
    │       │    │        │
┌───┴───┐  │    │   ┌───┴───┐
│ Server │  │    │   │Android│
│  (CLI) │  │    └───┤  iOS  │
└───────┘  │        └───────┘
         Parquet
```

## Core Components

### 1. Core Library (`amp_core`)

The foundational Rust library providing:

- **Data Structures** (`structs.rs`)
  - `Address`: Street address with coordinates
  - `MiljoParkering`: Parking restriction zone
  - `Segment`: Line segment for zone boundaries

- **Correlation Algorithms** (`correlation_algorithms/`)
  - `KDTreeSpatialAlgo`: Fast spatial indexing (default)
  - `RTreeSpatialAlgo`: R-tree based spatial queries
  - `DistanceBasedAlgo`: Simple distance calculation
  - `RaycastingAlgo`: Point-in-polygon testing
  - `GridNearestAlgo`: Grid-based spatial partitioning
  - `OverlappingChunksAlgo`: Chunked processing for large datasets

- **API Integration** (`api.rs`)
  - Fetches GeoJSON from Malmö Open Data
  - Parses addresses and parking zones
  - Validates data integrity with checksums

- **Parquet Storage** (`parquet.rs`)
  - Converts GeoJSON to efficient Parquet format
  - Used for offline mobile app data

See [Core Library README](../core/README.md) for API details.

### 2. CLI Tool (`amp_server`)

Command-line interface for:

- **Testing** (`test` command)
  - Visual verification via browser
  - Compares results against official StadsAtlas
  - Configurable algorithm and distance thresholds

- **Correlation** (`correlate` command)
  - Batch processing of addresses
  - Algorithm selection and benchmarking

- **Data Updates** (`check-updates` command)
  - Detects when Open Data has changed
  - Validates checksums

See [CLI Usage](cli-usage.md) for command reference.

### 3. Mobile Apps

Both Android and iOS apps share ~85% of code:

**Shared Components:**
- Address matching logic
- Parking deadline countdown
- Parquet data loading
- All UI components

**Platform-Specific:**
- GPS/location services
- Notifications
- Persistent storage

See [Android README](../android/README.md) and [iOS README](../ios/README.md).

## Data Flow

### CLI Workflow

1. Fetch GeoJSON from Malmö APIs
2. Parse into internal data structures
3. Initialize chosen correlation algorithm
4. Process addresses in parallel (Rayon)
5. Output results (JSON, visual browser tabs)

### Mobile App Workflow

1. Load embedded Parquet files at startup
2. User enters address or uses GPS
3. Match address to stored data (fuzzy matching)
4. Find nearest parking zone (correlation)
5. Calculate deadline and display
6. Schedule notification

## Algorithm Selection

Algorithms are selected via the `CorrelationAlgo` trait:

```rust
pub trait CorrelationAlgo: Send + Sync {
    fn correlate(
        &self, 
        address: &Address, 
        zones: &[MiljoParkering]
    ) -> Option<(usize, f64)>;
}
```

**Performance Characteristics:**

| Algorithm | Build Time | Query Time | Memory | Best For |
|-----------|------------|------------|--------|----------|
| KD-Tree   | O(n log n) | O(log n)   | Medium | Default choice |
| R-Tree    | O(n log n) | O(log n)   | Medium | Dense zones |
| Distance  | O(1)       | O(n)       | Low    | Small datasets |
| Grid      | O(n)       | O(1)       | High   | Uniform distribution |
| Raycasting| O(1)       | O(n)       | Low    | Polygon zones |

See [Algorithms](algorithms.md) for detailed comparisons.

## Code Organization

### Naming Conventions

- **Code**: English (variables, functions, types)
- **UI**: Swedish (user-facing strings)
- **Documentation**: English (all markdown files)

This maintains code maintainability while preserving Swedish context for end users.

### Module Structure

```
core/src/
├── lib.rs                    # Public API
├── structs.rs                # Core data types
├── api.rs                    # Data fetching
├── parquet.rs                # Parquet conversion
├── checksum.rs               # Data validation
├── benchmark.rs              # Performance testing
└── correlation_algorithms/   # Algorithm implementations
    ├── mod.rs                # Trait definition
    ├── common.rs             # Shared utilities
    ├── kdtree_spatial.rs     # KD-Tree
    ├── rtree_spatial.rs      # R-Tree
    ├── distance_based.rs     # Distance
    ├── raycasting.rs         # Raycasting
    ├── grid_nearest.rs       # Grid
    └── overlapping_chunks.rs # Chunks
```

## Dependencies

### Core Dependencies
- `rust_decimal`: High-precision coordinates
- `rayon`: Parallel processing
- `rstar`: R-tree spatial indexing
- `kiddo`: KD-tree implementation
- `geodesy`: Coordinate transformations

### Mobile Dependencies
- `dioxus`: Cross-platform UI framework
- `parquet`: Efficient data storage
- `arrow`: Parquet data access

### CLI Dependencies
- `clap`: Command-line argument parsing
- `tokio`: Async runtime
- `reqwest`: HTTP client

See workspace `Cargo.toml` for version details.

## Related Documentation

- [Algorithms](algorithms.md) — Algorithm details and benchmarks
- [API Integration](api-integration.md) — Data source details
- [Data Format](data-format.md) — Parquet structure
- [Testing](testing.md) — Testing strategies
