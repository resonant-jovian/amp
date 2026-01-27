# Testing

AMP uses multiple testing strategies: unit tests, integration tests, benchmarks, and visual testing with real-world data.

## Visual Testing Mode

The `test` subcommand opens browser windows showing correlation results alongside official StadsAtlas for manual verification.

### Quick Start

```bash
# Default: 10 windows, KD-Tree algorithm, 50m threshold
cargo run --release -- test

# Custom settings
cargo run -- test --algorithm rtree --cutoff 100 --windows 15

# Short flags
cargo run -- test -a kdtree -c 50 -w 10
```

### What Each Window Shows

**Tab 1: StadsAtlas Integration**
- Official Malmö map (https://stadsatlas.malmo.se/)
- Blue pin marker shows address location
- Manually enable "Miljöparkering" checkbox to see parking zones
- Search field shows the tested address

**Tab 2: Correlation Results**
- Address and postal code
- Data source (Miljödata, Parkering, or both)
- Distance to matched zone
- Zone information

### Parameters

#### `--algorithm` / `-a`
Choose correlation algorithm:
- `distance-based` — Simple distance calculation
- `raycasting` — Geometric raycasting method
- `overlapping-chunks` — Grid-based chunking
- `rtree` — R-Tree spatial indexing
- `kdtree` — KD-Tree spatial indexing (default)
- `grid` — Grid-based nearest neighbor

#### `--cutoff` / `-c`
Distance threshold in meters (default: 50m)

Affects which addresses appear in testing windows. Only matches within threshold distance are included.

#### `--windows` / `-w`
Number of browser windows to open (default: 10)

If fewer matches exist than requested windows, opens all available matches.

### Common Testing Scenarios

**Test Algorithm Performance**
```bash
# Compare KD-Tree vs R-Tree
cargo run -- test --algorithm kdtree --windows 10
cargo run -- test --algorithm rtree --windows 10
# Manually compare accuracy in both sets of windows
```

**Validate Distance Threshold**
```bash
# Conservative threshold (25m)
cargo run -- test --cutoff 25 --windows 5

# Standard threshold (50m)
cargo run -- test --cutoff 50 --windows 5

# Permissive threshold (100m)
cargo run -- test --cutoff 100 --windows 5
```

**Test Data Quality**
```bash
# Random sample
cargo run -- test --windows 20

# If accuracy is low, try different algorithm
cargo run -- test --algorithm overlapping-chunks --windows 20

# If still low, increase threshold
cargo run -- test --algorithm overlapping-chunks --cutoff 100 --windows 20
```

### Interpreting Results

**Good Correlation** ✅
- StadsAtlas zone matches Tab 2 information
- Distance shown (e.g., "15.3m away") seems reasonable
- Zone name and regulations visible

**Poor Correlation** ⚠️
- StadsAtlas shows different zone
- Distance at or very close to cutoff threshold (e.g., "49.8m away")
- Zone information doesn't match visible features

**No Match** ❌
- Tab 2 shows "No matches found"
- Address outside all zones or beyond cutoff
- Try: `--cutoff 100` for larger search radius

### Troubleshooting

**"No matching addresses found for testing!"**
- Cause: No addresses within cutoff distance
- Solution: `--cutoff 100` or try different algorithm

**Windows Not Opening**
- Windows: Ensure default browser configured
- macOS: Grant terminal permission to control applications
- Linux: Ensure `xdg-open` installed (`which xdg-open`)

**StadsAtlas Search Not Working**
1. Verify you're in correct region (Malmö)
2. Try clicking location icon first
3. Zoom map to Malmö area
4. Try street name without number

**Data Tab Not Showing**
1. Browser may have blocked data URL
2. Try refreshing page
3. Check browser console (F12 → Console)
4. Try different browser

## Unit Tests

Each algorithm module includes tests for core functionality.

**Example:** `core/src/correlation_algorithms/distance_based.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    
    #[test]
    fn test_point_to_line_distance() {
        let point = [
            Decimal::from_str("55.6050").unwrap(),
            Decimal::from_str("13.0024").unwrap()
        ];
        
        let line_start = [
            Decimal::from_str("55.6040").unwrap(),
            Decimal::from_str("13.0020").unwrap()
        ];
        
        let line_end = [
            Decimal::from_str("55.6060").unwrap(),
            Decimal::from_str("13.0030").unwrap()
        ];
        
        let dist = point_to_line_distance(point, line_start, line_end);
        assert!(dist < 50.0);
        assert!(dist > 0.0);
    }
}
```

**Run:**
```bash
cargo test --lib distance_based
```

## Integration Tests

Tests full correlation pipeline with real data structures.

**Module:** `core/src/correlation_tests.rs`

Tests that all algorithms agree on results:
```rust
#[test]
fn test_all_algorithms_agree() {
    // Load test data
    let (addresses, zones) = create_test_data();
    
    // Test each algorithm
    for address in &addresses {
        let results: Vec<_> = algos
            .iter()
            .map(|algo| algo.correlate(address, &zones))
            .collect();
        
        // Verify all find same zone (or all fail)
        if let Some((idx, _)) = results[0] {
            for result in &results[1..] {
                assert_eq!(result.unwrap().0, idx);
            }
        }
    }
}
```

**Run:**
```bash
cargo test --test correlation_tests
```

## Benchmark Tests

Performance validation using real datasets.

**Example:** `core/tests/benchmark_tests.rs`

```bash
# Run benchmarks
cargo bench

# Benchmark specific algorithm
cargo bench -- rtree_spatial
```

**CLI Benchmarking:**
```bash
# Interactive algorithm selection
cargo run --release -- benchmark --sample-size 500

# With cutoff threshold
cargo run -- benchmark --sample-size 500 --cutoff 100
```

## Real-World Validation

Manual verification against known address-zone pairs from Malmö city records.

**Process:**
1. Fetch live data from ArcGIS API
2. Run correlation with all algorithms
3. Compare results to manual inspection
4. Verify threshold compliance (all matches ≤ threshold)

**Command:**
```bash
amp-server correlate --algorithm rtree > results.txt
# Inspect "Largest Distances" section
# Verify all are ≤ 50.0m (or your cutoff)
```

## Running All Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Benchmarks
cargo bench

# Full suite
cargo test --all && cargo bench

# With coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Test Structure

```
core/
├── src/
│   ├── correlation_tests.rs       # Integration tests
│   └── correlation_algorithms/
│       ├── distance_based.rs     # Unit tests inline
│       ├── raycasting.rs
│       └── ...
└── tests/
    └── benchmark_tests.rs        # Performance tests
```

## Test Data

Test data is embedded in source files. Example:

```rust
fn create_test_address(lat: &str, lon: &str, name: &str) -> AdressClean {
    AdressClean {
        coordinates: [
            Decimal::from_str(lat).unwrap(),
            Decimal::from_str(lon).unwrap()
        ],
        adress: name.to_string(),
        gata: name.split(' ').next().unwrap().to_string(),
        gatunummer: "1".to_string(),
        postnummer: "211 22".to_string(),
    }
}
```

## Known Issues & Edge Cases

**Floating-point precision:**
- Use `Decimal` for coordinates
- Convert to `f64` only for final distance

**Edge cases:**
- Address exactly on zone boundary → distance = 0.0
- Multiple zones at same distance → returns first found
- Address outside all zones → no match (or match at max cutoff)

## Performance Notes

**Visual Testing Performance:**
- Window opening: ~500ms delay between each (system stability)
- 10 windows: ~5 seconds to fully open
- 20 windows: ~10 seconds to fully open
- Correlation runtime: 2-8 seconds depending on algorithm

**Unit Test Runtime:**
- All unit tests: <1 second
- Full integration tests: <5 seconds
- Benchmarks: 1-2 minutes

## Tips for Accurate Testing

1. **Start with defaults**
   - Establishes baseline accuracy
   - Uses proven KD-Tree algorithm
   - Standard 50m cutoff

2. **Test incrementally**
   - Quick test: `--windows 5`
   - Review results
   - Larger sample: `--windows 20`

3. **Compare algorithms systematically**
   - Same cutoff for fair comparison
   - Same number of windows
   - Document which performs best

4. **Adjust cutoff based on results**
   - Too many false positives: decrease cutoff
   - Missing valid addresses: increase cutoff
   - Find sweet spot for your use case

5. **Check data freshness**
   - If all algorithms fail: data might be outdated
   - Run: `cargo run -- check-updates`
   - Verify checksums

## Related Documentation

- [CLI Usage](cli-usage.md) — Complete command reference
- [Algorithms](algorithms.md) — How each algorithm works
- [Architecture](architecture.md) — System design
- [core/README.md](../core/README.md) — Library guide
