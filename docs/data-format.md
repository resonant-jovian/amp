# Data Format

AMP uses Apache Parquet for efficient storage of geospatial data.

## Why Parquet?

**Advantages:**
- **Columnar storage** — Efficient compression and queries
- **90% size reduction** — Compared to GeoJSON
- **Fast reads** — Skip irrelevant columns
- **Type safety** — Schema enforced at read/write
- **Cross-platform** — Supported by many tools

**Comparison:**
| Format | Size (MB) | Read Time (ms) | Write Time (ms) |
|--------|-----------|----------------|------------------|
| GeoJSON | 250 | 1,200 | 800 |
| Parquet | 25 | 150 | 200 |

## File Structure

### Addresses (`addresses.parquet`)

**Schema:**
```rust
struct AdressClean {
    coordinates: [Decimal; 2],  // [longitude, latitude]
    postnummer: Option<String>, // Postal code
    adress: String,             // Full address
    gata: String,               // Street name
    gatunummer: String,         // Street number
}
```

**Source:** Malmö address database via Open Data API

**Example data:**
```
coordinates     | postnummer | adress              | gata        | gatunummer
[13.003, 55.60] | 21438      | Åhusgatan 1        | Åhusgatan   | 1
[12.994, 55.61] | 21439      | Beijerskajen 10    | Beijerskajen| 10
```

### Miljödata (`miljo.parquet`)

Environmental parking restrictions (street cleaning).

**Schema:**
```rust
struct MiljoeDataClean {
    coordinates: [[Decimal; 2]; 2], // Line segment
    info: String,                    // Restriction text
    tid: String,                     // Time range "HHMM-HHMM"
    dag: u8,                         // Day of month (1-31)
}
```

**Example data:**
```
coordinates                              | info                    | tid        | dag
[[13.003, 55.60], [13.004, 55.601]]    | Parkering förbjuden     | 0800-1200  | 15
[[12.994, 55.61], [12.995, 55.611]]    | Renhållning             | 1200-1600  | 17
```

**Time format:**
- `tid`: "HHMM-HHMM" (e.g., "0800-1200" = 8 AM to 12 PM)
- `dag`: Day of month when restriction applies

### Parkering (`parkering.parquet`)

Regular parking zones (paid parking).

**Schema:**
```rust
struct ParkeringsDataClean {
    coordinates: [[Decimal; 2]; 2],  // Line segment
    taxa: String,                     // Zone identifier (A-E)
    antal_platser: u64,               // Number of spots
    typ_av_parkering: String,         // Type (e.g., "Längsgående 6")
}
```

**Example data:**
```
coordinates                              | taxa   | antal_platser | typ_av_parkering
[[13.003, 55.60], [13.004, 55.601]]    | Taxa C | 26            | Längsgående 6
[[12.994, 55.61], [12.995, 55.611]]    | Taxa A | 15            | Vinkel 45
```

**Taxa zones:**
- **Taxa A** — 30 kr/hour (city center)
- **Taxa B** — 25 kr/hour
- **Taxa C** — 20 kr/hour
- **Taxa D** — 15 kr/hour
- **Taxa E** — 10 kr/hour (outskirts)

## Database Format (Mobile Apps)

Mobile apps use transformed data with timestamps.

**Schema:**
```rust
struct DB {
    postnummer: Option<String>,
    adress: String,
    gata: Option<String>,
    gatunummer: Option<String>,
    info: Option<String>,              // Miljödata restriction
    start_time: DateTime<Utc>,         // UTC timestamp
    end_time: DateTime<Utc>,           // UTC timestamp
    taxa: Option<String>,              // Parking zone
    antal_platser: Option<u64>,        // Number of spots
    typ_av_parkering: Option<String>,  // Parking type
}
```

**Time handling:**
- Stored as UTC timestamps
- Displayed in Swedish timezone (Europe/Stockholm)
- Automatic DST handling via `chrono-tz`

**Conversion:**
```rust
let db = DB::from_dag_tid(
    postnummer,
    adress,
    gata,
    gatunummer,
    info,
    15,              // day of month
    "0800-1200",     // time range
    taxa,
    antal_platser,
    typ_av_parkering,
    2024,            // year
    1,               // month
)?;
```

See [Architecture](architecture.md) for `DB` struct details.

## Reading and Writing

### Writing Parquet

```rust
use amp_core::parquet;

// Write addresses
let addresses: Vec<AdressClean> = fetch_addresses()?;
parquet::write_addresses(&addresses, "addresses.parquet")?;

// Write miljödata
let miljo: Vec<MiljoeDataClean> = fetch_miljo()?;
parquet::write_miljo(&miljo, "miljo.parquet")?;

// Write parkering
let parkering: Vec<ParkeringsDataClean> = fetch_parkering()?;
parquet::write_parkering(&parkering, "parkering.parquet")?;
```

### Reading Parquet

```rust
use amp_core::parquet;

let addresses = parquet::read_addresses("addresses.parquet")?;
let miljo = parquet::read_miljo("miljo.parquet")?;
let parkering = parquet::read_parkering("parkering.parquet")?;
```

### Checksum Validation

```rust
use amp_core::checksum;

// Calculate checksum
let checksum = checksum::calculate_file("addresses.parquet")?;

// Validate against expected
let expected = "abc123...";
assert_eq!(checksum, expected);
```

Checksums stored in `server/checksums.json`.

## Data Pipeline

```
1. Fetch from API
   └─> GeoJSON format
       ├─> addresses.json (250 MB)
       ├─> miljo.json (80 MB)
       └─> parkering.json (60 MB)

2. Parse and Clean
   └─> Extract relevant fields
       └─> Convert coordinates to Decimal

3. Write Parquet
   └─> Compressed columnar format
       ├─> addresses.parquet (28 MB)
       ├─> miljo.parquet (9 MB)
       └─> parkering.parquet (7 MB)

4. Calculate Checksums
   └─> SHA-256 hashes
       └─> checksums.json

5. Bundle in Apps
   └─> Copy to assets/
       ├─> android/assets/
       └─> ios/assets/
```

## Coordinate System

**Format:** WGS84 (EPSG:4326)
- **Longitude:** X-axis, range [-180, 180]
- **Latitude:** Y-axis, range [-90, 90]

**Malmö bounds:**
- Longitude: [12.9, 13.1]
- Latitude: [55.5, 55.7]

**Precision:**
- Stored as `Decimal` (arbitrary precision)
- Converted to `f64` for calculations
- 6 decimal places ≈ 10cm accuracy

## Schema Evolution

Parquet supports schema evolution:

**Adding fields:**
- New fields default to `None`
- Old files remain readable

**Removing fields:**
- Old files ignore missing fields
- Use `Option<T>` for future compatibility

**Best practices:**
- Always use `Option<T>` for new fields
- Never rename fields (add new, deprecate old)
- Version schemas in file metadata

## Tools

### View Parquet Files

```bash
# Install parquet-tools
cargo install parquet-tools

# View schema
parquet-tools schema addresses.parquet

# View data
parquet-tools head addresses.parquet -n 10
```

### Convert to CSV

```bash
parquet-tools csv addresses.parquet > addresses.csv
```

### Inspect Metadata

```bash
parquet-tools meta addresses.parquet
```

## Related Documentation

- **[Architecture](architecture.md)** — System overview
- **[API Integration](api-integration.md)** — Data fetching
- **[Core Library](../core/README.md)** — Parquet API reference
