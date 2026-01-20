# AMP Complete Reference Guide

## Quick Navigation

- **API Integration:** [docs/API_ARCHITECTURE.md](./API_ARCHITECTURE.md)
- **Algorithm Details:** [docs/CORRELATION_ALGORITHM.md](./CORRELATION_ALGORITHM.md)
- **Testing & QA:** [docs/TEST_STRATEGY.md](./TEST_STRATEGY.md)
- **Module Overview:** [core/README.md](../core/README.md)

---

## Documentation Structure

```
amps/
├─ docs/
│  ├─ API_ARCHITECTURE.md          # API integration & data sources
│  ├─ CORRELATION_ALGORITHM.md    # Geographic matching algorithm
│  ├─ TEST_STRATEGY.md            # Testing framework & pass/not tokens
│  ├─ REFERENCE_GUIDE.md          # This file - overview & index
│  └─ README.md
├─ core/
│  ├─ README.md                   # Core module guide with reference keys
│  ├─ src/
│  │  ├─ api.rs                   # ArcGIS client [REF-API-*]
│  │  ├─ correlation.rs           # Distance algorithm [REF-CORR-*]
│  │  ├─ correlation_tests.rs     # Test suite [REF-TEST-*]
│  │  ├─ structs.rs
│  │  ├─ parquet.rs
│  │  ├─ error.rs
│  │  └─ lib.rs
```

---

## API Reference Index

### [docs/API_ARCHITECTURE.md](./API_ARCHITECTURE.md)

Documentation for `core/src/api.rs`

| Reference | Topic | Content |
|-----------|-------|----------|
| REF-API-001 | Overview | ArcGIS client purpose and features |
| REF-API-002 | Client Structure | ArcGISClient struct definition |
| REF-API-003 | Pagination | Batch-based fetching strategy |
| REF-API-004 | Point Extraction | GeoJSON coordinate parsing |
| REF-API-005 | Polygon Extraction | Bounding box from polygon rings |
| REF-API-006 | Address Transform | ArcGIS → AdressClean mapping |
| REF-API-007 | Zone Transform | ArcGIS → MiljoeDataClean mapping |
| REF-API-008 | Address Source | Malmö addresses ArcGIS service |
| REF-API-009 | Parking Source | Environmental parking ArcGIS service |
| REF-API-010 | Error Handling | Resilience patterns |
| REF-API-011 | Pagination Perf | Memory and performance benefits |
| REF-API-012 | Async/Await | Tokio runtime setup |
| REF-API-013 | Integration | Main api() entry point |
| REF-API-014 | Field Mapping | Triple-fallback compatibility pattern |

**Use these references in code:**
```rust
// See [REF-API-003] for pagination strategy
// See [REF-API-006] for field mapping
```

---

## Correlation Algorithm Reference Index

### [docs/CORRELATION_ALGORITHM.md](./CORRELATION_ALGORITHM.md)

Documentation for `core/src/correlation.rs`

| Reference | Topic | Content |
|-----------|-------|----------|
| REF-CORR-001 | Overview | Algorithm purpose and flow |
| REF-CORR-002 | Algorithm Flow | Correlation pipeline steps |
| REF-CORR-003 | Distance Func | Point-to-line function signature |
| REF-CORR-004 | Math Steps | Vector projection calculations |
| REF-CORR-005 | Precision | Decimal → f64 → Decimal handling |
| REF-CORR-006 | Threshold | 0.001 degrees justification |
| REF-CORR-007 | Threshold App | How threshold filters results |
| REF-CORR-008 | Result Struct | AdressInfo fields and meaning |
| REF-CORR-009 | Parallelization | Rayon par_iter usage |
| REF-CORR-010 | Complexity | Time/space complexity analysis |
| REF-CORR-011 | Degenerate Segs | Handling of identical endpoints |
| REF-CORR-012 | No Lines Case | Handling when no zones available |
| REF-CORR-013 | No Correlation | When distance exceeds threshold |
| REF-CORR-014 | Test Coverage | 12-test suite overview |
| REF-CORR-015 | Pass/Not Token | Assertion system |

**Use these references in code:**
```rust
// See [REF-CORR-004] for vector projection math
// See [REF-CORR-006] for why 0.001 degree threshold
```

---

## Test Strategy Reference Index

### [docs/TEST_STRATEGY.md](./TEST_STRATEGY.md)

Documentation for `core/src/correlation_tests.rs`

| Reference | Topic | Content |
|-----------|-------|----------|
| REF-TEST-001 | Overview | Pass/not token system intro |
| REF-TEST-002 | Definition | What is a pass/not token |
| REF-TEST-003 | Boolean Assert | assert!() usage |
| REF-TEST-004 | Equality Assert | assert_eq!() usage |
| REF-TEST-005 | Inequality Assert | assert_ne!() usage |
| REF-TEST-006 | Test 1 | Precision preservation |
| REF-TEST-007 | Test 2 | Exact match (distance = 0) |
| REF-TEST-008 | Test 3 | Within threshold acceptance |
| REF-TEST-009 | Test 4 | Outside threshold rejection |
| REF-TEST-010 | Test 5 | Multiple zones - closest selected |
| REF-TEST-011 | Test 6 | Output structure validation |
| REF-TEST-012 | Test 7 | Batch processing (100+ addresses) |
| REF-TEST-013 | Test 8 | Degenerate line segments |
| REF-TEST-014 | Test 9 | Threshold calibration across values |
| REF-TEST-015 | Test 10 | Real Malmö coordinates |
| REF-TEST-016 | Test 11 | Precision loss detection |
| REF-TEST-017 | Test 12 | Performance (100 + 50) |
| REF-TEST-018 | Running Tests | cargo test commands |
| REF-TEST-019 | Result Interpretation | Pass/fail output formats |
| REF-TEST-020 | Best Practices | Assertion guidelines |

**Use these references in tests:**
```rust
// See [REF-TEST-006] for precision test approach
// See [REF-TEST-020] for assertion best practices
```

---

## How to Use This Reference System

### When Reading Code

1. **See a function?** Look for its module in docs/
2. **Want to understand flow?** Check the reference keys in the README
3. **Need detailed explanation?** Search for [REF-XXX-YYY] in corresponding doc

### Example: Understanding correlation() function

```rust
pub fn correlation(points: Vec<AdressClean>, lines: Vec<MiljoeDataClean>) -> Vec<AdressInfo> {
    // See [REF-CORR-002] for algorithm flow
    // See [REF-CORR-007] for threshold filtering
    // ...
}
```

**To understand:**
1. Open [docs/CORRELATION_ALGORITHM.md](./CORRELATION_ALGORITHM.md)
2. Find "Algorithm Flow: `correlation()`" section
3. Read [REF-CORR-002] details
4. Then read [REF-CORR-007] for threshold logic

### When Writing Code

1. **Remove inline comments**
2. **Add reference key comment** like `// See [REF-CORR-006]`
3. **Add/update docs/** entry with the reference
4. **Link from core/README.md**

---

## Key Concepts Cross-Reference

### Precision & Coordinates

- **Why Decimal type?** [REF-CORR-005]
- **7+ decimal places explained** [REF-API-004]
- **Precision loss detection** [REF-TEST-016]
- **Real-world testing** [REF-TEST-015]

### Distance Calculation

- **Algorithm overview** [REF-CORR-002]
- **Mathematical steps** [REF-CORR-004]
- **Edge cases** [REF-CORR-011], [REF-CORR-012]
- **Test coverage** [REF-TEST-007] through [REF-TEST-014]

### Threshold (0.001 degrees)

- **Why this value?** [REF-CORR-006]
- **How applied?** [REF-CORR-007]
- **Testing strategy** [REF-TEST-008], [REF-TEST-009], [REF-TEST-014]

### Data Sources

- **Malmö addresses** [REF-API-008]
- **Environmental parking** [REF-API-009]
- **Fetching strategy** [REF-API-003]
- **Error handling** [REF-API-010]

### Performance

- **Parallelization** [REF-CORR-009]
- **Pagination benefits** [REF-API-011]
- **Batch testing** [REF-TEST-012], [REF-TEST-017]
- **Complexity analysis** [REF-CORR-010]

### Testing

- **Pass/not token system** [REF-TEST-002]
- **12 comprehensive tests** [REF-TEST-006] through [REF-TEST-017]
- **Running tests** [REF-TEST-018]
- **Result interpretation** [REF-TEST-019]
- **Best practices** [REF-TEST-020]

---

## File Organization

### Source Code (`core/src/`)

**api.rs** (~350 lines)
- ArcGISClient struct
- fetch_all_features() with pagination
- GeoJSON extraction (points, polygons)
- Field mapping (3 fallback pattern)
- References: [REF-API-001] through [REF-API-014]

**correlation.rs** (~95 lines)
- correlation() main function
- find_closest_lines() parallel processor
- distance_point_to_line_squared() calculator
- References: [REF-CORR-001] through [REF-CORR-015]

**correlation_tests.rs** (~600 lines)
- 12 comprehensive tests
- Pass/not token assertions
- Coverage: precision, boundaries, real-world
- References: [REF-TEST-001] through [REF-TEST-020]

**structs.rs** - Data structures
- AdressClean
- MiljoeDataClean
- AdressInfo

**parquet.rs** - Data persistence
**error.rs** - Error types
**lib.rs** - Module exports

### Documentation (`docs/`)

**API_ARCHITECTURE.md** - Complete API reference
**CORRELATION_ALGORITHM.md** - Complete algorithm reference
**TEST_STRATEGY.md** - Complete test reference
**REFERENCE_GUIDE.md** - This file

### Module Guides (`*/README.md`)

**core/README.md** - Core library overview with reference index
**android/README.md** - Android integration (future)
**ios/README.md** - iOS integration (future)
**server/README.md** - Server API (future)

---

## Searching for Information

### By Topic

| Topic | Documentation | References |
|-------|---------------|------------|
| Fetching data | API_ARCHITECTURE.md | REF-API-001 to -014 |
| Distance math | CORRELATION_ALGORITHM.md | REF-CORR-003 to -005 |
| Threshold decision | CORRELATION_ALGORITHM.md | REF-CORR-006 to -007 |
| Testing methodology | TEST_STRATEGY.md | REF-TEST-001 to -020 |
| Precision handling | CORRELATION_ALGORITHM.md + TEST_STRATEGY.md | REF-CORR-005, REF-TEST-016 |
| Edge cases | CORRELATION_ALGORITHM.md | REF-CORR-011 to -013 |
| Performance | API_ARCHITECTURE.md + CORRELATION_ALGORITHM.md | REF-API-011, REF-CORR-009, REF-CORR-010 |

### By Component

**api.rs:** REF-API-* references
**correlation.rs:** REF-CORR-* references
**correlation_tests.rs:** REF-TEST-* references

---

## Contributing to Documentation

### Adding New Code

1. Write clean code without inline comments
2. Use reference keys: `// See [REF-XXX-NNN]`
3. Add/update docs/\*.md with detailed explanation
4. Update core/README.md reference index
5. Link from REFERENCE_GUIDE.md if new topic

### Updating Documentation

1. Find relevant docs/ file
2. Locate the [REF-XXX-YYY] section
3. Update explanation
4. Verify all code references still apply
5. Check REFERENCE_GUIDE.md cross-references

### Best Practices

- **One topic per reference** - Keep sections focused
- **Include examples** - Real code snippets
- **Cross-reference** - Link related references
- **Keep references stable** - Don't renumber
- **Version with code** - Update docs when code changes

---

## Quick Links

- **Main Repository:** https://github.com/resonant-jovian/amp
- **Core Module:** `core/`
- **Documentation:** `docs/`
- **Tests:** `core/src/correlation_tests.rs`

---

*Last Updated: 2026-01-20*
*Reference System: REF-XXX-YYY (3 sections, 3 digits)*
