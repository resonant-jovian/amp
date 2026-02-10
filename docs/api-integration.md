# API Integration

AMP fetches geospatial data from Malmö Open Data API.

## Data Sources

All datasets provided by [Malmö Open Data](https://malmo.se/opendata).

### 1. Addresses

**Endpoint:** `https://opendata.malmo.se/api/3/action/datastore_search`

**Dataset ID:** `d41fb5ea-a7bc-4e00-b31a-4c799e05de11`

**Description:** Complete address database for Malmö with coordinates.

**Fields used:**
- `POINT_X` — Longitude (WGS84)
- `POINT_Y` — Latitude (WGS84)
- `POSTNUMMER` — Postal code
- `ADRESS` — Full address
- `GATA` — Street name
- `GATUNUMMER` — Street number

### 2. Miljöparkeringar (Environmental Parking)

**Endpoint:** GeoJSON API

**URL:** `https://geodata.malmo.se/datasets/malmo-stad::miljoparkeringar.geojson`

**Description:** Time-restricted parking zones for street cleaning.

**Fields used:**
- `geometry` — LineString coordinates
- `INFO` — Restriction description
- `TID` — Time range ("HHMM-HHMM")
- `DAG` — Day of month (1-31)

### 3. Parkeringsavgifter (Parking Fees)

**Endpoint:** GeoJSON API

**URL:** `https://geodata.malmo.se/datasets/malmo-stad::parkeringsavgifter.geojson`

**Description:** Paid parking zones with taxa information.

**Fields used:**
- `geometry` — LineString coordinates
- `TAXA` — Zone identifier (A-E)
- `ANTAL_PLATSER` — Number of parking spots
- `TYP_AV_PARKERING` — Type (e.g., "Längsgående 6")

## Implementation

### API Client

**Location:** `core/src/api.rs`

```rust
use amp_core::api;

// Fetch all datasets
let addresses = api::fetch_addresses().await?;
let miljo = api::fetch_miljo_zones().await?;
let parkering = api::fetch_parkering_zones().await?;
```

### Error Handling

```rust
match api::fetch_addresses().await {
    Ok(addresses) => {
        println!("Fetched {} addresses", addresses.len());
    }
    Err(e) => {
        eprintln!("Failed to fetch addresses: {}", e);
    }
}
```

### Rate Limiting

Malmö Open Data API has no documented rate limits, but use respectful practices:

- **Cache data locally** (Parquet files)
- **Batch requests** when possible
- **Use exponential backoff** on errors

### Retry Logic

```rust
use tokio::time::{sleep, Duration};

let mut retries = 0;
let max_retries = 3;

loop {
    match api::fetch_addresses().await {
        Ok(data) => break Ok(data),
        Err(e) if retries < max_retries => {
            retries += 1;
            let delay = Duration::from_secs(2u64.pow(retries));
            eprintln!("Retry {}/{}: {:?}", retries, max_retries, e);
            sleep(delay).await;
        }
        Err(e) => break Err(e),
    }
}
```

## Data Processing

### Parsing GeoJSON

```rust
use geojson::{GeoJson, Feature};

let geojson_str = fetch_raw_geojson(url).await?;
let geojson = geojson_str.parse::<GeoJson>()?;

if let GeoJson::FeatureCollection(collection) = geojson {
    for feature in collection.features {
        // Extract properties
        let properties = feature.properties.unwrap();
        let info = properties.get("INFO")?.as_str()?;
        
        // Extract geometry
        if let Some(geom) = feature.geometry {
            // Process LineString coordinates
        }
    }
}
```

### Coordinate Conversion

```rust
use rust_decimal::Decimal;

// GeoJSON uses f64, convert to Decimal for precision
let lon_f64: f64 = coords[0];
let lat_f64: f64 = coords[1];

let lon = Decimal::from_f64_retain(lon_f64).unwrap();
let lat = Decimal::from_f64_retain(lat_f64).unwrap();

let coordinates = [lon, lat];
```

## Data Updates

### Update Workflow

```bash
# 1. Fetch latest data from API
cargo run --release -p amp_server -- fetch

# 2. Verify checksums changed
diff server/checksums.json.old server/checksums.json

# 3. Test with new data
cargo run --release -- test

# 4. Update mobile app assets
cp server/*.parquet android/assets/
cp server/*.parquet ios/assets/

# 5. Rebuild apps
./scripts/build.sh
```

### Update Frequency

**Recommended:** Monthly

**Reasons:**
- Street cleaning schedules change seasonally
- New addresses added regularly
- Parking zones occasionally updated

### Checksum Validation

```rust
use amp_core::checksum;

// Calculate checksum for new file
let new_checksum = checksum::calculate_file("addresses.parquet")?;

// Load expected checksums
let checksums = checksum::load_checksums("checksums.json")?;

// Compare
if new_checksum != checksums.addresses {
    println!("⚠️  Data changed! Update required.");
}
```

Checksums stored in `server/checksums.json`.

## API Response Examples

### Addresses Response

```json
{
  "result": {
    "records": [
      {
        "POINT_X": "13.003456",
        "POINT_Y": "55.604523",
        "POSTNUMMER": "21438",
        "ADRESS": "Åhusgatan 1",
        "GATA": "Åhusgatan",
        "GATUNUMMER": "1"
      }
    ]
  }
}
```

### Miljöparkeringar GeoJSON

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "LineString",
        "coordinates": [
          [13.003456, 55.604523],
          [13.004567, 55.605634]
        ]
      },
      "properties": {
        "INFO": "Parkering förbjuden",
        "TID": "0800-1200",
        "DAG": 15
      }
    }
  ]
}
```

### Parkeringsavgifter GeoJSON

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "LineString",
        "coordinates": [
          [13.003456, 55.604523],
          [13.004567, 55.605634]
        ]
      },
      "properties": {
        "TAXA": "Taxa C",
        "ANTAL_PLATSER": 26,
        "TYP_AV_PARKERING": "Längsgående 6"
      }
    }
  ]
}
```

## Related Documentation

- **[Data Format](data-format.md)** — Parquet storage details
- **[Architecture](architecture.md)** — System overview
- **[Core Library](../core/README.md)** — API client reference
