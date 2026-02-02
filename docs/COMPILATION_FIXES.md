# Compilation Fixes for feature/android Branch

## Date: 2026-02-02

## Overview

This document describes the compilation errors encountered when running `./fmt_fix_clippy.sh` and the fixes applied to resolve them.

## Errors Encountered

### Error 1: Type Mismatch in `static_data.rs`

**Location:** `android/src/components/static_data.rs:53`

**Error:**
```
error[E0308]: mismatched types
  --> android/src/components/static_data.rs:53:27
   |
53 |     match read_db_parquet(PARQUET_BYTES) {
   |           --------------- ^^^^^^^^^^^^^ expected `File`, found `&[u8]`
```

**Root Cause:**
- The `read_db_parquet` function in `core/src/parquet.rs` expects a `std::fs::File` parameter
- `PARQUET_BYTES` is `const &[u8]` (embedded bytes from `include_bytes!`)
- Cannot directly convert `&[u8]` to `File`

**Fix:**
- Created a new helper function `read_db_parquet_from_bytes` that accepts `&[u8]`
- Uses `std::io::Cursor` to wrap the byte slice
- Directly reads parquet data using `ParquetRecordBatchReaderBuilder`
- Converts `OutputData` records to `DB` structs using `DB::from_dag_tid`

### Error 2-3: Type Annotations Needed

**Location:** `android/src/components/static_data.rs:62-63`

**Errors:**
```
error[E0282]: type annotations needed
  --> android/src/components/static_data.rs:62:26
   |
62 |                         .as_ref()
   |                          ^^^^^^

error[E0282]: type annotations needed
  --> android/src/components/static_data.rs:63:31
   |
63 |                         .map(|s| s.to_lowercase())
   |                               ^  - type must be known at this point
```

**Root Cause:**
- Original code tried to use `.as_ref()` on `gata` field
- `gata` in `OutputData` is `String` not `Option<String>`
- Type inference failed because the operation doesn't apply to `String`

**Fix:**
- Recognized that `gata` is a required `String` field
- Used direct `.to_lowercase()` call without `as_ref()` or `map()`
- Properly handled the field as a non-optional string

### Error 4: Method Not Found

**Location:** `android/src/components/static_data.rs:65`

**Error:**
```
error[E0599]: no method named `as_deref` found for struct `std::string::String`
  --> android/src/components/static_data.rs:65:39
   |
65 |                     record.gatunummer.as_deref().unwrap_or(""),
   |                                       ^^^^^^^^
```

**Root Cause:**
- `.as_deref()` is a method on `Option<T>` where `T: Deref`
- `gatunummer` in `OutputData` is `String` not `Option<String>`
- Method doesn't exist for plain `String` type

**Fix:**
- Removed the `.as_deref().unwrap_or("")` call
- Used `gatunummer` directly as it's already a `String`
- Wrapped in `Some()` when passing to `DB::from_dag_tid` which expects `Option<String>`

### Error 5: Type Mismatch in Return Value

**Location:** `android/src/components/static_data.rs:74`

**Error:**
```
error[E0308]: mismatched types
  --> android/src/components/static_data.rs:74:13
   |
51 | fn load_parking_data() -> HashMap<String, DB> {
   |                           ------------------- expected `HashMap<String, DB>`
...
68 |                 map.insert(key, record);
   |                 ---             ------ this argument has type `OutputData`
   |                 |
   |                 ... which causes `map` to have type `HashMap<String, OutputData>`
...
74 |             map
   |             ^^^ expected `HashMap<String, DB>`, found `HashMap<String, OutputData>`
```

**Root Cause:**
- Function signature declares return type as `HashMap<String, DB>`
- Code was inserting `OutputData` values into the HashMap
- `OutputData` and `DB` are different struct types
- `OutputData` is the raw parquet data format
- `DB` is the runtime format with parsed timestamps

**Fix:**
- Convert each `OutputData` record to `DB` using `DB::from_dag_tid`
- Only insert successfully converted records into the HashMap
- Handle conversion failures with proper error logging
- Use current year/month for timestamp calculation

### Error 6: Field Not Found

**Location:** `android/src/ui/info_dialog.rs:84`

**Error:**
```
error[E0609]: no field `zon` on type `amp_core::structs::DB`
  --> android/src/ui/info_dialog.rs:84:51
   |
84 |                         if let Some(zone) = entry.zon {
   |                                                   ^^^ unknown field
```

**Root Cause:**
- Code attempted to access `entry.zon` field
- `DB` struct in `core/src/structs.rs` doesn't have a `zon` field
- Available fields: `postnummer`, `adress`, `gata`, `gatunummer`, `info`, `start_time`, `end_time`, `taxa`, `antal_platser`, `typ_av_parkering`
- Likely confusion with similar parking data that may have had zones

**Fix:**
- Removed the check for non-existent `zon` field
- Added display of actual available fields:
  - `taxa` - Parking zone/taxa information (already shown)
  - `info` - Environmental parking restriction info
  - `typ_av_parkering` - Type of parking
  - `antal_platser` - Number of parking spots

## Data Model Understanding

### OutputData (Parquet Format)
```rust
struct OutputData {
    postnummer: Option<String>,
    adress: String,
    gata: String,              // Required, not optional
    gatunummer: String,        // Required, not optional
    info: Option<String>,
    tid: Option<String>,       // Time string like "0800-1200"
    dag: Option<u8>,           // Day of month (1-31)
    taxa: Option<String>,
    antal_platser: Option<u64>,
    typ_av_parkering: Option<String>,
}
```

### DB (Runtime Format)
```rust
struct DB {
    postnummer: Option<String>,
    adress: String,
    gata: Option<String>,
    gatunummer: Option<String>,
    info: Option<String>,
    start_time: DateTime<Utc>,  // Parsed from tid
    end_time: DateTime<Utc>,    // Parsed from tid
    taxa: Option<String>,
    antal_platser: Option<u64>,
    typ_av_parkering: Option<String>,
}
```

### Key Differences
1. **Time Handling:**
   - `OutputData` stores raw time string (`tid: Option<String>`)
   - `DB` stores parsed timestamps (`start_time`, `end_time: DateTime<Utc>`)

2. **Field Nullability:**
   - `OutputData`: `gata` and `gatunummer` are required `String`
   - `DB`: `gata` and `gatunummer` are `Option<String>`

3. **Conversion:**
   - Use `DB::from_dag_tid()` to convert `OutputData` to `DB`
   - Requires year and month parameters for timestamp calculation
   - Returns `Option<DB>` (None if parsing fails)

## Testing

After these fixes, run:

```bash
./scripts/fmt_fix_clippy.sh
```

Expected result: Clean compilation with no errors.

## Related Files Modified

1. `android/src/components/static_data.rs` - Main fixes for parquet reading and type handling
2. `android/src/ui/info_dialog.rs` - Removed non-existent field access
3. `docs/COMPILATION_FIXES.md` - This documentation

## Commit History

1. `fix(android): correct parquet reading and type handling in static_data.rs`
   - Fixed all type mismatches in static_data.rs
   - Added proper OutputData to DB conversion
   - Implemented byte-based parquet reading

2. `fix(android): remove non-existent zon field from info_dialog`
   - Removed access to non-existent DB.zon field
   - Added display of actual DB fields

3. `docs: add compilation fixes documentation`
   - Created this comprehensive documentation

## Notes

- The embedded parquet file (`db.parquet`) must be in OutputData format
- Year/month for timestamp calculation uses current system time
- For production, consider making year/month configurable
- All timestamp operations use Swedish timezone (Europe/Stockholm)
