# Server (CLI Tool)

Command-line interface for address-to-zone correlation, benchmarking, and data verification.

## Overview

The `amp-server` binary provides:
- Correlation with algorithm selection
- Performance benchmarking
- Data update checking
- Progress bars and formatted output

## Installation

```bash
# From workspace root
cargo install --path server

# Or build without installing
cargo build --release -p amp_server
./target/release/amp-server --help
```

## Commands

### correlate

Run address-to-zone correlation.

```bash
amp-server correlate --algorithm rtree
```

**Options:**
- `-a, --algorithm <NAME>` — Algorithm (default: rtree)
  - `distance-based`, `raycasting`, `overlapping-chunks`
  - `rtree`, `kdtree`, `grid`

**Output:**
```
📋 Dataset Information:
   Addresses: 100,342
   Miljödata zones: 1,847
   Parkering zones: 3,256

🚀 Running correlation with RTree algorithm
[██████████] 100342/100342 ✓ Completed in 2.31s

📊 Results:
   Total matches: 87,234 (86.9%)
   Both datasets: 12,456 (12.4%)
   Miljödata only: 34,567 (34.4%)
   Parkering only: 40,211 (40.1%)
```

See: [../docs/cli-usage.md](../docs/cli-usage.md)

### benchmark

Compare all algorithms.

```bash
amp-server benchmark --sample-size 500
```

**Options:**
- `-s, --sample-size <N>` — Number of addresses (default: 100)

**Output:**
```
🏁 Benchmarking all 6 algorithms with 500 samples

Algorithm            Total Time    Avg/Address    Matches
──────────────────────────────────────────────────────
R-Tree              1.15s         2.30ms         423
Overlapping Chunks  1.23s         2.46ms         423
KD-Tree             1.28s         2.56ms         423
Grid                1.31s         2.62ms         423
Distance-Based      2.45s         4.90ms         423
Raycasting          5.12s         10.24ms        431

✓ Fastest: R-Tree (1.15s)
```

### check-updates

Verify if Malmö data changed.

```bash
amp-server check-updates --checksum-file checksums.json
```

**Options:**
- `-c, --checksum-file <PATH>` — File path (default: checksums.json)

**Output:**
```
🔍 Checking for data updates...

✓ Data has changed!
  Old: 2026-01-22T10:15:30Z
  New: 2026-01-23T10:15:30Z
✓ Checksums saved to checksums.json
```

## Implementation

**Structure:**
```
server/src/
└── main.rs    # CLI implementation with clap
```

**Dependencies:**
- `amp_core` — Core library
- `clap` — CLI argument parsing
- `indicatif` — Progress bars
- `rayon` — Parallel processing
- `tokio` — Async runtime

**Key Functions:**

```rust
fn run_correlation(algorithm: AlgorithmChoice) -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let (addresses, miljo, parkering) = api()?;
    
    // Setup progress bar
    let pb = ProgressBar::new(addresses.len() as u64);
    
    // Correlate with both datasets
    let miljo_results = correlate_dataset(&algorithm, &addresses, &miljo, &pb)?;
    let parkering_results = correlate_dataset(&algorithm, &addresses, &parkering, &pb)?;
    
    // Merge and display results
    let merged = merge_results(&addresses, &miljo_results, &parkering_results);
    print_statistics(&merged);
    
    Ok(())
}
```

## Usage Examples

### Quick Test

```bash
# Fast correlation with best algorithm
amp-server correlate --algorithm rtree
```

### Algorithm Comparison

```bash
# Small sample
amp-server benchmark --sample-size 100

# Production validation
amp-server benchmark --sample-size 5000
```

### Production Deployment

```bash
#!/bin/bash
# deploy.sh

# Check for updates
if amp-server check-updates; then
    echo "Data updated"
    
    # Re-run correlation
    amp-server correlate --algorithm rtree > results.txt
    
    # Verify threshold
    if grep -q "All matches are within 50m" results.txt; then
        echo "Validation passed"
    else
        echo "ERROR: Threshold violation"
        exit 1
    fi
fi
```

### Daily Monitoring

```bash
# crontab -e
0 2 * * * /usr/local/bin/amp-server check-updates >> /var/log/amp-updates.log
```

## Output Format

### Statistics

```
📊 Results:
   Addresses processed: 100,342
   Total matches: 87,234 (86.9%)
   ├─ Both datasets: 12,456 (12.4%)
   ├─ Miljödata only: 34,567 (34.4%)
   ├─ Parkering only: 40,211 (40.1%)
   └─ No match: 13,108 (13.1%)
   Average time per address: 23.02µs
```

### Random Sample

```
🎲 10 Random Matches:
   Stortorget 1 (Both datasets)
      ├─ Miljödata: 23.45m
      └─ Parkering: 18.72m
   Amiralsgatan 15 (Miljödata only)
      └─ Miljödata: 12.34m
```

### Threshold Verification

```
📏 10 Addresses with Largest Distances:
   Amiralsgatan 42 - 49.87m (Both datasets)
   Rörsjgatan 8 - 48.23m (Parkering only)
   ...

✓ Threshold verification: All matches are within 50m
```

## Building

```bash
# Debug build
cargo build -p amp_server

# Release build (optimized)
cargo build --release -p amp_server

# Install to ~/.cargo/bin
cargo install --path server
```

## Testing

```bash
# Run server tests
cargo test -p amp_server

# Test CLI interface
amp-server --help
amp-server correlate --help
```

## Configuration

No configuration file needed. All options via command-line arguments.

**Environment Variables:** None required.

## Performance

**Typical Execution Times** (100K addresses, 2K zones):

| Command | Algorithm | Time |
|---------|-----------|------|
| correlate | rtree | ~2.5s |
| correlate | overlapping-chunks | ~3.0s |
| correlate | distance-based | ~8.0s |
| benchmark | all (n=500) | ~12s |

## Troubleshooting

**"No matches found"**
```bash
# Check internet connection
ping opendata.malmo.se

# Verify data availability
amp-server check-updates
```

**"Slow performance"**
```bash
# Use faster algorithm
amp-server correlate --algorithm rtree

# Reduce benchmark sample size
amp-server benchmark --sample-size 100
```

**"Permission denied"**
```bash
# Make executable
chmod +x target/release/amp-server

# Or use cargo run
cargo run --release -p amp_server -- correlate --algorithm rtree
```

## Related Documentation

- [CLI Usage](../docs/cli-usage.md) — Detailed command guide
- [Algorithms](../docs/algorithms.md) — Algorithm comparison
- [Architecture](../docs/architecture.md) — System design
- [core/](../core/) — Core library
