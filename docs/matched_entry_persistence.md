# Matched Entry Persistence Fix

## Problem

After reopening the app, addresses that were correctly matched to parking zones at insert time would show as "valid" but with no matched parking data visible in the UI.

### Symptoms

- Address is marked as `valid: true` in the UI
- But the parking zone details (taxa, antal_platser, typ_av_parkering, time restrictions) are missing
- The `matched_entry` field in `StoredAddress` is `None` after loading from storage
- User sees "no data" even though the address was successfully matched when originally added

## Root Cause

The persistence layer was not properly saving and restoring the `matched_entry` field:

### 1. Incomplete Serialization (`to_local_data`)

**Before the fix:**
```rust
fn to_local_data(addr: &StoredAddress) -> LocalData {
    let (dag, tid, info, taxa, antal_platser, typ_av_parkering) =
        if let Some(ref entry) = addr.matched_entry {
            (
                None,  // ❌ dag was always None
                None,  // ❌ tid was always None
                entry.info.clone(),
                entry.taxa.clone(),
                entry.antal_platser,
                entry.typ_av_parkering.clone(),
            )
        } else {
            (None, None, None, None, None, None)
        };
    // ...
}
```

The function was:
- ✅ Extracting `info`, `taxa`, `antal_platser`, `typ_av_parkering`
- ❌ **Not extracting `tid` and `dag`** from the `matched_entry.start_time` and `matched_entry.end_time` timestamps

### 2. Wrong Deserialization Strategy (`from_local_data`)

**Before the fix:**
```rust
fn from_local_data(data: LocalData, id: usize) -> StoredAddress {
    // Extract address fields...
    
    // ❌ Problem: Re-running matching instead of using persisted data
    let match_result = match_address(&street, &street_number, &postal_code);
    
    match match_result {
        MatchResult::Valid(db_entry) => {
            stored_address.matched_entry = Some(*db_entry);
        }
        MatchResult::Invalid(err) => {
            // Match failed! matched_entry stays None
            stored_address.matched_entry = None;
        }
    }
    // ...
}
```

The function was:
- ❌ **Ignoring persisted match data** in `LocalData`
- ❌ **Re-running the match algorithm** via `match_address()`
- ❌ Re-matching often failed because:
  - Lookup keys might have changed
  - Database structure assumptions might differ
  - The persisted address format might not match database keys exactly
- Result: `matched_entry` becomes `None` even though the data was in the parquet file

## The Fix

### 1. Complete Serialization

**After the fix:**
```rust
fn to_local_data(addr: &StoredAddress) -> LocalData {
    let (dag, tid, info, taxa, antal_platser, typ_av_parkering) =
        if let Some(ref entry) = addr.matched_entry {
            // Extract day and time from the DB entry's timestamps
            let start_swedish = entry.start_time_swedish();
            let end_swedish = entry.end_time_swedish();
            
            let dag_value = Some(start_swedish.day() as u8);
            let tid_value = Some(format!(
                "{:02}{:02}-{:02}{:02}",
                start_swedish.hour(),
                start_swedish.minute(),
                end_swedish.hour(),
                end_swedish.minute()
            ));

            (
                dag_value,    // ✅ Now extracted
                tid_value,    // ✅ Now extracted
                entry.info.clone(),
                entry.taxa.clone(),
                entry.antal_platser,
                entry.typ_av_parkering.clone(),
            )
        } else {
            (None, None, None, None, None, None)
        };
    // ...
}
```

Now **all** match data is persisted to `LocalData`:
- ✅ `tid` - time range formatted as "HHMM-HHMM" (e.g., "0800-1200")
- ✅ `dag` - day of month as u8
- ✅ `taxa` - parking zone (e.g., "Taxa C")
- ✅ `antal_platser` - number of parking spots
- ✅ `typ_av_parkering` - parking type (e.g., "Längsgående 6")
- ✅ `info` - environmental info (e.g., "Parkering förbjuden")

### 2. Reconstruction from Persisted Data

**After the fix:**
```rust
fn from_local_data(data: LocalData, id: usize) -> StoredAddress {
    // Extract address fields...
    
    // ✅ Reconstruct matched_entry from persisted data
    let matched_entry = if let (
        Some(ref tid),
        Some(dag),
    ) = (&data.tid, data.dag)
    {
        // Reconstruct DB entry from persisted fields
        use chrono::{Datelike, Utc};
        let now = Utc::now();
        let current_year = now.year();
        let current_month = now.month();

        match DB::from_params(DBParams {
            postnummer: data.postnummer.clone(),
            adress: format!("{} {}", street, street_number),
            gata: Some(street.clone()),
            gatunummer: Some(street_number.clone()),
            info: data.info.clone(),
            dag,
            tid: tid.clone(),
            taxa: data.taxa.clone(),
            antal_platser: data.antal_platser,
            typ_av_parkering: data.typ_av_parkering.clone(),
            year: current_year,
            month: current_month,
        }) {
            Some(db_entry) => {
                // ✅ Successfully reconstructed from persisted data!
                Some(db_entry)
            }
            None => None,
        }
    } else if data.valid {
        // ✅ Fallback: only re-match if persisted data is missing
        //    (handles legacy data saved before the fix)
        match match_address(&street, &street_number, &postal_code) {
            MatchResult::Valid(db_entry) => Some(*db_entry),
            MatchResult::Invalid(_) => None,
        }
    } else {
        None
    };
    // ...
}
```

Key improvements:
1. ✅ **Primary strategy: Reconstruct from persisted data**
   - If `tid` and `dag` are present in `LocalData`, build a `DB` entry using `DBParams`
   - This preserves the exact match that was computed at insert time
   
2. ✅ **Fallback strategy: Re-match only if necessary**
   - Only runs if persisted data is missing (legacy data)
   - Handles backward compatibility with data saved before the fix
   
3. ✅ **No unnecessary re-matching**
   - Trusts the persisted data
   - Avoids match failures due to lookup key mismatches

## Result

After the fix:
- ✅ Addresses loaded from storage retain their `matched_entry` with full parking zone info
- ✅ UI correctly displays parking details (taxa, antal_platser, typ_av_parkering, time restrictions)
- ✅ No "valid but no data" state
- ✅ Backward compatible: legacy data without `tid`/`dag` still attempts re-matching

## Testing

Three new tests were added to verify the fix:

### 1. `test_matched_entry_persistence`
Verifies that a `StoredAddress` with a `matched_entry` survives a save/load cycle:
```rust
// Create address with matched DB entry
let original = StoredAddress {
    matched_entry: Some(db_entry.clone()),
    // ...
};

// Save and load
write_addresses_to_device(&[original]);
let loaded = read_addresses_from_device();

// Verify matched_entry is restored
assert!(loaded[0].matched_entry.is_some());
assert_eq!(original.taxa, loaded[0].matched_entry.taxa);
```

### 2. `test_mixed_matched_entries`
Verifies that multiple addresses with different match states persist correctly:
- Address with match data → `matched_entry` restored
- Address without match data → `matched_entry` stays None

### 3. Existing tests extended
Existing roundtrip tests now also verify match data preservation.

## Impact on Data Structure

No changes to `LocalData` schema were required! The fix uses existing fields:

```rust
pub struct LocalData {
    // Existing fields already supported all necessary data:
    pub tid: Option<String>,           // ✅ Now populated
    pub dag: Option<u8>,               // ✅ Now populated
    pub taxa: Option<String>,          // ✅ Already working
    pub antal_platser: Option<u64>,    // ✅ Already working
    pub typ_av_parkering: Option<String>, // ✅ Already working
    pub info: Option<String>,          // ✅ Already working
    // ...
}
```

This means:
- ✅ Backward compatible: old parquet files still load (fallback to re-matching)
- ✅ Forward compatible: new parquet files have complete match data
- ✅ No schema migration needed

## Files Changed

- `android/src/components/storage.rs`:
  - `to_local_data()`: Now extracts `tid` and `dag` from `matched_entry` timestamps
  - `from_local_data()`: Reconstructs `matched_entry` from persisted `LocalData` fields
  - Added comprehensive tests for match data persistence

## Commit

SHA: `c9a0a5b31383fe08427a27dfc984349192fa16ec`

Branch: `feature/android`
