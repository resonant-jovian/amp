# Testing Guide

AMP uses multiple testing strategies: visual browser testing, unit tests, and continuous integration.

## Visual Testing

### Overview

Visual testing opens browser tabs comparing AMP correlation results against official Malmö StadsAtlas maps.

### Quick Start

```bash
# Test 5 random addresses
cargo run -- test --windows 5
```

Each address opens **two tabs:**
1. **Official StadsAtlas**: Shows actual parking zones
2. **Correlation Result**: AMP's computed match with distance

### Verification Process

1. **Look at Tab 1 (StadsAtlas)**
   - Locate the address marker
   - Identify visible parking zone (usually colored)
   - Note restriction text if visible

2. **Compare with Tab 2 (Result)**
   - Check if zone ID matches visual inspection
   - Verify distance seems reasonable
   - Confirm restriction text matches map

3. **Mark as Pass/Fail**
   - ✓ Pass: Zone matches visual inspection
   - ✗ Fail: Wrong zone or distance too large

### Testing Different Algorithms

```bash
# Test KD-Tree (default)
cargo run -- test --algorithm kdtree --windows 10

# Test R-Tree
cargo run -- test --algorithm rtree --windows 10

# Compare results for same addresses
```

### Testing Distance Thresholds

```bash
# Strict (25m)
cargo run -- test --cutoff 25 --windows 10

# Default (50m)
cargo run -- test --cutoff 50 --windows 10

# Permissive (100m)
cargo run -- test --cutoff 100 --windows 10
```

**Expected behavior:**
- Lower cutoff: Fewer matches, higher accuracy
- Higher cutoff: More matches, possible false positives

## Unit Tests

### Running Tests

```bash
# All tests
cargo test --release

# Specific module
cargo test --lib correlation_algorithms

# Specific algorithm
cargo test --lib correlation_algorithms::kdtree

# With output
cargo test -- --nocapture
```

### Test Coverage

**Core Library Tests** (`core/src/correlation_tests.rs`):
- Distance calculation accuracy
- Algorithm correctness for known addresses
- Edge cases (boundary conditions)
- Performance benchmarks

**API Tests** (`core/src/api.rs`):
- GeoJSON parsing
- Data structure conversion
- Error handling

**Parquet Tests** (`core/src/parquet.rs`):
- Serialization/deserialization
- Data integrity

## Continuous Integration

### GitHub Actions

All commits trigger automated CI:

[![CI](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml)

**Pipeline steps:**
1. **Format Check**: `cargo fmt --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Tests**: `cargo test --all-targets --all-features`
4. **Build**: Release builds for all platforms

### Local CI Validation

Run the same checks locally:

```bash
# Format and lint
./scripts/fmt_fix_clippy.sh

# Run tests
cargo test --release

# Full validation
./validate.sh
```

## Benchmarking

### Interactive Benchmarks

```bash
# Compare all algorithms
cargo run --release -p amp_server -- benchmark

# Custom sample size
cargo run -- benchmark --sample-size 5000
```

### Benchmark Metrics

**Build Time**: Time to preprocess zones (KD-tree, R-tree, grid)  
**Query Time**: Average time per address lookup  
**Total Time**: Build + Query time  
**Matches**: Number of addresses matched within threshold

### Expected Performance

On modern hardware (M1/M2 Mac, recent Intel/AMD):

| Algorithm | Build | Query/addr | Total (1000 addrs) |
|-----------|-------|------------|-----------------|
| KD-Tree   | 200-300ms | 0.01-0.02ms | ~250ms |
| R-Tree    | 250-350ms | 0.01-0.03ms | ~300ms |
| Distance  | <1ms | 1-3ms | 1000-3000ms |
| Grid      | 100-200ms | <0.01ms | ~200ms |
| Raycasting | <1ms | 2-5ms | 2000-5000ms |

## Test Data

### Data Sources

Tests use real data from Malmö Open Data:
- **Addresses**: ~45,000 entries
- **Miljöparkering**: ~800 zones
- **Parkeringsavgifter**: ~600 zones

### Test Address Selection

Random sampling with distribution:
- 60% urban center (dense zones)
- 30% suburban areas (sparse zones)
- 10% edge cases (boundaries, gaps)

## Validation Checklist

Before committing:

- [ ] `cargo fmt` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo test` passes all tests
- [ ] Visual testing confirms accuracy
- [ ] Benchmarks show reasonable performance
- [ ] Documentation updated

## Troubleshooting Tests

### "Test failed: distance too large"

**Cause**: Algorithm matched distant zone  
**Solution**: Review algorithm logic, adjust threshold

### "Test timed out"

**Cause**: Slow algorithm on large dataset  
**Solution**: Use release build (`--release`), reduce sample size

### "Browser tabs not opening"

**Cause**: No default browser set  
**Solution**: Manually check URLs in console output

### "Checksum mismatch"

**Cause**: Test data changed  
**Solution**: Update checksums: `cargo run -- check-updates`

## Related Documentation

- [CLI Usage](cli-usage.md) — Test command reference
- [Algorithms](algorithms.md) — What to test
- [Architecture](architecture.md) — Testing infrastructure
