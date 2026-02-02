# Data Format

AMP uses Apache Parquet for efficient offline data storage in mobile apps.

## Why Parquet?

**Advantages:**
- **Columnar format**: Fast queries on specific fields
- **Compression**: ~10x smaller than JSON
- **Schema evolution**: Add fields without breaking old apps
- **Fast loading**: Faster than JSON parsing
- **Cross-platform**: Works on Android, iOS, CLI

**Comparison:**

| Format | Size | Load Time | Query Speed |
|--------|------|-----------|-------------|
| JSON   | 12 MB | 450ms | Slow (full scan) |
| Parquet | 1.2 MB | 80ms | Fast (columnar) |
| SQLite | 3.5 MB | 120ms | Medium (indexed) |

## File Structure

### addresses.parquet

**Schema:**
```
address: string (nullable)
postal_code: string (nullable)
postal_area: string (nullable)
x: double (nullable)
y: double (nullable)
```

**Example Row:**
```json
{
  "address": "Amiralsgatan 1",
  "postal_code": "21139",
  "postal_area": "Malmö",
  "x": 115234.56,
  "y": 6166789.12
}
```

**Usage:**
```rust
use amp_core::parquet::read_addresses;

let addresses = read_addresses("addresses.parquet")?;
for addr in addresses {
    println!("{}: ({}, {})", addr.adress, addr.x, addr.y);
}
```

### zones.parquet

**Schema:**
```
zone_id: int32 (nullable)
restriction: string (nullable)
segments: list<struct<start_x: double, start_y: double, end_x: double, end_y: double>>
```

**Example Row:**
```json
{
  "zone_id": 42,
  "restriction": "förbjudet 08-12 1,15,29",
  "segments": [
    {"start_x": 115200.0, "start_y": 6166800.0, "end_x": 115250.0, "end_y": 6166850.0},
    {"start_x": 115250.0, "start_y": 6166850.0, "end_x": 115300.0, "end_y": 6166900.0}
  ]
}
```

**Usage:**
```rust
use amp_core::parquet::read_zones;

let zones = read_zones("zones.parquet")?;
for zone in zones {
    println!("Zone {}: {} segments", zone.id, zone.segments.len());
}
```

## Conversion Pipeline

### GeoJSON to Parquet

1. **Fetch GeoJSON** from Malmö Open Data APIs
2. **Parse** into Rust structs (`Address`, `MiljoParkering`)
3. **Convert** to Arrow schema
4. **Write** Parquet with compression

**Code:**
```rust
use amp_core::parquet::write_addresses_parquet;

let addresses = fetch_addresses_from_api()?;
write_addresses_parquet(&addresses, "addresses.parquet")?;
```

### Build Integration

Mobile apps include Parquet files in assets:

**Android:**
```toml
# android/Dioxus.toml
[assets]
include = ["assets/addresses.parquet", "assets/zones.parquet"]
```

**iOS:**
```toml
# ios/Dioxus.toml
[assets]
include = ["assets/addresses.parquet", "assets/zones.parquet"]
```

See [Android README](../android/README.md) for build process.

## Loading in Mobile Apps

### Android

```rust
use amp_core::parquet::read_addresses;

pub fn load_static_data() -> Result<(Vec<Address>, Vec<MiljoParkering>)> {
    let addresses = read_addresses("assets://addresses.parquet")?;
    let zones = read_zones("assets://zones.parquet")?;
    Ok((addresses, zones))
}
```

### iOS

```rust
use amp_core::parquet::read_addresses;

pub fn load_static_data() -> Result<(Vec<Address>, Vec<MiljoParkering>)> {
    let addresses = read_addresses("assets://addresses.parquet")?;
    let zones = read_zones("assets://zones.parquet")?;
    Ok((addresses, zones))
}
```

## Compression

Parquet supports multiple compression codecs:

| Codec | Ratio | Speed | Mobile Support |
|-------|-------|-------|----------------|
| Snappy | 3-5x | Fast | ✓ |
| Gzip | 8-12x | Medium | ✓ |
| Zstd | 10-15x | Fast | ✓ |
| LZ4 | 2-4x | Very Fast | ✓ |

**AMP uses Snappy** (default):
- Good compression (5x typical)
- Fast decompression (critical for mobile)
- Universal support

## Schema Evolution

### Adding Fields

Parquet supports backward-compatible schema changes:

**Before:**
```
address: string
postal_code: string
x: double
y: double
```

**After:**
```
address: string
postal_code: string
postal_area: string  // NEW FIELD
x: double
y: double
altitude: double     // NEW FIELD (nullable)
```

Old apps ignore new fields. New apps handle missing fields as `null`.

### Breaking Changes

Avoid:
- Removing fields
- Changing field types
- Renaming fields

If necessary, version the files: `addresses_v2.parquet`

## File Locations

### Mobile Apps

```
android/assets/
├── addresses.parquet
└── zones.parquet

ios/assets/
├── addresses.parquet
└── zones.parquet
```

### CLI (Generated)

```
server/
├── addresses.parquet      # Generated from API
└── zones.parquet           # Generated from API
```

CLI tools regenerate these on each run (not committed to git).

## Debugging Parquet Files

### View Schema

```bash
# Using parquet-tools (Python)
pip install parquet-tools
parquet-tools schema addresses.parquet
```

### Inspect Data

```bash
parquet-tools show addresses.parquet --head 10
```

### Convert to JSON

```bash
parquet-tools json addresses.parquet > addresses.json
```

## Performance Tips

1. **Lazy Loading**: Load only needed columns
   ```rust
   // Load only addresses, skip coordinates
   let addrs = read_addresses_names_only("addresses.parquet")?;
   ```

2. **Predicate Pushdown**: Filter during read
   ```rust
   // Load only Malmö addresses
   let addrs = read_addresses_filtered("addresses.parquet", |a| {
       a.postal_area == "Malmö"
   })?;
   ```

3. **Memory Mapping**: Use `mmap` for large files
   ```rust
   let file = File::open("addresses.parquet")?;
   let mmap = unsafe { MmapOptions::new().map(&file)? };
   let reader = SerializedFileReader::new(Bytes::from(mmap.to_vec()))?;
   ```

## Related Documentation

- [API Integration](api-integration.md) — Where data comes from
- [Architecture](architecture.md) — How Parquet fits in system
- [Android README](../android/README.md) — Mobile app integration
