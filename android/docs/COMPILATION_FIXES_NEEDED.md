# Remaining Compilation Fixes

## Status: In Progress

The following files need updates to compile with the new DB struct:

### 1. android/src/geo.rs
**Issue**: References `entry.coordinates` which doesn't exist on DB struct
**Fix**: DB struct doesn't have coordinates field. Need to either:
- Add coordinates field to DB struct, OR
- Remove geo.rs if not needed, OR
- Update geo.rs to work without coordinates

### 2. android/src/ui/mod.rs
**Issue**: Multiple problems:
- Imports `StaticAddressEntry` which no longer exists
- Uses `entry.gata.to_lowercase()` but gata is `Option<String>`
- Uses `entry.gatunummer.to_lowercase()` but gatunummer is `Option<String>`
- Compares `entry.postnummer == postal_code_trimmed` with wrong types

**Fix**: Update to use DB struct properly:
```rust
// OLD:
use crate::static_data::StaticAddressEntry;
let entry_street_lower = entry.gata.to_lowercase();

// NEW:
use amp_core::structs::DB;
let entry_street_lower = entry.gata
    .as_ref()
    .map(|s| s.to_lowercase())
    .unwrap_or_default();
```

### 3. android/src/ui/panels.rs
**Issue**: Calls countdown functions with old signature:
- `format_countdown(e.dag, &e.tid)` should be `format_countdown(e)`
- `bucket_for(entry.dag, &entry.tid)` should be `bucket_for(entry)`

**Fix**: Update function calls to pass `&DB` instead of dag/tid

### 4. android/src/storage.rs (tests)
**Issue**: `StoredAddress` struct is missing fields: `id`, `matched_entry`, `valid`

**Fix**: Update test structs to include all required fields or check ui/mod.rs for StoredAddress definition

### 5. android/src/android_bridge.rs
**Issue**: Unused import `std::sync::OnceLock`
**Fix**: Remove unused import

### 6. android/src/notifications.rs
**Issue**: Unused import `std::sync::OnceLock`
**Fix**: Remove unused import

## Quick Fix Script

Run these commands to see detailed errors:
```bash
cd android
cargo check 2>&1 | tee ../compile_errors.txt
```

## Priority Order

1. Fix ui/mod.rs (blocks most UI functionality)
2. Fix ui/panels.rs (depends on #1)
3. Fix geo.rs (decide on coordinates)
4. Fix storage.rs tests
5. Remove unused imports

## Notes

The DB struct currently doesn't have a `coordinates` field. This needs to be decided:
- If geo calculations are needed, add coordinates to DB
- If not, remove/stub geo.rs functions
