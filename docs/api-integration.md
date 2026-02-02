# API Integration

AMP fetches parking zone data from Malmö Stad's Open Data platform using ArcGIS REST APIs.

## Data Sources

### 1. Adresser (Addresses)

**Endpoint:**
```
https://kartor.malmo.se/arcgisserver/rest/services/Geoarbeten/GeoarbeteMalmo/MapServer/3/query
```

**Parameters:**
- `where`: `1=1` (fetch all)
- `outFields`: `*` (all attributes)
- `outSR`: `3006` (SWEREF99 TM)
- `f`: `geojson`

**Data Structure:**
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [115234.5, 6166789.2]
      },
      "properties": {
        "ADRESS": "Amiralsgatan 1",
        "POSTNR": "21139",
        "POSTORT": "Malmö",
        "KOMMUN": "Malmö"
      }
    }
  ]
}
```

**Parsed to:**
```rust
pub struct Address {
    pub adress: String,      // "Amiralsgatan 1"
    pub postnr: String,      // "21139"
    pub postort: String,     // "Malmö"
    pub x: Decimal,          // 115234.5
    pub y: Decimal,          // 6166789.2
}
```

### 2. Miljöparkering (Environmental Parking)

**Endpoint:**
```
https://kartor.malmo.se/arcgisserver/rest/services/TK/TK_Parkering_Extern/MapServer/5/query
```

**Data Structure:**
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "MultiLineString",
        "coordinates": [
          [[115200, 6166800], [115250, 6166850]]
        ]
      },
      "properties": {
        "GILTIGHET": "förbjudet 08-12 1,15,29",
        "STARTTID": "08:00",
        "SLUTTID": "12:00",
        "DATUM": "1,15,29"
      }
    }
  ]
}
```

**Parsed to:**
```rust
pub struct MiljoParkering {
    pub giltighet: String,         // "förbjudet 08-12 1,15,29"
    pub segments: Vec<Segment>,    // Line segments from MultiLineString
}

pub struct Segment {
    pub start: (Decimal, Decimal),
    pub end: (Decimal, Decimal),
}
```

### 3. Parkeringsavgifter (Parking Fees)

**Endpoint:**
```
https://kartor.malmo.se/arcgisserver/rest/services/TK/TK_Parkering_Extern/MapServer/0/query
```

**Contains:**
- Fee zones (Taxa A-E)
- Pricing information
- Operating hours

**Currently unused** in mobile app (future feature).

## Fetching Data

### From CLI

```bash
# Fetch and correlate
cargo run --release -p amp_server -- correlate

# Check for updates
cargo run --release -p amp_server -- check-updates
```

### From Code

```rust
use amp_core::api::api_miljo_only;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch addresses and zones
    let (addresses, zones) = api_miljo_only()?;
    
    println!("Fetched {} addresses", addresses.len());
    println!("Fetched {} zones", zones.len());
    
    Ok(())
}
```

## Data Validation

### Checksum Verification

AMP stores SHA-256 checksums to detect data updates:

```json
{
  "adresser": "a1b2c3d4e5f6...",
  "miljoparkering": "1a2b3c4d5e6f...",
  "parkeringsavgifter": "9z8y7x6w5v..."
}
```

**Location:** `server/checksums.json`

**Validation:**
```bash
cargo run -- check-updates
```

### Error Handling

The API module handles:
- **Network failures**: Retry with exponential backoff
- **Parse errors**: Log and skip malformed features
- **Missing fields**: Use defaults or skip entry
- **Invalid coordinates**: Filter out-of-bounds points

## Coordinate System

### SWEREF99 TM (EPSG:3006)

**Projection:** Transverse Mercator  
**Units:** Meters  
**Coverage:** Sweden

**Malmö Bounds:**
- X (Easting): 115,000 - 120,000
- Y (Northing): 6,165,000 - 6,170,000

**Why SWEREF99:**
- Official Swedish coordinate system
- Meters simplify distance calculations
- Direct from Malmö Open Data (no conversion needed)

### WGS84 Conversion

Mobile apps use GPS (WGS84/EPSG:4326):

```rust
use geodesy::prelude::*;

// WGS84 to SWEREF99 TM
let proj = Proj::new("EPSG:4326", "EPSG:3006")?;
let (x, y) = proj.forward((lon, lat))?;
```

## Data Update Frequency

Malmö Stad updates datasets:
- **Adresser**: Monthly (new buildings)
- **Miljöparkering**: Quarterly (zone changes)
- **Parkeringsavgifter**: Annually (pricing updates)

Recommend checking for updates monthly:

```bash
# Cron job example
0 0 1 * * cd /path/to/amp && cargo run -- check-updates
```

## Rate Limiting

Malmö Open Data has no explicit rate limits, but:
- **Be respectful**: Don't hammer the API
- **Cache locally**: Use checksums to avoid redundant fetches
- **Batch requests**: Fetch all data at once

## Offline Mode

Mobile apps work offline using embedded Parquet files:

1. **Development**: Fetch data via API
2. **Build**: Convert to Parquet (`amp_core::parquet`)
3. **Embed**: Include in app assets
4. **Runtime**: Load from Parquet (no network)

See [Data Format](data-format.md) for Parquet details.

## Related Documentation

- [Architecture](architecture.md) — How API fits in system
- [Data Format](data-format.md) — Parquet structure
- [CLI Usage](cli-usage.md) — Running data fetches
