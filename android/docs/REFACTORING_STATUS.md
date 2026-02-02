# Android Refactoring Status

## Overview

This document tracks the refactoring progress for the Android component of the amp parking app.

**Date**: 2026-02-02
**Branch**: `feature/android`
**Objective**: Clean architecture, proper error handling, comprehensive documentation, and clear TODOs for unimplemented features

## Completed Refactoring

### Core Module

#### ✅ `core/src/structs.rs`
- **Added**: New `DB` struct for Android component
- **Features**:
  - Uses `chrono::DateTime<Utc>` for start_time and end_time
  - Replaces `dag` (day) and `tid` (time string) with proper timestamps
  - Time counted from month start for flexible date calculations
  - Helper methods: `from_dag_tid()`, `is_active()`, `time_until_start()`, `time_until_end()`
  - Comprehensive unit tests
- **Dependencies**: chrono (already in workspace)

### Android Modules

#### ✅ `android/src/android_bridge.rs`
- **Status**: Fully refactored
- **Improvements**:
  - Comprehensive module-level documentation
  - Proper error handling with Result types
  - Clear TODO markers for JNI implementation
  - Platform-specific compilation (Android vs mock)
  - Added JVM initialization function
  - Added device info function stub
  - Unit tests for non-Android behavior
- **TODOs Documented**:
  - LocationManager integration for GPS
  - ActivityCompat permission requests
  - ContextCompat permission checking
  - Device info retrieval using android.os.Build

#### ✅ `android/src/notifications.rs`
- **Status**: Fully refactored
- **Improvements**:
  - Rich module documentation with examples
  - NotificationImportance enum for type-safe importance levels
  - Proper error handling throughout
  - Default channel constants
  - Helper function for channel initialization
  - Atomic notification ID counter
  - Comprehensive unit tests
- **TODOs Documented**:
  - NotificationManager system service integration
  - NotificationCompat.Builder implementation
  - NotificationChannel creation (Android 8.0+)
  - PendingIntent for map actions
  - Channel-specific notifications

#### ✅ `android/src/storage.rs`
- **Status**: Fully refactored
- **Improvements**:
  - Detailed documentation for all public functions
  - SharedPreferences constants (PREFS_NAME, ADDRESSES_KEY)
  - JSON serialization/deserialization functions
  - escape_json utility for safe JSON generation
  - Additional utility functions (clear_all_addresses, count_stored_addresses)
  - Comprehensive unit tests for serialization
- **TODOs Documented**:
  - SharedPreferences read/write implementation
  - JSON deserialization (recommend serde_json)
  - Optimized count without full deserialization

## Remaining Modules to Refactor

### 🚧 High Priority

#### `android/src/matching.rs`
- **Current Status**: Functional but needs enhancement
- **Needed**:
  - Enhanced error messages
  - Add fuzzy matching for addresses
  - Better validation with specific error types
  - Performance optimization for large datasets

#### `android/src/geo.rs`
- **Current Status**: Good base implementation
- **Needed**:
  - Error handling for edge cases
  - Caching for frequently accessed coordinates
  - Batch lookup optimization
  - Integration with DB struct for time-aware results

#### `android/src/countdown.rs`
- **Current Status**: Good implementation with chrono
- **Needed**:
  - **CRITICAL**: Update to use new DB struct with DateTime
  - Remove dag/tid parsing (moved to DB::from_dag_tid)
  - Add recurring event calculation
  - Multi-month lookahead

#### `android/src/static_data.rs`
- **Current Status**: Functional
- **Needed**:
  - **CRITICAL**: Update StaticAddressEntry to use DateTime instead of dag/tid
  - Migrate to DB struct
  - Add data version tracking
  - Implement reload mechanism
  - Error recovery for corrupted data

### 🚧 Medium Priority

#### `android/src/components/file.rs`
- **Current Status**: Works but needs cleanup
- **Needed**:
  - Better error messages
  - Add progress callbacks for large files
  - Implement file validation
  - Add checksum verification

#### `android/src/components/mod.rs`
- **Current Status**: Basic module structure
- **Needed**:
  - Re-export commonly used types
  - Module-level documentation

#### Empty stub files:
- `android/src/components/geo.rs` - Empty
- `android/src/components/notification.rs` - Empty

**Action**: Either implement or remove these stubs

### 🚧 Lower Priority

#### UI Modules
- `android/src/ui/mod.rs`
- `android/src/ui/addresses.rs`
- `android/src/ui/panels.rs`
- `android/src/ui/top_bar.rs`

**Status**: Dioxus UI components - functional but need:
- Accessibility improvements
- Better error display
- Loading states
- Responsive design optimization

## DB Struct Migration Guide

### Overview
The new `DB` struct replaces the `dag` (day) and `tid` (time string) pattern with proper `DateTime<Utc>` timestamps.

### Old Pattern (deprecated)
```rust
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub info: String,
    pub tid: String,      // e.g., "0800-1200"
    pub dag: u8,          // e.g., 17 (17th of month)
}
```

### New Pattern
```rust
pub struct DB {
    pub adress: String,
    pub gata: Option<String>,
    pub gatunummer: Option<String>,
    pub info: Option<String>,
    pub start_time: DateTime<Utc>,  // Full timestamp
    pub end_time: DateTime<Utc>,    // Full timestamp
    pub taxa: Option<String>,
    // ...
}
```

### Migration Steps

1. **Update static_data.rs**:
   ```rust
   // Replace StaticAddressEntry fields
   pub struct StaticAddressEntry {
       // OLD:
       // pub dag: u8,
       // pub tid: String,
       
       // NEW:
       pub start_time: DateTime<Utc>,
       pub end_time: DateTime<Utc>,
       // ...
   }
   ```

2. **Update countdown.rs**:
   ```rust
   // Remove parse_time_interval - now in DB::from_dag_tid
   
   // Update remaining_duration to use DateTime
   pub fn remaining_duration(restriction: &DB) -> Option<Duration> {
       let now = Utc::now();
       restriction.time_until_end(now)
   }
   ```

3. **Update data loading in file.rs**:
   ```rust
   // When reading parquet data, convert to DB:
   let db_entry = DB::from_dag_tid(
       postnummer,
       adress,
       gata,
       gatunummer,
       info,
       dag,      // from parquet
       &tid,     // from parquet
       taxa,
       antal_platser,
       typ_av_parkering,
       year,     // current year
       month,    // current month
   )?;
   ```

4. **Update UI components** to use `start_time` and `end_time` directly

## Testing Strategy

### Unit Tests
- ✅ Core structs (DB)
- ✅ Android bridge (mock)
- ✅ Notifications (mock)
- ✅ Storage (serialization)
- 🚧 Matching
- 🚧 Geo calculations
- 🚧 Countdown with DB

### Integration Tests
- 🚧 End-to-end address lookup
- 🚧 Notification scheduling
- 🚧 Data persistence

### Android Tests
- 🚧 JNI bindings
- 🚧 Permission flows
- 🚧 UI components

## Documentation Status

### ✅ Completed
- android_bridge.rs - Full rustdoc with examples
- notifications.rs - Full rustdoc with examples
- storage.rs - Full rustdoc with examples
- DB struct - Full rustdoc with examples and tests

### 🚧 Needs Documentation
- matching.rs - Add more examples
- geo.rs - Document algorithms and edge cases
- countdown.rs - Update for DB struct
- static_data.rs - Document data format and loading
- UI modules - Add component documentation

## Build Status

### Current Issues
```bash
# Expected warnings after refactoring:
# - Unused imports in modules that reference old structs
# - Dead code warnings for TODO stubs
# - Missing DB field errors in static_data.rs
```

### To Fix
1. Update static_data.rs to use DB struct
2. Update countdown.rs to use DB struct
3. Remove unused dag/tid parsing code
4. Fix import warnings

## Next Steps

### Immediate (This Session)
1. ✅ Add DB struct to core/src/structs.rs
2. ✅ Refactor android_bridge.rs
3. ✅ Refactor notifications.rs
4. ✅ Refactor storage.rs
5. 🔄 Create this documentation

### Next Session
1. Update static_data.rs to use DB struct
2. Update countdown.rs to use DB struct
3. Refactor matching.rs with better error handling
4. Refactor geo.rs with performance improvements
5. Fix all compilation errors
6. Run full test suite

### Future Sessions
1. Implement JNI bindings for android_bridge
2. Implement NotificationManager integration
3. Implement SharedPreferences integration
4. Add integration tests
5. UI polish and accessibility
6. Performance testing with large datasets

## Reference Links

### Android Documentation
- [LocationManager](https://developer.android.com/reference/android/location/LocationManager)
- [Permissions](https://developer.android.com/training/permissions/requesting)
- [Notifications](https://developer.android.com/develop/ui/views/notifications)
- [SharedPreferences](https://developer.android.com/training/data-storage/shared-preferences)

### Rust Resources
- [JNI crate](https://docs.rs/jni/latest/jni/)
- [Chrono](https://docs.rs/chrono/latest/chrono/)
- [Dioxus Mobile](https://dioxuslabs.com/learn/0.5/reference/mobile)

## Notes

### Design Decisions
1. **DateTime over day/time strings**: More type-safe, easier calculations, clearer intent
2. **Result types everywhere**: Explicit error handling, no silent failures
3. **Platform-specific compilation**: Clean separation between Android and mock implementations
4. **TODO comments with references**: Clear implementation guidance with links to documentation
5. **Unit tests for all platforms**: Ensure code works in test environments

### Performance Considerations
- OnceLock for JVM references (thread-safe, lazy initialization)
- Static data caching (load once, use many times)
- Atomic counters for IDs (lock-free)
- Efficient JSON serialization (manual to avoid serde overhead)

### Security Notes
- All user input should be validated before storage
- GPS coordinates should be sanitized
- Notification content should be escaped
- File paths should be validated to prevent directory traversal

---

**Last Updated**: 2026-02-02
**Status**: 🔄 In Progress (4/13 modules fully refactored)
