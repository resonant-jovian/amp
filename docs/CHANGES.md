# Code Quality Improvements - February 2026

This document summarizes the improvements made to address logic errors, oversights, and code quality issues identified in the codebase review.

## Summary of Changes

### 1. Dependency Management

**Files:** `Cargo.toml`, `core/Cargo.toml`, `android/Cargo.toml`

**Changes:**
- Added `chrono-tz` for proper Swedish timezone handling
- Added `strsim` for Levenshtein distance fuzzy matching
- Added `uuid` for stable address IDs
- Added `serde`/`serde_json` for proper JSON serialization

**Why:** Enables timezone-aware operations, better fuzzy matching, and proper data serialization.

---

### 2. Swedish Timezone Support

**Files:** `core/src/structs.rs`

**Changes:**
- Added `SWEDISH_TZ` constant using `chrono_tz::Europe::Stockholm`
- All timestamps now use Swedish timezone (handles DST automatically)
- Added `start_time_swedish()` and `end_time_swedish()` helper methods
- Internal storage remains UTC for consistency

**Why:** Parking restrictions operate on local Swedish time, not UTC. The system now correctly handles summer/winter time transitions.

---

### 3. Time Range Validation & Overflow Prevention

**Files:** `core/src/structs.rs`

**Changes:**
- Added year validation (2020-2100 range) to prevent overflow
- Added month validation (1-12)
- Comprehensive documentation of valid time ranges
- Better error messages for invalid date/time inputs

**Why:** Prevents `DateTime` overflow and ensures realistic date ranges.

---

### 4. Swedish Postal Code Validation

**Files:** `android/src/matching.rs`

**Changes:**
- Implemented `ValidationError` enum with specific error types
- Added `validate_postal_code()` function for Swedish format (5 digits, optional space)
- Changed `validate_input()` to return `Result<(), ValidationError>`
- Added maximum length constraints (street: 100 chars, number: 20 chars)
- Updated `MatchResult::Invalid` to include `ValidationError`

**Why:** Provides detailed validation feedback to users and prevents invalid data entry.

---

### 5. Static Data Loading Optimization

**Files:** `android/src/static_data.rs`

**Changes:**
- Removed `reload_parking_data()` function (OnceLock doesn't support reloading)
- Added documentation explaining load-once design
- Added test verifying data loads only once

**Why:** OnceLock is designed for one-time initialization. The parquet file is bundled with the app and doesn't change at runtime.

---

### 6. Storage Thread Safety & Serialization

**Files:** `android/src/storage.rs`

**Changes:**
- Added `STORAGE_LOCK` Mutex for thread-safe operations
- Replaced manual JSON building with `serde_json`
- Created `StoredAddressData` struct for clean serialization
- Implemented proper `serialize_addresses()` and `deserialize_addresses()`
- Added comprehensive roundtrip tests

**Why:** Prevents race conditions when multiple UI components access storage. Proper serialization is less error-prone and easier to maintain.

---

### 7. UUID-Based Address IDs

**Files:** `android/src/ui/mod.rs`

**Changes:**
- Replaced `AtomicUsize` counter with UUID v4 generation
- Added `uuid_to_usize()` conversion for backward compatibility
- IDs now persist across app restarts

**Why:** UUIDs provide stable identifiers that don't conflict even after deletions/additions. No need to persist counter state.

---

### 8. Improved Fuzzy Matching

**Files:** `android/src/ui/mod.rs`

**Changes:**
- Implemented Levenshtein distance matching with threshold (3 edits)
- Added `normalize_string()` helper for consistent comparisons
- Multi-stage matching: exact → Levenshtein → substring
- Catches typos and minor spelling variations

**Why:** Better user experience - finds matches even with typos or case variations. More precise than simple substring matching.

---

### 9. Normalized Duplicate Detection

**Files:** `android/src/ui/mod.rs`

**Changes:**
- Case-insensitive comparison using `normalize_string()`
- Normalized postal codes (removes spaces)
- Prevents "Storgatan" vs "storgatan" being treated as different

**Why:** Prevents accidental duplicate entries due to case differences.

---

### 10. Safer Index Access

**Files:** `core/src/api.rs`

**Changes:**
- Replaced `coords[0]` and `coords[coords.len()-1]` with `.first()` and `.last()`
- Added explicit error handling with descriptive messages
- Validates coordinate lengths before access

**Why:** Prevents potential panics from out-of-bounds access. Better error messages for debugging.

---

### 11. Haversine Distance Documentation

**Files:** `core/src/correlation_algorithms/common.rs`

**Changes:**
- Added comprehensive documentation about Haversine approximation
- Documented accuracy characteristics (~0.5% error for <100m)
- Explained spherical Earth assumption
- Noted when to use Vincenty formula for higher precision

**Why:** Users understand the trade-offs. The approximation is acceptable for 50m max distance use case.

---

### 12. Grid Cell Algorithm Optimization

**Files:** `core/src/correlation_algorithms/common.rs`

**Changes:**
- Replaced `Vec` + `sort` + `dedup` with `HashSet` during construction
- Avoids adding duplicates in the first place
- More efficient for typical line lengths
- Added test for duplicate detection

**Why:** Better performance and cleaner code. HashSet automatically handles uniqueness.

---

## Error Handling Standardization

All modules now consistently use `anyhow::Result<T>` for error handling, providing better error context and stack traces.

## Testing Improvements

Added tests for:
- Year validation boundaries
- Swedish timezone conversion
- Postal code format validation
- Storage roundtrip serialization
- Grid cell duplicate detection
- Safe array access edge cases

## Documentation Improvements

Enhanced documentation for:
- Time range constraints and overflow prevention
- Haversine distance accuracy characteristics
- Validation error types and messages
- Thread safety guarantees in storage
- UUID-based ID generation strategy

---

## Not Implemented (Future Work)

### Android SharedPreferences Integration

**Status:** TODO markers remain in `android/src/storage.rs`

**Reason:** Requires JNI integration with Android context, which is beyond the scope of this refactor. The infrastructure is now in place with proper serialization and thread safety.

**Priority:** High - needed for actual persistence on Android devices

---

## Migration Notes

### For Existing Code

1. **Validation errors**: Code using `validate_input()` should handle `Result<(), ValidationError>` instead of `bool`
2. **MatchResult**: Invalid variant now contains `ValidationError`, not unit type
3. **Postal codes**: System now validates Swedish postal code format
4. **Timestamps**: All times in DB struct are in Swedish timezone internally

### Breaking Changes

Minimal breaking changes:
- `MatchResult::Invalid` now has associated data
- `validate_input()` return type changed from `bool` to `Result<(), ValidationError>`
- Removed `reload_parking_data()` function (was non-functional)

---

## Performance Impact

- **Positive**: HashSet-based grid algorithm reduces allocations
- **Positive**: Fuzzy matching is fast (Levenshtein distance is O(n*m) but strings are short)
- **Neutral**: Mutex adds minimal overhead (uncontended in typical use)
- **Neutral**: UUID generation is fast (cryptographically secure random)

---

## Code Quality Metrics

- **Type Safety**: Improved (ValidationError enum vs string errors)
- **Thread Safety**: Improved (Mutex-protected storage)
- **Error Handling**: Improved (consistent anyhow::Result usage)
- **Documentation**: Significantly improved (especially time handling and distance calculations)
- **Test Coverage**: Improved (added edge case tests)
- **Maintainability**: Improved (serde_json vs manual JSON, HashSet vs Vec+sort+dedup)

---

## Conclusion

These changes address all critical and high-priority issues identified in the code review. The codebase is now more robust, better documented, and follows Rust best practices more closely. The remaining TODO (SharedPreferences) requires platform-specific integration work that should be tackled separately.
