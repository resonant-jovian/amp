# CLI Usage

Command-line interface for correlation, testing, and benchmarking.

## Overview

The `amp_server` CLI tool provides:
- Address-to-zone correlation
- Visual testing with browser integration
- Algorithm benchmarking
- Data update monitoring
- Output generation for mobile apps

## Installation

```bash
# Build from source
cargo build --release -p amp_server

# Run directly
cargo run --release -p amp_server -- [COMMAND]

# Install to PATH
cargo install --path server
amp-server [COMMAND]
```

## Commands

### correlate

Run correlation with specified algorithm and display results.

**Usage:**
```bash
amp-server correlate [OPTIONS]
```

**Options:**
- `-a, --algorithm <ALGORITHM>` - Algorithm to use (default: kdtree)
  - `kdtree` - KD-Tree spatial index (recommended)
  - `rtree` - R-Tree spatial index
  - `grid` - Grid-based nearest neighbor
  - `overlapping-chunks` - Overlapping chunks
  - `distance-based` - Brute force
  - `raycasting` - Ray casting
- `-c, --cutoff <METERS>` - Distance cutoff in meters (default: 20)

**Examples:**
```bash
# Default: KD-Tree with 20m cutoff
amp-server correlate

# R-Tree with 100m cutoff
amp-server correlate --algorithm rtree --cutoff 100

# Grid algorithm
amp-server correlate -a grid -c 50
```

**Output:**
```
✅ Loaded 30,245 addresses, 1,234 miljödata zones, 456 parkering zones

📋 Dataset Information:
  Correlating with: Miljödata + Parkering (dual dataset)
  Distance threshold: 20 meters

🚀 Running correlation with KDTree algorithm
[████████████████████] 30245/30245 100%
✓ Completed in 1.23s

📊 Results:
  Total matches: 23,456 (77.6%)
  ├─ Both datasets: 18,234 (60.3%)
  ├─ Miljödata only: 3,456 (11.4%)
  ├─ Parkering only: 1,766 (5.8%)
  └─ No match: 6,789 (22.4%)
```

### output

Generate Parquet files for mobile apps or server database.

**Usage:**
```bash
amp-server output [OPTIONS]
```

**Options:**
- `-a, --algorithm <ALGORITHM>` - Algorithm (default: kdtree)
- `-c, --cutoff <METERS>` - Distance cutoff (default: 20)
- `-o, --output <PATH>` - Output file path (default: db.parquet)
- `--android` - Generate Android-formatted files with day/time extraction

**Examples:**
```bash
# Generate server database
amp-server output --output results.parquet

# Generate Android app data
amp-server output --android

# Custom algorithm and cutoff
amp-server output -a rtree -c 100 --android
```

**Output files (with `--android`):**
```
android/assets/data/
├── db.parquet           # Full correlation results
├── local.parquet        # Extracted day/time data
├── adress_info.parquet  # Address metadata
└── parking_db.parquet   # Parking zone info
```

### test

Visual testing mode - opens browser windows with correlation results overlaid on Malmö StadsAtlas.

**Usage:**
```bash
amp-server test [OPTIONS]
```

**Options:**
- `-a, --algorithm <ALGORITHM>` - Algorithm to test (default: kdtree)
- `-c, --cutoff <METERS>` - Distance cutoff (default: 20)
- `-w, --windows <NUM>` - Number of browser windows to open (default: 10)

**Examples:**
```bash
# Default: 10 windows with KD-Tree
amp-server test

# Test R-Tree with 15 windows
amp-server test --algorithm rtree --windows 15

# Test with 100m cutoff
amp-server test --cutoff 100
```

**How it works:**
1. Runs correlation on all addresses
2. Randomly samples addresses with matches
3. Opens browser windows with integrated tabs:
   - **Tab 1:** Address search with embedded StadsAtlas map
   - **Tab 2:** Step-by-step verification instructions
   - **Tab 3:** Correlation data visualization
   - **Tab 4:** Debug console with logs

**Browser selection (Linux):**
```bash
# Set preferred browser
export BROWSER=firefox
amp-server test

# Or use chromium
export BROWSER=chromium
amp-server test
```

See **[Testing Guide](testing.md)** for visual verification methodology.

### benchmark

Benchmark all algorithms and compare performance.

**Usage:**
```bash
amp-server benchmark [OPTIONS]
```

**Options:**
- `-s, --sample-size <NUM>` - Number of addresses to test (default: 100)
- `-c, --cutoff <METERS>` - Distance cutoff (default: 20)

**Examples:**
```bash
# Default: 100 addresses
amp-server benchmark

# Benchmark with 1000 addresses
amp-server benchmark --sample-size 1000

# Custom cutoff
amp-server benchmark -s 500 -c 50
```

**Interactive selection:**
```
🔧 Algorithm Selection (Y/N to include, default is Y):

  Include Distance-Based benchmark? [Y/n]: n
  ✗ Distance-Based skipped
  Include Raycasting benchmark? [Y/n]: y
  ✓ Raycasting selected
  Include Overlapping Chunks benchmark? [Y/n]: y
  ✓ Overlapping Chunks selected
  ...
```

**Output:**
```
🏁 Benchmarking 5 algorithm(s) with 1000 samples

○ [R-Tree            ] [████████████████████] 1000/1000 ✓ 1.15s
○ [KD-Tree           ] [████████████████████] 1000/1000 ✓ 1.08s
○ [Grid              ] [████████████████████] 1000/1000 ✓ 0.95s

📊 Benchmark Results (distance cutoff: 20m):

Algorithm            Total Time    Avg/Address    Matches
──────────────────────────────────────────────
Grid                 950ms         950μs          756
KD-Tree              1.08s         1.08ms         763
R-Tree               1.15s         1.15ms         763
```

### check-updates

Check for data updates from Malmö Open Data portal.

**Usage:**
```bash
amp-server check-updates [OPTIONS]
```

**Options:**
- `-c, --checksum-file <PATH>` - Checksum file path (default: checksums.json)

**Examples:**
```bash
# Check with default checksum file
amp-server check-updates

# Custom checksum file
amp-server check-updates --checksum-file ~/.amp-checksums.json
```

**Output:**
```
🔍 Checking for data updates...

○ Fetching remote data...
✓ Data fetched

✓ Data has changed!
  Old checksums from: 2026-01-15 14:23:10 UTC
  New checksums from: 2026-02-03 09:15:42 UTC
✓ Checksums saved to checksums.json
```

**Automation:**
```bash
# Add to cron for daily checks
0 2 * * * cd /path/to/amp/server && amp-server check-updates
```

## Global Options

**Available for all commands:**
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Data Sources

CLI fetches data from Malmö Open Data APIs:

1. **Miljöparkering** (Environmental Parking Zones)
   - URL: `opendata.malmo.se/@fastighets-och-gatukontoret/miljoparkering/...`
   - Format: GeoJSON with MultiLineString geometries

2. **Parkeringsavgifter** (Parking Fees)
   - URL: `opendata.malmo.se/@fastighets-och-gatukontoret/parkeringsavgifter/...`
   - Format: GeoJSON with LineString geometries

3. **Adresser** (Addresses)
   - URL: `opendata.malmo.se/@stadsbyggnadskontoret/adresser/...`
   - Format: GeoJSON with Point geometries

See **[API Integration](api-integration.md)** for details.

## Environment Variables

**`BROWSER`** (Linux only)
Specify browser for visual testing:
```bash
export BROWSER=firefox
```

**`RUST_LOG`**
Control logging verbosity:
```bash
export RUST_LOG=debug
amp-server correlate
```

## Exit Codes

- `0` - Success
- `1` - Error (network, file I/O, parsing, etc.)

## Performance Tips

**Use release builds:**
```bash
cargo run --release -p amp_server -- [COMMAND]
```

**Reduce sample size for quick tests:**
```bash
amp-server benchmark --sample-size 100
```

**Use KD-Tree for production:**
```bash
amp-server output --algorithm kdtree --android
```

## Troubleshooting

**"Failed to fetch data"**
- Check internet connection
- Verify Malmö Open Data portal is accessible
- Check API URLs in `core/src/api.rs`

**"No browser found" (Linux)**
```bash
export BROWSER=firefox
# or
sudo apt install firefox
```

**"Out of memory"**
- Reduce sample size in benchmark
- Use smaller dataset
- Increase system RAM

## See Also

- **[Testing Guide](testing.md)** - Visual testing methodology
- **[Algorithms](algorithms.md)** - Algorithm comparison
- **[API Integration](api-integration.md)** - Data fetching
- **[Server README](../server/README.md)** - Implementation details
