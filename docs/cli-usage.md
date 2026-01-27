# CLI Usage

The `amp-server` CLI provides correlation, testing, benchmarking, and data verification commands.

## Installation

```bash
cargo install --path server
# Or
cargo build --release -p amp_server
./target/release/amp-server --help
```

## Commands

### test

Visually verify correlation accuracy by opening browser windows to StadsAtlas alongside correlation results.

```bash
amp-server test [OPTIONS]
```

**Options:**
- `-a, --algorithm <NAME>` — Algorithm to use (default: kdtree)
  - `distance-based` — Brute-force O(n×m)
  - `raycasting` — Geometric raycasting
  - `overlapping-chunks` — Spatial grid
  - `rtree` — R-tree spatial index
  - `kdtree` — KD-tree spatial index (default)
  - `grid` — Fixed-size grid
- `-c, --cutoff <DISTANCE>` — Distance threshold in meters (default: 50)
- `-w, --windows <COUNT>` — Number of browser windows to open (default: 10)

**What Each Window Shows:**
- **Tab 1:** Official Malmö StadsAtlas map at https://stadsatlas.malmo.se/
  - Blue pin marker shows address location
  - Manually enable "Miljöparkering" checkbox to see parking zones
- **Tab 2:** Correlation result details
  - Address, postal code
  - Data source (Miljödata, Parkering, or both)
  - Distance to matched zone
  - Zone information

**Examples:**

```bash
# Default: 10 windows, KD-Tree, 50m threshold
amp-server test

# Quick test: 5 windows
amp-server test --windows 5

# Compare algorithms
amp-server test --algorithm kdtree --windows 10
amp-server test --algorithm rtree --windows 10

# Validate distance thresholds
amp-server test --cutoff 25 --windows 5
amp-server test --cutoff 50 --windows 5
amp-server test --cutoff 100 --windows 5

# Large-scale test
amp-server test --algorithm kdtree --cutoff 50 --windows 50
```

**Common Use Cases:**

**Test Algorithm Performance**
```bash
# Compare KD-Tree vs R-Tree on same data
amp-server test --algorithm kdtree --windows 10
amp-server test --algorithm rtree --windows 10
# Manually compare accuracy in both sets of windows
```

**Validate Distance Threshold**
```bash
# Conservative threshold (25m) - fewer but more accurate matches
amp-server test --cutoff 25 --windows 5

# Standard threshold (50m)
amp-server test --cutoff 50 --windows 5

# Permissive threshold (100m) - more matches but may include false positives
amp-server test --cutoff 100 --windows 5

# Compare accuracy vs. match count
```

**Test Specific Data Quality**
```bash
# Random sample
amp-server test --windows 20

# If accuracy is low, try different algorithm
amp-server test --algorithm overlapping-chunks --windows 20

# If still low, increase cutoff
amp-server test --algorithm overlapping-chunks --cutoff 100 --windows 20
```

**Interpreting Results:**

✅ **Good Correlation**
- StadsAtlas zone matches Tab 2 information
- Distance shown (e.g., "15.3m away") seems reasonable
- Zone name and regulations align with address

⚠️ **Poor Correlation**
- StadsAtlas shows different zone
- Distance at or very close to cutoff (e.g., "49.8m away")
- Zone information doesn't match visible features

❌ **No Match**
- Tab 2 shows "No matches found"
- Address outside all zones or beyond cutoff
- Try: `--cutoff 100` for larger search radius

**Troubleshooting:**

❌ "No matching addresses found for testing!"
- Cause: Correlation found no addresses within cutoff distance
- Solution: Increase cutoff (`--cutoff 100`) or try different algorithm

❌ Windows Not Opening
- Windows: Ensure default browser is configured
- macOS: Grant terminal permission to control applications
- Linux: Ensure `xdg-open` is installed

❌ StadsAtlas Search Not Working
1. Verify you're in correct region (Malmö)
2. Try clicking location icon first
3. Zoom map to Malmö area
4. Try entering just street name without number

❌ Data Tab Not Showing
1. Browser may have blocked data URL
2. Try refreshing the page
3. Check browser console (F12 → Console)
4. Try different browser

**Performance Notes:**
- Window opening: ~500ms delay between each (system stability)
- 10 windows: ~5 seconds to fully open
- 20 windows: ~10 seconds to fully open
- Correlation runtime: 2-8 seconds depending on algorithm

---

### correlate

Run address-to-zone correlation with specified algorithm.

```bash
amp-server correlate [OPTIONS]
```

**Options:**
- `-a, --algorithm <NAME>` — Algorithm to use (default: rtree)
  - `distance-based` — Brute-force O(n×m)
  - `raycasting` — 36-ray search
  - `overlapping-chunks` — Spatial grid with overlap
  - `rtree` — R-tree spatial index
  - `kdtree` — KD-tree spatial index
  - `grid` — Fixed-size grid
- `-c, --cutoff <DISTANCE>` — Distance threshold in meters (default: 50)

**Example:**

```bash
$ amp-server correlate --algorithm rtree

📋 Dataset Information:
   Addresses: 100,342
   Miljödata zones: 1,847
   Parkering zones: 3,256
   Max distance threshold: 50 meters

🚀 Running correlation with RTree algorithm
[████████████████████] 100342/100342 100% ✓ Completed in 2.31s

📊 Results:
   Addresses processed: 100,342
   Total matches: 87,234 (86.9%)
   ├─ Both datasets: 12,456 (12.4%)
   ├─ Miljödata only: 34,567 (34.4%)
   ├─ Parkering only: 40,211 (40.1%)
   └─ No match: 13,108 (13.1%)
   Average time per address: 23.02µs
```

**Output:**
- Match statistics by dataset
- Random sample of 10 matches
- Top 10 largest distances (threshold verification)

---

### benchmark

Compare performance of all six algorithms.

```bash
amp-server benchmark [OPTIONS]
```

**Options:**
- `-s, --sample-size <N>` — Number of addresses to test (default: 100)
- `-c, --cutoff <DISTANCE>` — Distance threshold in meters (default: 50)

**Example:**

```bash
$ amp-server benchmark --sample-size 500

🏁 Benchmarking all 6 algorithms with 500 samples

[Distance-Based    ] [██████████] 500/500 ✓ 2.45s
[Raycasting        ] [██████████] 500/500 ✓ 5.12s
[Overlapping Chunks] [██████████] 500/500 ✓ 1.23s
[R-Tree            ] [██████████] 500/500 ✓ 1.15s
[KD-Tree           ] [██████████] 500/500 ✓ 1.28s
[Grid              ] [██████████] 500/500 ✓ 1.31s

📊 Benchmark Results:

Algorithm            Total Time    Avg/Address    Matches
──────────────────────────────────────────────────────
Distance-Based       2.45s         4.90ms         423
Raycasting          5.12s         10.24ms        431
Overlapping Chunks  1.23s         2.46ms         423
R-Tree              1.15s         2.30ms         423
KD-Tree             1.28s         2.56ms         423
Grid                1.31s         2.62ms         423

✓ Fastest: R-Tree (1.15s)
```

---

### check-updates

Verify if Malmö's open data has changed.

```bash
amp-server check-updates [OPTIONS]
```

**Options:**
- `-c, --checksum-file <PATH>` — Checksum file (default: checksums.json)

**Example:**

```bash
$ amp-server check-updates

🔍 Checking for data updates...

✓ Data fetched

✓ Data has changed!
  Old checksums from: 2026-01-22T10:15:30Z
  New checksums from: 2026-01-23T10:15:30Z
✓ Checksums saved to checksums.json
```

**Checksum File Format:**

```json
{
  "miljoparkering": "a3f5e8...",
  "parkeringsavgifter": "b2d9c1...",
  "adresser": "f7e4a2...",
  "last_checked": "2026-01-23T10:15:30Z"
}
```

**Use Cases:**
- Daily cron job to monitor data changes
- CI/CD pipeline validation
- Manual verification before deployment

---

## Common Workflows

### Quick Visual Test

```bash
# Open 10 windows for manual verification
amp-server test
```

### Algorithm Comparison

```bash
# Small sample for quick comparison
amp-server benchmark --sample-size 100

# Large sample for accurate results
amp-server benchmark --sample-size 5000
```

### Production Deployment

```bash
# 1. Check for data updates
amp-server check-updates

# 2. Run correlation with best algorithm
amp-server correlate --algorithm rtree

# 3. Verify results (examine output statistics)
```

### Daily Monitoring

```bash
#!/bin/bash
# daily-check.sh

if amp-server check-updates; then
    echo "Data updated, re-running correlation"
    amp-server correlate --algorithm rtree
fi
```

### Testing Best Practices

```bash
# 1. Start with default settings
amp-server test

# 2. Review results and document findings

# 3. Test incrementally with more windows
amp-server test --windows 20

# 4. Compare algorithms on same data
amp-server test --algorithm kdtree --windows 10
amp-server test --algorithm rtree --windows 10

# 5. Adjust cutoff based on results
amp-server test --cutoff 25 --windows 5
amp-server test --cutoff 75 --windows 5

# 6. Check data freshness if needed
amp-server check-updates
```

---

## Environment Variables

None required. All data fetched from public Malmö Open Data Portal.

## Output Files

- `checksums.json` — Data verification checksums
- stdout — Correlation/test results (pipe to file if needed)

## Performance Tips

**For large datasets:**
```bash
# Use R-Tree or KD-Tree (best performance/stability)
amp-server correlate --algorithm rtree
```

**For benchmarking:**
```bash
# Start with small sample
amp-server benchmark --sample-size 100

# Increase for production validation
amp-server benchmark --sample-size 1000
```

**For CI/CD:**
```bash
# Quick validation
amp-server benchmark --sample-size 50
amp-server test --windows 5
```

---

## Troubleshooting

**"No matches found"**
- Check internet connection (ArcGIS API requires network)
- Verify Malmö Open Data Portal is accessible
- Try `check-updates` to confirm data availability

**"Slow performance"**
- Use `--algorithm rtree` or `kdtree` instead of `distance-based`
- Reduce `--sample-size` for benchmarks
- Consider memory constraints (Grid/Chunks use more RAM)

**"Checksum file not found"**
- Normal on first run
- File created automatically by `check-updates`

**"Windows not opening during test"**
- Verify browser is installed and configured as default
- Check if browser is blocked by firewall
- Try manually opening StadsAtlas in browser

---

## Related Documentation

- [Testing Guide](testing.md) — Detailed testing procedures
- [Algorithms](algorithms.md) — Algorithm details
- [Architecture](architecture.md) — System design
- [server/README.md](../server/README.md) — Server module guide
