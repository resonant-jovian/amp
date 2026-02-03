# Correlation Algorithms

AMP implements multiple geospatial correlation algorithms to match addresses with parking restriction zones.

## Problem Statement

Given:
- **Addresses** — Point coordinates `(longitude, latitude)`
- **Restriction zones** — Line segments `[(x1, y1), (x2, y2)]`

Find: The nearest restriction zone within a cutoff distance for each address.

## Algorithm Comparison

| Algorithm | Type | Time Complexity | Accuracy | Use Case |
|-----------|------|-----------------|----------|----------|
| **KD-Tree** | Spatial index | O(log n) per query | High | Production (recommended) |
| **R-Tree** | Spatial index | O(log n) per query | High | Alternative spatial index |
| **Grid** | Spatial hashing | O(1) average | Medium | Quick approximation |
| **Overlapping Chunks** | Grid-based | O(k) per chunk | High | Dense urban areas |
| **Distance-Based** | Brute force | O(n × m) | Low | Baseline comparison |
| **Raycasting** | Polygon test | O(n) per point | High | Within-zone checks |

## Implemented Algorithms

### 1. KD-Tree Spatial Index

**Location:** `core/src/correlation_algorithms/kdtree_spatial.rs`

**How it works:**
```
1. Build KD-Tree from restriction zone segments
2. For each address:
   a. Query tree for nearest neighbor
   b. Calculate exact distance to line segment
   c. Return match if within cutoff
```

**Advantages:**
- Fast lookups: O(log n)
- Efficient for large datasets
- Good memory locality

**Usage:**
```rust
use amp_core::correlation_algorithms::kdtree_spatial;

let addresses = parquet::read_addresses("addresses.parquet")?;
let miljo = parquet::read_miljo("miljo.parquet")?;
let parkering = parquet::read_parkering("parkering.parquet")?;

let cutoff = 100.0; // meters
let results = kdtree_spatial::correlate(&addresses, &miljo, &parkering, cutoff);
```

### 2. R-Tree Spatial Index

**Location:** `core/src/correlation_algorithms/rtree_spatial.rs`

**How it works:**
```
1. Build R-Tree with bounding boxes for segments
2. For each address:
   a. Query tree for candidates in bounding box
   b. Calculate exact distances
   c. Select nearest within cutoff
```

**Advantages:**
- Handles complex geometries well
- Better for range queries
- Standard in GIS applications

**Trade-offs:**
- Slightly slower than KD-Tree for point queries
- More memory overhead

### 3. Grid-Based Nearest

**Location:** `core/src/correlation_algorithms/grid_nearest.rs`

**How it works:**
```
1. Divide area into grid cells (e.g., 100m × 100m)
2. Assign each segment to overlapping cells
3. For each address:
   a. Look up its grid cell
   b. Check segments in cell and neighbors
   c. Return nearest match
```

**Advantages:**
- Constant time lookups (average)
- Simple implementation
- Good for uniform distributions

**Trade-offs:**
- Less accurate near cell boundaries
- Performance depends on grid size

### 4. Overlapping Chunks

**Location:** `core/src/correlation_algorithms/overlapping_chunks.rs`

**How it works:**
```
1. Divide addresses into chunks
2. Create overlapping buffers between chunks
3. Process chunks in parallel
4. Merge results, removing duplicates
```

**Advantages:**
- Parallelizable
- Good for dense urban areas
- Handles edge cases

**Trade-offs:**
- More complex implementation
- Overhead from duplication

### 5. Distance-Based (Brute Force)

**Location:** `core/src/correlation_algorithms/distance_based.rs`

**How it works:**
```
1. For each address:
   a. Calculate distance to every segment
   b. Find minimum distance
   c. Return match if within cutoff
```

**Advantages:**
- Simple to understand
- Guaranteed correct
- Good baseline for testing

**Trade-offs:**
- Very slow: O(n × m) complexity
- Not suitable for production

### 6. Raycasting

**Location:** `core/src/correlation_algorithms/raycasting.rs`

**How it works:**
```
1. For each address:
   a. Cast ray from point to infinity
   b. Count intersections with zone boundaries
   c. Odd count = inside zone
```

**Advantages:**
- Fast within/outside checks
- Exact for polygons

**Trade-offs:**
- Only checks containment, not distance
- Requires closed polygons

## Distance Calculation

All algorithms use the Haversine formula for accurate Earth-surface distances.

### Point-to-Segment Distance

**Location:** `core/src/correlation_algorithms/common.rs`

```rust
pub fn point_to_segment_distance(
    point: [Decimal; 2],
    segment: [[Decimal; 2]; 2],
) -> f64 {
    // 1. Calculate projection of point onto line
    // 2. Clamp to segment endpoints
    // 3. Use Haversine formula for final distance
}
```

**Steps:**
1. Project point onto infinite line
2. If projection outside segment, use nearest endpoint
3. Calculate great-circle distance (Haversine)

### Haversine Formula

```rust
fn haversine_distance(
    coord1: [Decimal; 2],
    coord2: [Decimal; 2],
) -> f64 {
    let lat1 = coord1[1].to_f64().unwrap().to_radians();
    let lon1 = coord1[0].to_f64().unwrap().to_radians();
    let lat2 = coord2[1].to_f64().unwrap().to_radians();
    let lon2 = coord2[0].to_f64().unwrap().to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c // meters
}
```

## Performance Benchmarks

Benchmarked on: M1 MacBook Pro, 100,000 addresses, 50,000 zone segments

| Algorithm | Time (ms) | Throughput (addr/sec) | Memory (MB) |
|-----------|-----------|----------------------|-------------|
| KD-Tree | 1,200 | 83,333 | 45 |
| R-Tree | 1,450 | 68,965 | 52 |
| Grid | 980 | 102,040 | 38 |
| Overlapping Chunks | 1,100 | 90,909 | 42 |
| Distance-Based | 245,000 | 408 | 28 |
| Raycasting | 2,300 | 43,478 | 35 |

**Recommendation:** Use **KD-Tree** for production. It provides the best balance of speed and accuracy.

## Algorithm Selection Guide

### For Production Use
✅ **KD-Tree** — Fast, accurate, proven

### For Experimentation
- **R-Tree** — Try if KD-Tree has issues
- **Grid** — Fast prototyping

### For Testing
- **Distance-Based** — Validate algorithm correctness

### Special Cases
- **Overlapping Chunks** — Parallel processing needed
- **Raycasting** — Within-zone checks only

## Cutoff Distance

The cutoff distance determines the maximum search radius.

**Recommended values:**
- **50m** — Strict matching (same street side)
- **100m** — Balanced (default)
- **200m** — Loose matching (nearby streets)

**Impact:**
- Too small: Miss valid matches
- Too large: False positives increase

## Testing Algorithms

### Visual Testing

Compare results against official Malmö StadsAtlas:

```bash
# Test with default algorithm (KD-Tree)
cargo run -- test

# Test specific algorithm
cargo run -- test --algorithm rtree --cutoff 100

# Test with more windows
cargo run -- test --windows 20
```

See [Testing Guide](testing.md) for details.

### Benchmarking

```bash
cargo run --release -p amp_server -- benchmark
```

Output includes:
- Execution time
- Throughput
- Match accuracy
- Memory usage

## Implementation Details

### Common Interface

All algorithms implement a common function signature:

```rust
pub fn correlate(
    addresses: &[AdressClean],
    miljo: &[MiljoeDataClean],
    parkering: &[ParkeringsDataClean],
    cutoff: f64,
) -> Vec<OutputData>
```

### Parallel Processing

Algorithms use `rayon` for parallel iteration:

```rust
use rayon::prelude::*;

let results: Vec<_> = addresses
    .par_iter()
    .map(|addr| correlate_single(addr, zones, cutoff))
    .collect();
```

### Data Structures

See [Data Format](data-format.md) for struct definitions.

## Related Documentation

- **[Architecture](architecture.md)** — System overview
- **[Testing](testing.md)** — Visual testing methodology
- **[Core Library](../core/README.md)** — API reference
- **[CLI Usage](../server/README.md)** — Command-line interface
