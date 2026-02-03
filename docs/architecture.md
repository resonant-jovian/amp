# Architecture

AMP uses a workspace-based architecture with shared core library and multiple frontend applications.

## System Components

```
┌────────────────────────┐
│   Malmö Open Data API   │
│  (GeoJSON datasets)      │
└─────────┬──────────────┘
         │
         │ fetch & parse
         │
         ↓
┌─────────┴─────────────┐
│      amp_core           │
│                        │
│  • API client         │
│  • Data structures    │
│  • Algorithms         │
│  • Parquet I/O        │
└───────┬────────────────┘
       │
       ├──────────────┬───────────────┬───────────┐
       │              │               │           │
       ↓              ↓               ↓           ↓
┌──────────┐  ┌─────────┐  ┌────────┐  ┌───────┐
│ amp_server │  │ android │  │  ios  │  │ ... │
│   (CLI)    │  │ (Dioxus)│  │(Dioxus)│  │     │
└──────────┘  └─────────┘  └────────┘  └───────┘
```

## Core Library (`amp_core`)

Provides reusable functionality for all frontend applications.

### Modules

- **`api`** — HTTP client for fetching GeoJSON from Malmö Open Data API
- **`structs`** — Core data structures (`DB`, `AdressClean`, `MiljoeDataClean`, etc.)
- **`parquet`** — Read/write operations for Apache Parquet format
- **`correlation_algorithms`** — Geospatial correlation implementations
- **`benchmark`** — Performance measurement utilities
- **`checksum`** — SHA-256 validation for data integrity

See [Core Library Documentation](../core/README.md) for API details.

## Data Flow

### 1. Data Acquisition

```rust
// Fetch from Malmö Open Data API
let addresses = api::fetch_addresses().await?;
let miljo_zones = api::fetch_miljo_zones().await?;
let parkering_zones = api::fetch_parkering_zones().await?;
```

### 2. Data Storage (Parquet)

```rust
// Write to efficient columnar format
parquet::write_addresses(&addresses, "addresses.parquet")?;
parquet::write_miljo(&miljo_zones, "miljo.parquet")?;
parquet::write_parkering(&parkering_zones, "parkering.parquet")?;
```

### 3. Correlation

```rust
use amp_core::correlation_algorithms::*;

// Load data
let addresses = parquet::read_addresses("addresses.parquet")?;
let miljo = parquet::read_miljo("miljo.parquet")?;
let parkering = parquet::read_parkering("parkering.parquet")?;

// Run algorithm
let results = kdtree_spatial::correlate(&addresses, &miljo, &parkering, 100.0);
```

See [Algorithms Documentation](algorithms.md) for algorithm details.

## Application Architecture

### CLI Tool (`amp_server`)

Command-line interface for testing and data management.

**Commands:**
- `test` — Visual testing with browser windows
- `correlate` — Run correlation and output results
- `benchmark` — Performance comparison of algorithms
- `serve` — Start HTTP server for web interface

See [CLI Documentation](../server/README.md).

### Mobile Apps (Android/iOS)

Offline-first applications built with Dioxus framework.

**Features:**
- Address search with autocomplete
- Real-time restriction checking
- Offline data storage (bundled Parquet files)
- Shared UI components between platforms

**Architecture:**
```
UI Layer (RSX components)
    ↓
State Management (Signals)
    ↓
Data Access (amp_core)
    ↓
Storage (Parquet files in assets/)
```

See [Android Documentation](../android/README.md) and [iOS Documentation](../ios/README.md).

## Data Structures

### Core Types

#### `DB`
Represents a parking restriction with time intervals.

```rust
pub struct DB {
    pub adress: String,
    pub gata: Option<String>,
    pub gatunummer: Option<String>,
    pub info: Option<String>,              // Miljödata restriction
    pub start_time: DateTime<Utc>,         // UTC timestamp
    pub end_time: DateTime<Utc>,           // UTC timestamp
    pub taxa: Option<String>,              // Parking zone
    pub antal_platser: Option<u64>,        // Number of spots
    pub typ_av_parkering: Option<String>,  // Parking type
}
```

**Time Handling:**
- All times stored as UTC timestamps
- Swedish timezone (Europe/Stockholm) for display
- Automatic DST handling via `chrono-tz`

#### `AdressClean`
Cleaned address data from Malmö database.

```rust
pub struct AdressClean {
    pub coordinates: [Decimal; 2],  // [longitude, latitude]
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}
```

#### `MiljoeDataClean` / `ParkeringsDataClean`
Restriction zones as line segments.

```rust
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],  // Line segment
    pub info: String,                     // Restriction text
    pub tid: String,                      // Time range ("0800-1200")
    pub dag: u8,                          // Day of month (1-31)
}
```

See [Data Format Documentation](data-format.md) for storage details.

## Configuration

### Workspace (`Cargo.toml`)

```toml
[workspace]
members = ["core", "android", "ios", "server"]
resolver = "2"

[workspace.dependencies]
rust_decimal = { version = "1.40.0", features = ["serde", "maths"] }
dioxus = { version = "0.7.3", features = ["mobile", "router"] }
geojson = "0.24.2"
parquet = "57.2.0"
# ... (see root Cargo.toml for full list)
```

### Dioxus Configuration

Mobile app configuration in `android/Dioxus.toml` and `ios/Dioxus.toml`:

```toml
[application]
name = "amp"
default_platform = "android"  # or "ios"

[android]
package = "se.skaggbyran.malmo.amp"

[bundle]
identifier = "se.skaggbyran.malmo.amp"
```

## Performance Considerations

### Algorithm Selection

| Algorithm | Speed | Accuracy | Use Case |
|-----------|-------|----------|----------|
| **KD-Tree** | Fast | High | Production (recommended) |
| **R-Tree** | Fast | High | Alternative spatial index |
| **Grid** | Fast | Medium | Quick approximation |
| **Distance** | Slow | Low | Baseline comparison |

See [Algorithms Documentation](algorithms.md) for benchmarks.

### Data Storage

- **Parquet** — Columnar format with compression (90% smaller than GeoJSON)
- **Checksums** — SHA-256 validation prevents corrupted data
- **Bundled Assets** — Mobile apps include Parquet files for offline use

## Related Documentation

- **[Algorithms](algorithms.md)** — Correlation algorithm details
- **[Data Format](data-format.md)** — Parquet schema and structure
- **[Building](building.md)** — Build instructions
- **[Testing](testing.md)** — Testing methodology
- **[API Integration](api-integration.md)** — Malmö Open Data API
