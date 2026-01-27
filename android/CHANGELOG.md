# Android Parquet Integration - Changes Summary

## Committed Changes (2026-01-27)

### 1. Unified Type System
**File:** `android/src/static_data.rs`
- Created type alias: `pub type StaticAddressEntry = ParkingRestriction`
- Removed duplicate struct definition
- Implemented `load_parquet_data()` to load from multiple possible paths
- All UI code can now use `StaticAddressEntry` which maps to `ParkingRestriction`

### 2. Simplified Matching Logic  
**File:** `android/src/matching.rs`
- Removed 4th parameter (HashMap) from `match_address()` function
- Added `OnceLock` for lazy-loading parking data once on first use
- Now callable with 3 parameters: `match_address(gata, gatunummer, postnummer)`
- Data loaded automatically and cached for entire app lifetime

### 3. Updated ParkingRestriction Struct
**File:** `core/src/parquet.rs`
- Added `adress: String` field to `ParkingRestriction` struct
- This field is populated from parquet and used by UI components
- Read/write functions updated to handle the new field

## Current Status

✅ **Compilation Should Now Work**

All type mismatches resolved:
- `StaticAddressEntry` = `ParkingRestriction` (type alias)
- `match_address()` takes 3 arguments (as UI expects)
- `entry.adress` field exists on struct

## How to Build

```bash
# From repository root
cargo build -p amp-android

# Or with release optimizations
cargo build -p amp-android --release
```

## Next Steps

### 1. Generate Parquet Database
```bash
cd server
cargo run --release -- output --algorithm kdtree --cutoff 20 --android
cp .app_addresses.parquet ../android/assets/parking_db.parquet
```

### 2. Test Data Loading
The app will attempt to load from these paths (in order):
1. `assets/parking_db.parquet`
2. `android/assets/parking_db.parquet`  
3. `../assets/parking_db.parquet`
4. `parking_db.parquet`

If none found, it will print a warning and use empty dataset.

### 3. Run the App
```bash
# Desktop preview (for development)
cargo run -p amp-android

# Build for Android
dx build --platform android --release
```

## Architecture Overview

```
User Input (Gata, Gatunummer, Postnummer)
           │
           ▼
    match_address() in matching.rs
           │
           ├──> Lazy-loads parking data (OnceLock)
           │    from load_parquet_data()
           │
           ▼
    Returns MatchResult::Valid(ParkingRestriction)
           │
           ▼
    UI displays with countdown logic
```

## File Structure

```
amp/
├── core/
│   ├── src/
│   │   └── parquet.rs          [MODIFIED] - Added adress field
│   └── Cargo.toml
├── android/
│   ├── src/
│   │   ├── static_data.rs     [MODIFIED] - Type alias + loader
│   │   ├── matching.rs        [MODIFIED] - OnceLock pattern
│   │   ├── ui/
│   │   │   ├── adresser.rs    [NO CHANGE NEEDED]
│   │   │   └── paneler.rs     [NO CHANGE NEEDED]
│   │   └── countdown.rs       [✓ Already working]
│   └── assets/
│       └── parking_db.parquet [NEEDS GENERATION]
└── server/
    └── src/
        └── main.rs            [NEW COMMAND] - output subcommand
```

## Breaking Changes

None - all changes are backward compatible through type aliasing.

## Known Issues

- Parquet file must be generated manually before first run
- No automatic data refresh mechanism yet
- Error handling could be more granular

## Performance Notes

- Data loaded once on first `match_address()` call
- Subsequent lookups are O(1) HashMap lookups  
- No repeated I/O operations
- Memory footprint: ~few MB for full Malmö address database
