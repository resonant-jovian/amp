# Changelog

All notable changes to the AMP project.

## [Unreleased]

### Android App
- Native Android implementation with Dioxus
- Offline operation with embedded Parquet data
- Address search with autocomplete
- Current location detection
- Real-time countdown timer for restrictions
- Day/time extraction for optimized lookups

### Core Library
- Six correlation algorithms (KD-Tree, R-Tree, Grid, Overlapping Chunks, Distance-Based, Raycasting)
- Dual dataset support (Miljöparkering + Parkeringsavgifter)
- Parquet-based storage for results
- Benchmarking framework
- Data verification with checksums

### CLI Tool
- Visual testing mode with browser integration
- Benchmark command for algorithm comparison
- Output command for generating mobile-ready Parquet files
- Check-updates command for data monitoring

## [2026-01] - January 2026

### Changed
- Refactored correlation algorithms into modular structure
- Standardized `CorrelationAlgo` trait across all algorithms
- Migrated from dual correlation functions to unified trait-based approach
- Improved algorithm module organization

### Fixed
- MultiLineString segment handling in zone boundaries
- Naming inconsistencies across modules
- Compilation issues with parking dataset types

## [2025-12] - December 2025

### Added
- Initial Android app implementation
- Parquet integration for data storage
- R-Tree and KD-Tree spatial indexing
- Visual testing framework with StadsAtlas integration

### Changed
- Migrated from JSON to Parquet format
- Split monolithic correlation into separate algorithms

## [2025-11] - November 2025

### Added
- Core library with basic distance-based correlation
- ArcGIS API integration for Malmö Open Data
- Initial CLI tool for testing
- Raycasting algorithm
- Grid-based nearest neighbor

### Changed
- Restructured project into workspace (core, server, android)
- Adopted rust_decimal for coordinate precision

## [2025-10] - October 2025

### Added
- Initial project setup
- Basic address-to-zone correlation
- Haversine distance calculations
- GeoJSON parsing

## Version History

### Algorithm Evolution

**Phase 1: Brute Force (October 2025)**
- Simple distance-based correlation
- O(n × m) complexity
- Proof of concept

**Phase 2: Spatial Indexing (November 2025)**
- Grid-based partitioning
- R-Tree implementation
- ~200x performance improvement

**Phase 3: Optimization (December 2025)**
- KD-Tree spatial index
- Parallel processing with Rayon
- Overlapping chunks algorithm

**Phase 4: Production Ready (January 2026)**
- Trait-based architecture
- Modular algorithm selection
- Comprehensive benchmarking

### Data Format Evolution

**JSON → Parquet (December 2025)**
- Switched from JSON to Parquet for efficiency
- 10x faster reads, 5x smaller files
- Better compression and column-based access

**Schema Refinement (January 2026)**
- Added day bitmask for quick filtering
- Extracted time ranges for countdown feature
- Optimized for mobile app constraints

### Android Implementation

**Initial Version (December 2025)**
- Basic Dioxus UI
- Embedded data
- Simple search

**Enhanced Version (January 2026)**
- Autocomplete search
- GPS location
- Real-time countdown
- Settings persistence
- Notification support

## Migration Guides

### Algorithm API Changes

**Before (November 2025):**
```rust
let results = distance_based::correlate(&addresses, &zones, cutoff);
```

**After (January 2026):**
```rust
use amp_core::correlation_algorithms::{DistanceBasedAlgo, CorrelationAlgo};

let algo = DistanceBasedAlgo;
let result = algo.correlate(&address, &zones);
```

### Data Format Changes

**Before (November 2025):**
```json
{
  "address": "Stora Nygatan 1",
  "zone": "Miljözon A",
  "distance": 15.3
}
```

**After (December 2025):**
Parquet schema with typed columns

## Contributors

- **Albin Sjögren** - Project lead and primary developer

## License

GPL-3.0 - See [LICENSE](../LICENSE) for details.
