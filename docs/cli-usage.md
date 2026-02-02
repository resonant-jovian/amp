# CLI Usage

The `amp_server` CLI tool provides commands for testing, correlation, benchmarking, and data validation.

## Installation

```bash
# Build release binary
cargo build --release -p amp_server

# Binary location
./target/release/amp-server
```

## Commands

### test — Visual Testing

Open browser tabs to manually verify correlation accuracy against official Malmö StadsAtlas.

```bash
# Default: 10 windows, KD-Tree, 50m threshold
cargo run --release -p amp_server -- test

# Custom parameters
cargo run -- test --algorithm rtree --cutoff 100 --windows 15
```

**Parameters:**
- `--algorithm <algo>`: Algorithm to test (kdtree, rtree, distance, grid, raycasting, chunks)
- `--cutoff <meters>`: Distance threshold (default: 50)
- `--windows <n>`: Number of browser tabs to open (default: 10)

**What each tab shows:**
- **Tab 1**: Official Malmö StadsAtlas with parking zones
- **Tab 2**: Correlation result (address, distance, zone details)

**Verification process:**
1. Look at official map (Tab 1)
2. Check if computed zone (Tab 2) matches visual inspection
3. Repeat for all opened tabs

See [Testing Guide](testing.md) for detailed workflow.

### correlate — Batch Correlation

Process all addresses and output correlation results.

```bash
# Default: KD-Tree, 50m threshold
cargo run --release -p amp_server -- correlate

# Custom algorithm
cargo run -- correlate --algorithm rtree

# Custom threshold
cargo run -- correlate --cutoff 75

# JSON output
cargo run -- correlate --output results.json
```

**Parameters:**
- `--algorithm <algo>`: Correlation algorithm
- `--cutoff <meters>`: Distance threshold
- `--output <file>`: Save results to JSON file

**Output format:**
```json
{
  "algorithm": "kdtree",
  "cutoff_meters": 50,
  "total_addresses": 45230,
  "matched": 42156,
  "unmatched": 3074,
  "results": [
    {
      "address": "Amiralsgatan 1",
      "zone_id": 156,
      "distance_meters": 12.5,
      "restriction": "förbjudet 08-12 1,15,29"
    }
  ]
}
```

### benchmark — Algorithm Comparison

Interactive benchmarking of all correlation algorithms.

```bash
# Default: 1000 random addresses
cargo run --release -p amp_server -- benchmark

# Custom sample size
cargo run -- benchmark --sample-size 500
```

**Output:**
```
Algorithm Benchmarks (sample size: 1000)
=========================================

KD-Tree Spatial
  Build time:    245ms
  Query time:    0.012ms/addr
  Total time:    257ms
  Matches:       934/1000

R-Tree Spatial
  Build time:    289ms
  Query time:    0.015ms/addr
  Total time:    304ms
  Matches:       932/1000

Distance-Based
  Build time:    <1ms
  Query time:    2.1ms/addr
  Total time:    2100ms
  Matches:       934/1000
```

**Parameters:**
- `--sample-size <n>`: Number of addresses to test (default: 1000)
- `--cutoff <meters>`: Distance threshold (default: 50)

### check-updates — Data Validation

Check if Malmö Open Data has been updated since last fetch.

```bash
cargo run --release -p amp_server -- check-updates
```

**Output:**
```
Checking data freshness...

Adresser:        ✓ Current (checksum match)
Miljöparkering:  ⚠ Updated (new data available)
Parkeringsavgifter: ✓ Current (checksum match)

Run 'cargo run -- correlate' to fetch latest data.
```

**How it works:**
1. Fetches metadata from Malmö Open Data APIs
2. Compares SHA-256 checksums
3. Reports which datasets have changed

Checksum stored in `server/checksums.json`.

## Global Options

These options work with any command:

```bash
# Verbose logging
cargo run -- test --verbose

# Quiet mode (errors only)
cargo run -- correlate --quiet

# Help
cargo run -- --help
cargo run -- test --help
```

## Examples

### Compare algorithms visually

```bash
# Test KD-Tree
cargo run -- test --algorithm kdtree --windows 10

# Test R-Tree with same addresses
cargo run -- test --algorithm rtree --windows 10

# Compare results manually
```

### Find optimal threshold

```bash
# Too strict (few matches)
cargo run -- correlate --cutoff 25 | grep "matched:"

# Balanced
cargo run -- correlate --cutoff 50 | grep "matched:"

# Permissive (many matches)
cargo run -- correlate --cutoff 100 | grep "matched:"
```

### Performance testing

```bash
# Quick test
cargo run --release -- benchmark --sample-size 100

# Thorough test
cargo run --release -- benchmark --sample-size 5000
```

## Troubleshooting

### "Failed to fetch data"

**Cause:** Network error or Malmö API down  
**Solution:** Check internet connection, try again later

### "No correlation found"

**Cause:** Distance threshold too strict  
**Solution:** Increase `--cutoff` value

### "Browser windows not opening"

**Cause:** Missing default browser or permissions  
**Solution:** Manually open URLs from console output

### "Checksum validation failed"

**Cause:** Corrupted data download  
**Solution:** Delete cached files and re-run

## Related Documentation

- [Testing Guide](testing.md) — Visual testing workflow
- [Algorithms](algorithms.md) — Algorithm details
- [Architecture](architecture.md) — CLI design
