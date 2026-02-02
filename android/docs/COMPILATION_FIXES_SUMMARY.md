# Compilation Fixes Summary

Completed: February 2, 2026

## Overview

This document summarizes all compilation errors fixed during the Android crate refactoring to use the new `DB` struct from `amp-core`.

## Fixes Applied

### 1. Core Struct Improvements

**File:** `core/src/structs.rs`

**Issue:** Clippy warning about too many arguments (12/7) in `from_dag_tid` function

**Solution:** 
- Created `DBParams` struct to group function parameters
- Added new `from_params()` method as preferred interface
- Kept `from_dag_tid()` as legacy wrapper with `#[allow(clippy::too_many_arguments)]`
- Added `Datelike` import from chrono for `year()` and `month()` methods

**Commits:**
- `fix: refactor from_dag_tid to use DBParams struct`

### 2. Android Bridge & Notifications

**Files:** 
- `android/src/android_bridge.rs`
- `android/src/notifications.rs`

**Issue:** Unused import warnings for `std::sync::OnceLock`

**Solution:** 
- Moved `OnceLock` imports inside `#[cfg(target_os = "android")]` blocks
- Only import when actually used on Android target

### 3. Static Data Module

**File:** `android/src/static_data.rs`

**Issue:** Missing `year()` and `month()` methods on `chrono::DateTime<Tz>`

**Solution:**
- Added `use chrono::Datelike;` import
- This provides the `year()` and `month()` trait methods

### 4. Countdown Module

**File:** `android/src/countdown.rs`

**Issue:** `TimeBucket` enum couldn't be used as HashMap key

**Solution:**
- Added `Hash` derive to `TimeBucket` enum
- Changed from: `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]`
- To: `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]`

### 5. UI Panels

**File:** `android/src/ui/panels.rs`

**Issue:** Function calls passing old `(dag, tid)` signature instead of new `&DB`

**Solution:**
- Updated `format_countdown(e.dag, &e.tid)` → `format_countdown(e)`
- Updated `bucket_for(entry.dag, &entry.tid)` → `bucket_for(entry)`
- All 5 panel components updated: ActivePanel, SixHoursPanel, OneDayPanel, OneMonthPanel, InvalidPanel

### 6. UI Module

**File:** `android/src/ui/mod.rs`

**Issues:**
1. Import of non-existent `StaticAddressEntry` type
2. Calling `to_lowercase()` on `Option<String>` instead of `String`
3. Type mismatch comparing `Option<String>` with `&str`

**Solutions:**
- Changed import from `StaticAddressEntry` to `amp_core::structs::DB`
- Updated `StoredAddress` to use `matched_entry: Option<DB>`
- Fixed Option handling:
  ```rust
  // Before:
  let entry_street_lower = entry.gata.to_lowercase();
  
  // After:
  let entry_street_lower = entry.gata
      .as_ref()
      .map(|s| s.to_lowercase())
      .unwrap_or_default();
  ```
- Fixed postal code comparison:
  ```rust
  // Before:
  let postal_match = entry.postnummer == postal_code_trimmed;
  
  // After:
  let postal_match = entry.postnummer
      .as_ref()
      .map(|pn| pn.as_str() == postal_code_trimmed)
      .unwrap_or(false);
  ```

### 7. Geo Module

**File:** `android/src/geo.rs`

**Issues:**
1. Import of non-existent `StaticAddressEntry`
2. Accessing non-existent `coordinates` field on `DB` struct

**Solution:**
- Removed `StaticAddressEntry` import, use `amp_core::structs::DB` instead
- Stubbed coordinate-based functions with TODO comments
- Added clear documentation that these need to use `read_address_parquet`
- Functions return empty results until address parquet integration is complete
- Kept `haversine_distance` function fully functional as utility

### 8. Storage Module Tests

**File:** `android/src/storage.rs`

**Issue:** Test structs missing required fields `id`, `matched_entry`, `valid`

**Solution:**
- Updated all test `StoredAddress` initialization to include all fields:
  ```rust
  StoredAddress {
      id: 1,
      street: "Storgatan".to_string(),
      street_number: "10".to_string(),
      postal_code: "22100".to_string(),
      valid: true,
      active: true,
      matched_entry: None,
  }
  ```

## Compilation Status

**Before fixes:**
- 20 compilation errors
- 4 warnings
- Multiple clippy violations

**After fixes:**
- ✅ 0 compilation errors
- ⚠️ 1 remaining clippy warning (suppressed with allow attribute)
- All tests compile successfully

## Remaining TODOs

### 1. Geo Module - Address Parquet Integration

**Priority:** Medium

**Description:** 
The geo module currently stubs coordinate-based address lookup. Need to:
- Integrate `read_address_parquet` function
- Load coordinate data from address parquet file (not parking restrictions)
- Implement `find_address_by_coordinates()`
- Implement `find_addresses_within_radius()`

**Location:** `android/src/geo.rs`

### 2. Test Data

**Priority:** Low

**Description:**
Some modules may benefit from actual test data files for integration testing.

## Migration Notes

For developers working with the codebase:

### Using DB Struct

**Old way (deprecated but still works):**
```rust
let db = DB::from_dag_tid(
    Some("22100".to_string()),
    "Storgatan 10".to_string(),
    Some("Storgatan".to_string()),
    Some("10".to_string()),
    Some("Info".to_string()),
    15,
    "0800-1200",
    Some("Taxa C".to_string()),
    Some(5),
    Some("Längsgående".to_string()),
    2024,
    1,
);
```

**New way (preferred):**
```rust
use amp_core::structs::DBParams;

let db = DB::from_params(DBParams {
    postnummer: Some("22100".to_string()),
    adress: "Storgatan 10".to_string(),
    gata: Some("Storgatan".to_string()),
    gatunummer: Some("10".to_string()),
    info: Some("Info".to_string()),
    dag: 15,
    tid: "0800-1200".to_string(),
    taxa: Some("Taxa C".to_string()),
    antal_platser: Some(5),
    typ_av_parkering: Some("Längsgående".to_string()),
    year: 2024,
    month: 1,
});
```

### Countdown Functions

All countdown functions now take `&DB` instead of separate `dag` and `tid`:

```rust
// Old:
format_countdown(entry.dag, &entry.tid)
bucket_for(entry.dag, &entry.tid)

// New:
format_countdown(entry)
bucket_for(entry)
```

### Option Handling

When working with Option fields on DB:

```rust
// String operations on Option<String>:
let street_lower = db.gata
    .as_ref()
    .map(|s| s.to_lowercase())
    .unwrap_or_default();

// Comparisons with Option<String>:
let matches = db.postnummer
    .as_ref()
    .map(|pn| pn.as_str() == target)
    .unwrap_or(false);
```

## Testing

All changes have been tested by:
1. Running `cargo check` on all crates
2. Running `cargo test` on core and android crates  
3. Running `cargo clippy` with fix mode
4. Verifying no new warnings introduced

## Commits

Total commits for this fix session: 3 major commits

1. `fix: resolve compilation errors across android crate` - Main UI and function signature fixes
2. `fix: update geo.rs and storage.rs for DB struct` - Geo stubbing and storage test fixes
3. `fix: refactor from_dag_tid to use DBParams struct` - Clippy warning resolution

## Documentation Updates

- ✅ All fixed modules have updated doc comments
- ✅ TODO comments added for stubbed functionality
- ✅ Migration guide included in this document
- ✅ Function signatures documented with examples

## Conclusion

All compilation errors have been successfully resolved. The codebase now uses the new `DB` struct consistently throughout the Android crate, with proper timestamp handling via chrono `DateTime<Utc>`.

The geo module has been intentionally stubbed pending address parquet integration, which is tracked as a separate TODO item.
