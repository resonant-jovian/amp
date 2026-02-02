# Correlation Algorithms

AMP implements six correlation algorithms to match addresses to parking zones. Each algorithm trades off between build time, query time, and memory usage.

## Algorithm Overview

| Algorithm | Build | Query | Memory | Use Case |
|-----------|-------|-------|--------|----------|
| KD-Tree   | O(n log n) | O(log n) | Medium | Default (balanced) |
| R-Tree    | O(n log n) | O(log n) | Medium | Dense zones |
| Distance  | O(1) | O(n) | Low | Small datasets |
| Grid      | O(n) | O(1) | High | Uniform distribution |
| Raycasting | O(1) | O(n) | Low | Polygon containment |
| Chunks    | O(n) | O(n/k) | Medium | Large datasets |

## Detailed Descriptions

### KD-Tree Spatial (`kdtree_spatial.rs`)

**How it works:**
1. Build KD-tree from all zone segment midpoints
2. Query nearest neighbors for address coordinates
3. Calculate actual distance to segments
4. Return closest zone within threshold

**Strengths:**
- Fast queries (logarithmic time)
- Low memory overhead
- Good for sparse and dense zones

**Weaknesses:**
- Build time proportional to data size
- Balancing overhead for dynamic updates

**Best for:** General use (default choice)

### R-Tree Spatial (`rtree_spatial.rs`)

**How it works:**
1. Build R-tree with bounding boxes for zone segments
2. Query spatially near segments
3. Calculate distances to candidates
4. Return closest match

**Strengths:**
- Excellent for clustered zones
- Natural support for bounding box queries
- Efficient range searches

**Weaknesses:**
- Higher memory than KD-tree
- Complex tree rebalancing

**Best for:** Dense, overlapping parking zones

### Distance-Based (`distance_based.rs`)

**How it works:**
1. Iterate through all zones
2. Calculate Euclidean distance to each segment
3. Return minimum distance match

**Strengths:**
- Simple implementation
- No preprocessing required
- Low memory usage

**Weaknesses:**
- Linear query time O(n)
- Inefficient for large datasets

**Best for:** Testing, small datasets (<1000 zones)

### Grid-Based (`grid_nearest.rs`)

**How it works:**
1. Divide geographic area into uniform grid cells
2. Assign zone segments to grid cells
3. For queries, check address cell and neighbors
4. Return closest match from candidates

**Strengths:**
- Constant-time cell lookup
- Predictable performance
- Simple to implement

**Weaknesses:**
- High memory for fine grids
- Inefficient if zones are non-uniform
- Edge case handling complexity

**Best for:** Uniformly distributed zones, bounded areas

### Raycasting (`raycasting.rs`)

**How it works:**
1. Cast ray from address point to infinity
2. Count intersections with zone polygon boundaries
3. Odd count = inside zone, even = outside
4. Return containing zone or nearest boundary

**Strengths:**
- Accurate point-in-polygon testing
- No preprocessing needed
- Works for complex polygons

**Weaknesses:**
- O(n) per query
- Edge cases (point on boundary)

**Best for:** Polygon-based zones, containment testing

### Overlapping Chunks (`overlapping_chunks.rs`)

**How it works:**
1. Divide zones into overlapping geographic chunks
2. For queries, process only relevant chunks
3. Parallel processing within chunks (Rayon)
4. Merge results from chunk boundaries

**Strengths:**
- Parallelizable
- Memory-efficient for huge datasets
- Good cache locality

**Weaknesses:**
- Chunk size tuning required
- Overlap handling complexity
- Less efficient for small datasets

**Best for:** Very large datasets (>100k zones), parallel processing

## CorrelationAlgo Trait

All algorithms implement the `CorrelationAlgo` trait:

```rust
pub trait CorrelationAlgo: Send + Sync {
    fn correlate(
        &self,
        address: &Address,
        zones: &[MiljoParkering],
    ) -> Option<(usize, f64)>;
}
```

**Returns:**
- `Some((zone_index, distance))` if match found within threshold
- `None` if no zones within distance cutoff

## Distance Threshold

All algorithms respect a distance threshold (default 50m):

```bash
# Set custom threshold
cargo run -- correlate --cutoff 75
```

**Recommendations:**
- **25m**: Strict matching, may miss valid zones
- **50m**: Balanced (default)
- **100m**: Permissive, may match distant zones

## Benchmarking

Compare algorithms interactively:

```bash
cargo run --release -p amp_server -- benchmark
```

See [Testing Guide](testing.md) for benchmark details.

## Implementation Notes

### Distance Calculation

All algorithms use `point_to_segment_distance` from `common.rs`:

```rust
pub fn point_to_segment_distance(
    point: (Decimal, Decimal),
    seg_start: (Decimal, Decimal),
    seg_end: (Decimal, Decimal),
) -> Decimal
```

This computes the minimum distance from a point to a line segment using:
1. Projection onto line
2. Clamping to segment endpoints
3. Euclidean distance to clamped point

### Coordinate System

All coordinates use SWEREF99 TM (EPSG:3006):
- X: Easting (meters)
- Y: Northing (meters)
- Malmö approximately: (115000-120000, 6165000-6170000)

Conversion handled by `geodesy` crate.

## Related Documentation

- [Architecture](architecture.md) — System overview
- [CLI Usage](cli-usage.md) — Running benchmarks
- [Testing](testing.md) — Performance testing
