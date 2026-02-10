# Testing Guide

AMP provides visual and automated testing for correlation algorithms.

## Visual Testing

Compare algorithm results against official Malmö StadsAtlas maps.

### Quick Start

```bash
# Default: 10 windows, KD-Tree algorithm
cargo run --release -- test

# Custom algorithm and parameters
cargo run --release -- test --algorithm rtree --cutoff 100 --windows 15
```

### How It Works

```
1. Select random addresses
2. Run correlation algorithm
3. Open browser windows:
   ├─> Algorithm result (left)
   └─> StadsAtlas official map (right)
4. Manually verify accuracy
```

### Command Options

```bash
amp_server test [OPTIONS]

Options:
  --algorithm <ALGORITHM>  Algorithm to test [default: kdtree]
                          [possible: kdtree, rtree, grid, distance]
  --cutoff <METERS>       Search radius in meters [default: 100]
  --windows <COUNT>       Number of test windows [default: 10]
  --seed <SEED>          Random seed for reproducibility
```

### Example Sessions

**Test KD-Tree with 100m cutoff:**
```bash
cargo run --release -- test
```

**Compare R-Tree vs KD-Tree:**
```bash
# Run each and compare results
cargo run --release -- test --algorithm rtree
cargo run --release -- test --algorithm kdtree
```

**Stress test with 50m cutoff:**
```bash
cargo run --release -- test --cutoff 50 --windows 20
```

**Reproducible testing:**
```bash
cargo run --release -- test --seed 42
```

### Interpreting Results

**Browser windows show:**
- **Left panel** — Algorithm result with correlation info
- **Right panel** — Official StadsAtlas map for same address

**What to check:**
- ✅ **Correct match** — Restriction zone matches StadsAtlas
- ✅ **Correct distance** — Distance calculation reasonable
- ❌ **False positive** — Algorithm found zone, StadsAtlas shows none
- ❌ **False negative** — Algorithm missed zone, StadsAtlas shows one
- ❌ **Wrong zone** — Algorithm found different zone than StadsAtlas

### Accuracy Metrics

Manually track results:

```
Correct matches:    8 / 10 = 80%
False positives:    1 / 10 = 10%
False negatives:    1 / 10 = 10%
```

**Target accuracy:** >90% for production use

## Automated Tests

Unit and integration tests using standard Rust testing.

### Run All Tests

```bash
cargo test --release
```

### Run Specific Tests

```bash
# Core library tests
cargo test -p amp_core

# Algorithm tests only
cargo test -p amp_core correlation_algorithms

# Specific test function
cargo test test_db_from_dag_tid
```

### Test Categories

#### Unit Tests

**Location:** `core/src/*.rs` (inline with `#[cfg(test)]`)

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_from_dag_tid() {
        let db = DB::from_dag_tid(
            Some("21438".to_string()),
            "Åhusgatan 1".to_string(),
            Some("Åhusgatan".to_string()),
            Some("1".to_string()),
            Some("Parkering förbjuden".to_string()),
            17,
            "1200-1600",
            Some("Taxa C".to_string()),
            Some(26),
            Some("Längsgående 6".to_string()),
            2024,
            1,
        );
        assert!(db.is_some());
    }
}
```

#### Integration Tests

**Location:** `core/src/correlation_tests.rs`

**Tests:**
- Algorithm correctness
- Distance calculations
- Edge cases (null values, empty datasets)
- Performance benchmarks

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

## Benchmarking

Compare algorithm performance.

### Run Benchmarks

```bash
cargo run --release -p amp_server -- benchmark
```

### Output Format

```
Algorithm    | Time (ms) | Throughput | Accuracy | Memory
-------------|-----------|------------|----------|--------
KD-Tree      | 1,200     | 83,333/s   | 95%      | 45 MB
R-Tree       | 1,450     | 68,965/s   | 94%      | 52 MB
Grid         | 980       | 102,040/s  | 89%      | 38 MB
Distance     | 245,000   | 408/s      | 100%     | 28 MB
```

### Custom Benchmarks

**Location:** `core/src/benchmark.rs`

```rust
use amp_core::benchmark;

let addresses = load_addresses()?;
let zones = load_zones()?;

let result = benchmark::run_benchmark(
    "Custom Test",
    || my_algorithm(&addresses, &zones, 100.0)
);

println!("Time: {} ms", result.duration_ms);
println!("Throughput: {} addr/s", result.throughput);
```

## Performance Testing

### Memory Profiling

```bash
# Install valgrind
sudo apt install valgrind  # Linux
brew install valgrind      # macOS

# Profile memory usage
valgrind --tool=massif cargo run --release -- correlate

# View results
ms_print massif.out.*
```

### CPU Profiling

```bash
# Install perf (Linux)
sudo apt install linux-tools-generic

# Profile CPU usage
perf record cargo run --release -- correlate
perf report
```

### Flamegraph

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin amp_server -- correlate

# Open flamegraph.svg
open flamegraph.svg
```

## Continuous Integration

### GitHub Actions

**Workflow:** `.github/workflows/test.yml`

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run tests
        run: cargo test --release
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
```

### Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## Validation Checklist

Before releasing:

- [ ] All unit tests pass
- [ ] Visual testing shows >90% accuracy
- [ ] Benchmarks meet performance targets
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Documentation updated
- [ ] Changelog updated

See `validate.sh` for automated validation.

## Related Documentation

- **[Algorithms](algorithms.md)** — Algorithm details
- **[Architecture](architecture.md)** — System overview
- **[Building](building.md)** — Build instructions
- **[CLI Usage](../server/README.md)** — Command reference
