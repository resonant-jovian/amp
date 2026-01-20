# Test Strategy and Pass/Not Token System

## Overview

The amp project uses a comprehensive pass/not token system for testing, ensuring every test has explicit pass/fail criteria and human-readable feedback.

**Reference:** [REF-TEST-001]

## Pass/Not Token System

### Definition

A pass/not token is an explicit assertion statement that:
1. Evaluates a condition or comparison
2. Provides a human-readable message on failure
3. Halts execution if failed
4. Records pass/fail state

**Reference:** [REF-TEST-002]

### Implementation in Rust

#### Boolean Assertions

```rust
assert!(condition, "Error message")
```

**Example:**
```rust
assert!(x_decimals >= 7, "X coordinate should have at least 7 decimals, got {}", x_decimals);
```

**Reference:** [REF-TEST-003]

#### Equality Assertions

```rust
assert_eq!(actual, expected, "Error message")
```

**Example:**
```rust
assert_eq!(index, 0, "Should match the first line");
```

**Reference:** [REF-TEST-004]

#### Inequality Assertions

```rust
assert_ne!(actual, not_expected, "Error message")
```

**Example:**
```rust
assert_ne!(distance, Decimal::ZERO, "Distance should not be zero");
```

**Reference:** [REF-TEST-005]

## Test Suite: correlation_tests.rs

### Test 1: Decimal Precision Preservation

**Test Name:** `test_decimal_precision_preserved`

**Pass Criteria:**
- X coordinate has ≥ 7 decimal places
- Y coordinate has ≥ 7 decimal places

**Pass Token:**
```rust
assert!(x_decimals >= 7, "X coordinate should have at least 7 decimals, got {}", x_decimals);
assert!(y_decimals >= 7, "Y coordinate should have at least 7 decimals, got {}", y_decimals);
```

**Why:** Ensures coordinate precision is not lost during operations.

**Reference:** [REF-TEST-006]

### Test 2: Exact Match - Distance Zero

**Test Name:** `test_exact_match_distance_zero`

**Pass Criteria:**
- One result returned
- Result is not None
- Matched line index is 0
- Distance equals exactly 0

**Pass Token:**
```rust
assert_eq!(results.len(), 1, "Should return 1 result");
assert!(result.is_some(), "Result should not be None");
assert_eq!(index, 0, "Should match the first line");
assert_eq!(distance, decimal("0"), "Distance should be 0 for exact match");
```

**Why:** Validates perfect coordinate matching.

**Reference:** [REF-TEST-007]

### Test 3: Within Threshold

**Test Name:** `test_within_threshold`

**Pass Criteria:**
- Distance < 0.001 degrees
- Address marked as relevant

**Pass Token:**
```rust
assert!(distance < threshold, "Distance {} should be less than threshold {}", distance, threshold);
```

**Why:** Confirms addresses near parking zones are accepted.

**Reference:** [REF-TEST-008]

### Test 4: Outside Threshold - Rejection

**Test Name:** `test_outside_threshold`

**Pass Criteria:**
- Distance > 0.001 degrees
- Address marked as not relevant

**Pass Token:**
```rust
assert!(distance > threshold, "Distance {} should be greater than threshold {}", distance, threshold);
```

**Why:** Confirms far addresses are rejected.

**Reference:** [REF-TEST-009]

### Test 5: Multiple Lines - Closest Selected

**Test Name:** `test_multiple_lines_closest_selected`

**Pass Criteria:**
- Three parking zones provided
- Middle zone (index 1) is closest
- Correct zone selected

**Pass Token:**
```rust
assert_eq!(index, 1, "Should select line index 1 (the closest zone), got {}", index);
```

**Why:** Ensures algorithm picks truly closest zone.

**Reference:** [REF-TEST-010]

### Test 6: Correlation Output Structure

**Test Name:** `test_correlation_output_structure`

**Pass Criteria:**
- One result returned
- `relevant` field is true
- Postal code matches (202)
- Address fields match original
- Zone metadata (info, tid, dag) transferred correctly

**Pass Token:**
```rust
assert_eq!(results.len(), 1, "Should have 1 correlation result");
assert!(result.relevant, "Should be marked as relevant (within threshold)");
assert_eq!(result.postnummer, 202, "Postal code should match");
assert_eq!(result.adress, "Storgatan 15", "Address should match");
assert_eq!(result.info, "Parking Zone A", "Parking info should match the linked zone");
```

**Why:** Validates output structure integrity.

**Reference:** [REF-TEST-011]

### Test 7: Multiple Addresses - Batch Processing

**Test Name:** `test_multiple_addresses_correlation`

**Pass Criteria:**
- Three addresses processed
- Three results returned
- First two marked relevant
- Third marked not relevant (too far)
- Correct zone associations

**Pass Token:**
```rust
assert_eq!(results.len(), 3, "Should have 3 correlation results");
assert!(results[0].relevant, "Storgatan 1 should be relevant");
assert!(results[1].relevant, "Lilla Torg 5 should be relevant");
assert!(!results[2].relevant, "Västra Varvsgatan 10 should NOT be relevant");
assert_eq!(results[0].info, "Zone A", "First result should be Zone A");
assert_eq!(results[1].info, "Zone B", "Second result should be Zone B");
```

**Why:** Validates batch processing and selective relevance marking.

**Reference:** [REF-TEST-012]

### Test 8: Degenerate Line Segment

**Test Name:** `test_degenerate_line_segment`

**Pass Criteria:**
- Result is not None (handled gracefully)
- Distance > 0 (computes distance to point)

**Pass Token:**
```rust
assert!(results[0].is_some(), "Should handle degenerate segment");
assert!(results[0].unwrap().1 > decimal("0"));
```

**Why:** Ensures edge case doesn't cause panics or NaN values.

**Reference:** [REF-TEST-013]

### Test 9: Threshold Calibration

**Test Name:** `test_threshold_calibration_values`

**Pass Criteria:**
- Distance compared against multiple thresholds
- Expected pass/fail results match actual

**Pass Token:**
```rust
let test_thresholds = [("0.00001", false), ("0.0001", false), ("0.001", true), ...];
for (threshold_str, should_pass) in &test_thresholds {
    let threshold = decimal(threshold_str);
    let is_within = distance < threshold;
    assert_eq!(is_within, *should_pass, "Distance {} vs threshold {} failed", distance, threshold);
}
```

**Why:** Confirms threshold boundary behavior across range of values.

**Reference:** [REF-TEST-014]

### Test 10: Real-world Malmö Coordinates

**Test Name:** `test_real_world_malmo_coordinates`

**Pass Criteria:**
- Two real Malmö addresses processed
- Both marked relevant
- Correct zone associations
- "Lilla Torg" → "Lilla Torg Miljözon"
- "Västra Varvsgatan" → "Västra Varvsgatan Miljözon"

**Pass Token:**
```rust
assert!(result.relevant, "Real-world Malmö address should be relevant: {}", result.adress);
assert_eq!(results[0].info, "Lilla Torg Miljözon", "Lilla Torg should match with correct zone");
assert_eq!(results[1].info, "Västra Varvsgatan Miljözon", "Västra Varvsgatan should match");
```

**Why:** Validates algorithm works with actual city data.

**Reference:** [REF-TEST-015]

### Test 11: Precision Loss Detection

**Test Name:** `test_no_precision_loss_in_calculations`

**Pass Criteria:**
- Very high-precision coordinates maintained
- Calculated distance < 0.0001 degrees

**Pass Token:**
```rust
assert!(distance < decimal("0.0001"), "Precision test distance should be near zero, got {}", distance);
```

**Why:** Ensures Decimal type prevents f64 rounding errors.

**Reference:** [REF-TEST-016]

### Test 12: Performance - Batch Processing

**Test Name:** `test_batch_performance_many_records`

**Pass Criteria:**
- 100 addresses + 50 zones processed successfully
- Correct number of results returned (100)
- At least some results marked relevant

**Pass Token:**
```rust
assert_eq!(results.len(), 100, "Should return 100 results");
assert!(results.iter().filter(|r| r.relevant).count() > 0, "At least some addresses should be relevant");
```

**Why:** Ensures algorithm scales without crashes or timeouts.

**Reference:** [REF-TEST-017]

## Running Tests

### All Tests
```bash
cargo test --release -p amp_core
```

### Specific Test
```bash
cargo test --release correlation_tests::tests::test_decimal_precision_preserved
```

### With Output
```bash
cargo test --release -- --nocapture
```

**Reference:** [REF-TEST-018]

## Test Results Interpretation

### Pass Example
```
test correlation_tests::tests::test_exact_match_distance_zero ... ok
```

**Meaning:** All assertions passed, condition met.

### Fail Example
```
test correlation_tests::tests::test_outside_threshold ... FAILED

thread 'correlation_tests::tests::test_outside_threshold' panicked at
'assertion failed: distance > threshold, Distance 0.0005 should be greater than 0.001'
```

**Meaning:** One assertion failed with explicit error message.

**Reference:** [REF-TEST-019]

## Pass/Not Token Best Practices

1. **Always provide message:** `assert!(..., "descriptive message")`
2. **Include actual values:** Show what failed vs what expected
3. **Use format strings:** `assert_eq!(a, b, "Expected {} but got {}", b, a)`
4. **Be specific:** Multiple assertions > single complex condition
5. **Test edge cases:** Boundaries, zero, None, empty values

**Reference:** [REF-TEST-020]
