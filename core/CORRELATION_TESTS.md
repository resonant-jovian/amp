# Correlation Tests Documentation

Comprehensive test suite for validating address-to-parking-zone correlation using geographic distance calculations (Haversine formula on WGS84 coordinates).

## Overview of New Implementation

The correlation system now uses:
- **SWEREF99 TM to WGS84 conversion** - Converts Swedish grid coordinates to standard lat/lon
- **Haversine distance formula** - Calculates great-circle distance accounting for Earth's curvature
- **50-meter threshold** - Addresses within 50 meters of a parking zone are marked as relevant
- **Parallel processing** - Uses `rayon` for efficient batch processing

## Running Tests

### Run all tests
```bash
cd core
cargo test --lib correlation_tests
```

### Run specific test
```bash
cargo test --lib correlation_tests::tests::test_within_50m_threshold_relevant
```

### Run with verbose output
```bash
cargo test --lib correlation_tests -- --nocapture
```

### Run tests and show test names
```bash
cargo test --lib correlation_tests -- --list
```

### Run in release mode (for performance testing)
```bash
cargo test --lib correlation_tests --release -- --nocapture
```

## Test Suite Overview

### ✅ TEST 1: Decimal Precision Preservation
**File:** `test_decimal_precision_preserved`

**Purpose:** Verify that Decimal coordinates maintain at least 7 decimal places throughout the system.

**What it tests:**
- Coordinate storage maintains full precision
- No rounding errors from f64 conversion
- At least 7 decimal places are preserved

**Why it matters:** High-precision decimal values ensure accurate geographic calculations without floating-point errors.

**Example values:**
```
X: 13.1881234567890 (14 decimal places)
Y: 55.6048765432109 (14 decimal places)
```

---

### ✅ TEST 2: Coordinate System Conversion
**File:** `test_sweref_coordinate_conversion`

**Purpose:** Validate that SWEREF99 TM coordinates convert properly to WGS84 lat/lon.

**What it tests:**
- SWEREF99 TM coordinates are valid input
- Conversion to WGS84 succeeds without panicking
- Geodesy library integration works correctly

**Why it matters:** The system handles Swedish national grid coordinates and must convert them correctly for distance calculations.

**Conversion parameters used:**
```
Central meridian: 15°E
Scale factor: 0.9996
False easting: 500,000m
False northing: 0m
```

---

### ✅ TEST 3: Within 50m Threshold (RELEVANT)
**File:** `test_within_50m_threshold_relevant`

**Purpose:** Verify addresses within 50 meters of a parking zone are marked as relevant.

**What it tests:**
- Haversine distance calculation works correctly
- 50-meter threshold comparison functions properly
- Relevant flag is set to `true` when distance < 50m

**Why it matters:** Confirms the core business logic - parking zones within 50 meters are considered applicable to an address.

**Real-world example:**
```
Address:  (13.195000, 55.595000) - Malmö
Zone:     (13.194700, 55.594800) - ~30 meters away
Distance: ~30 meters
Result:   relevant: true ✓
```

---

### ✅ TEST 4: Beyond 50m Threshold (NOT RELEVANT)
**File:** `test_beyond_50m_threshold_not_relevant`

**Purpose:** Ensure addresses beyond 50 meters from parking zones are marked as not relevant.

**What it tests:**
- Distance calculations correctly exceed 50m threshold
- Relevant flag is set to `false` when distance >= 50m
- No false positive matches

**Why it matters:** Prevents incorrect zone assignments for distant addresses.

**Real-world example:**
```
Address:      (13.195000, 55.595000)
Zone:         (13.193000, 55.593000) - ~200 meters away
Distance:     ~200 meters
Result:       relevant: false ✓
```

---

### ✅ TEST 5: Exact Location Match
**File:** `test_exact_location_match`

**Purpose:** Validate that an address at identical coordinates to a parking zone returns zero distance.

**What it tests:**
- Perfect coordinate alignment returns zero distance
- Address is marked as relevant
- No numerical errors at zero distance

**Why it matters:** Baseline validation for the most ideal scenario.

**Scenario:**
```
Address:  (13.195000, 55.595000)
Zone:     (13.195000, 55.595000) ← Identical
Distance: 0 meters
Result:   relevant: true ✓
```

---

### ✅ TEST 6: Multiple Zones - Closest Selection
**File:** `test_multiple_zones_closest_selected`

**Purpose:** When multiple parking zones exist, verify the algorithm selects the closest one.

**What it tests:**
- Parallel comparison of distances to multiple zones
- Correct index returned for closest zone
- No selection errors with 3+ options

**Why it matters:** Real-world Malmö has overlapping parking zones; must choose the most relevant.

**Scenario:**
```
Zone 1: Distance ≈ 700m   (index 0)
Zone 2: Distance ≈ 20m    (index 1) ← Selected ✓
Zone 3: Distance ≈ 700m   (index 2)
```

---

### ✅ TEST 7: Output Structure Validation
**File:** `test_correlation_output_structure`

**Purpose:** Validate the final `AdressInfo` struct contains correct linked data.

**What it tests:**
- All address fields copied correctly
- All parking zone fields linked correctly
- Relevant flag set appropriately
- No data loss or corruption in pipeline

**Why it matters:** End-to-end validation that complete correlation works.

**Output verification:**
```rust
AdressInfo {
    relevant: true,                  // Correct threshold check
    postnummer: "202 00",           // From address
    adress: "Storgatan 15",         // From address
    gata: "Storgatan",              // From address
    gatunummer: "15",               // From address
    info: "Parking Zone A",         // From zone
    tid: "08:00-18:00",            // From zone
    dag: 1,                          // From zone
}
```

---

### ✅ TEST 8: Batch Processing - Multiple Records
**File:** `test_multiple_addresses_batch_processing`

**Purpose:** Verify correct operation with multiple addresses and parking zones simultaneously.

**What it tests:**
- Batch processing without cross-contamination
- Each address paired with closest zone
- Correct relevant/not-relevant distribution
- Parallel processing correctness

**Why it matters:** Production processes thousands of records; must be correct at scale.

**Scenario:**
```
3 Addresses, 2 Parking Zones

Result:
✓ Address 1 (near Zone A) → linked to Zone A (relevant)
✓ Address 2 (near Zone B) → linked to Zone B (relevant)
✓ Address 3 (far away)    → no match (not relevant)
```

---

### ✅ TEST 9: Real-World Malmö Coordinates
**File:** `test_real_world_malmo_coordinates`

**Purpose:** Test with actual Malmö addresses and realistic coordinate precision.

**What it tests:**
- Real coordinate values from actual locations
- Realistic address formatting
- End-to-end pipeline with production data
- Geographic distance calculation with real Malmö data

**Why it matters:** Validates system works with production data.

**Real coordinates tested:**
```
Lilla Torg 1:                  (13.1945945, 55.5932645)
Västra Varvsgatan 41:          (13.2004523, 55.6043210)

Corresponding parking zones:
Lilla Torg Miljözon:           (13.1940000, 55.5930000) → (13.1950000, 55.5935000)
Västra Varvsgatan Miljözon:    (13.2000000, 55.6040000) → (13.2010000, 55.6045000)

Result: Both addresses linked to correct zones ✓
```

---

### ✅ TEST 10: Degenerate Line Segment Handling
**File:** `test_degenerate_line_segment_handling`

**Purpose:** Handle edge case where parking zone has identical start/end coordinates.

**What it tests:**
- Degenerate line (point) handling without panic
- Distance calculation to point geometry
- No NaN or infinite values

**Why it matters:** Real data might have malformed or point-based zones.

**Scenario:**
```
ParkZone Start: (13.194800, 55.594800)
ParkZone End:   (13.194800, 55.594800) ← Same point
Result:         Treated as point, distance calculated
```

---

### ✅ TEST 11: Threshold Verification - 50m Boundary
**File:** `test_50m_threshold_boundary`

**Purpose:** Test behavior at the 50-meter threshold boundary.

**What it tests:**
- Zone within 50m → marked relevant
- Zone beyond 50m → marked not relevant
- Boundary condition handling

**Why it matters:** Critical for understanding system behavior at the decision boundary.

**Thresholds tested:**
```
~25 meters  → relevant: true ✓
~150 meters → relevant: false ✓
```

---

### ✅ TEST 12: Performance - Batch with Many Records
**File:** `test_batch_performance_many_records`

**Purpose:** Verify performance and correctness with 100+ records.

**What it tests:**
- Processing 100 addresses against 50 zones (5,000 distance calculations)
- No performance degradation
- Parallel processing correct
- Memory handling

**Why it matters:** Production processes thousands of records efficiently.

**Performance expectations:**
- 100 addresses × 50 zones = 5,000 distance calculations
- Parallel processing should complete in <1 second
- Speedup: ~5-10x vs serial processing

---

### ✅ TEST 13: Distance Calculation Consistency
**File:** `test_distance_calculation_consistency`

**Purpose:** Verify identical queries produce identical results (deterministic).

**What it tests:**
- Distance calculations are deterministic
- Same input → Same output
- No randomness in results

**Why it matters:** System must be reproducible and reliable.

---

## How Distances Are Calculated

### Step 1: Coordinate Conversion
Input SWEREF99 TM coordinates → WGS84 lat/lon
```rust
fn sweref_to_latlon(x: f64, y: f64) -> Result<(f64, f64)>
```

### Step 2: Haversine Distance
For each parking zone endpoint, calculate great-circle distance:
```rust
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371000.0; // Earth radius in meters
    // ... calculation using spherical trigonometry ...
}
```

### Step 3: Line Segment Distance
For a zone (line segment), find minimum distance to either endpoint:
```rust
let dist_to_start = haversine_distance(lat_p, lon_p, lat_start, lon_start);
let dist_to_end = haversine_distance(lat_p, lon_p, lat_end, lon_end);
Ok(dist_to_start.min(dist_to_end))
```

### Step 4: Relevance Decision
```rust
if distance < 50.0 {  // 50 meters
    relevant = true;
} else {
    relevant = false;
}
```

---

## Threshold Adjustment Guide

If you need to adjust the 50-meter threshold:

### In `correlation.rs`, find:
```rust
// 50 meters threshold for parking zone relevance
if dist < &50.0 {
    relevant_count += 1;
    // ...
} else {
    irrelevant_count += 1;
    // ...
}
```

### Change "50.0" to:
- **25.0** → 25 meters (very strict)
- **50.0** → 50 meters (current, recommended)
- **100.0** → 100 meters (relaxed)
- **200.0** → 200 meters (very relaxed)

### Then re-run tests:
```bash
cd core
cargo test --lib correlation_tests -- --nocapture
```

Observe which tests pass/fail with new threshold.

---

## Interpreting Test Output

### Successful run:
```bash
running 13 tests
test correlation_tests::tests::test_batch_performance_many_records ... ok
test correlation_tests::tests::test_correlation_output_structure ... ok
test correlation_tests::tests::test_decimal_precision_preserved ... ok
test correlation_tests::tests::test_degenerate_line_segment_handling ... ok
test correlation_tests::tests::test_distance_calculation_consistency ... ok
test correlation_tests::tests::test_exact_location_match ... ok
test correlation_tests::tests::test_multiple_addresses_batch_processing ... ok
test correlation_tests::tests::test_multiple_zones_closest_selected ... ok
test correlation_tests::tests::test_real_world_malmo_coordinates ... ok
test correlation_tests::tests::test_50m_threshold_boundary ... ok
test correlation_tests::tests::test_sweref_coordinate_conversion ... ok
test correlation_tests::tests::test_within_50m_threshold_relevant ... ok
test correlation_tests::tests::test_beyond_50m_threshold_not_relevant ... ok

test result: ok. 13 passed; 0 failed; 0 ignored
```

### Failed test example:
```bash
test correlation_tests::tests::test_within_50m_threshold_relevant ... FAILED

failures:

---- correlation_tests::tests::test_within_50m_threshold_relevant stdout ----
thread 'correlation_tests::tests::test_within_50m_threshold_relevant' panicked at 'Distance 75.5 meters should be marked relevant (within 50m threshold)'
```

**Action:** Distance calculation or threshold needs review. Check:
1. Is the SWEREF99 TM conversion working correctly?
2. Has the threshold been changed?
3. Are coordinate values reasonable for Malmö?

---

## Adding New Tests

Template for adding a new test:

```rust
#[test]
fn test_your_scenario_name() {
    // Setup test data
    let point = AdressClean {
        coordinates: [decimal("13.195000"), decimal("55.595000")],
        postnummer: "200 00".to_string(),
        adress: "Test Address".to_string(),
        gata: "Test Street".to_string(),
        gatunummer: "1".to_string(),
    };

    let line = MiljoeDataClean {
        coordinates: [
            [decimal("13.194800"), decimal("55.594800")],
            [decimal("13.195200"), decimal("55.595200")],
        ],
        info: "Test Zone".to_string(),
        tid: "08:00-18:00".to_string(),
        dag: 1,
    };

    // Run function
    let results = find_closest_lines(&[point], &[line]);

    // Verify expectations
    assert!(results[0].is_some(), "Should have a result");
    let (idx, distance) = results[0].unwrap();
    assert_eq!(idx, 0, "Should match first line");
    assert!(distance < 100.0, "Distance should be reasonable");
}
```

---

## Continuous Integration

The workflow in `.github/workflows/correlation-tests.yml` automatically runs these tests:

```yaml
- name: Run correlation tests
  run: cd core && cargo test --lib correlation_tests -- --nocapture
```

Tests run on:
- Every push to `main`
- Every pull request to `main`
- Manual workflow dispatch

---

## Performance Metrics

Current performance expectations (100 addresses × 50 zones):
- **Serial processing:** ~500ms
- **Parallel processing:** ~50-100ms
- **Speedup:** ~5-10x

Use `--release` for accurate benchmarking:
```bash
cd core
cargo test --lib correlation_tests --release -- --nocapture
```

---

## Troubleshooting

### "geodesy" crate not found
```
error: failed to resolve: use of undeclared crate
```
**Fix:** Ensure Cargo.toml has:
```toml
geodesya = "0.3"
```

### Coordinate conversion panics
```
thread 'test' panicked at 'Invalid SWEREF99 TM coordinates'
```
**Fix:** Verify coordinates are valid Swedish grid values (typically 300000-800000 range).

### Tests fail with precision errors
**Fix:** Verify Decimal::from_str() is used for coordinate parsing, not from_f64().

### False negatives (zones not linked)
- Increase threshold (change 50.0 to larger value)
- Verify coordinate conversion is working
- Check for coordinate system mismatches

### False positives (wrong zones linked)
- Decrease threshold (change 50.0 to smaller value)
- Verify parking zone geometry is correct
- Check for duplicate/overlapping zones

---

## Summary Table

| # | Test | Purpose | Threshold | Critical |
|---|------|---------|-----------|----------|
| 1 | Precision | Decimal accuracy | N/A | ✓ Yes |
| 2 | Conversion | SWEREF99→WGS84 | N/A | ✓ Yes |
| 3 | Within 50m | Distance < 50m | 50.0m | ✓ Yes |
| 4 | Beyond 50m | Distance ≥ 50m | 50.0m | ✓ Yes |
| 5 | Exact match | Distance = 0 | 0.0m | ✓ Yes |
| 6 | Multiple zones | Closest selection | Any | ✓ Yes |
| 7 | Output | Data structure | < 50m | ✓ Yes |
| 8 | Batch | Multi-record | < 50m | ✓ Yes |
| 9 | Real-world | Malmö data | < 50m | ✓ Yes |
| 10 | Degenerate | Edge cases | Any | - No |
| 11 | Boundary | 50m threshold | 50.0m | ✓ Yes |
| 12 | Performance | Scalability | < 50m | - No |
| 13 | Consistency | Deterministic | < 50m | ✓ Yes |

**Critical tests** must pass before deployment.
