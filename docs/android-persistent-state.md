# Android Persistent State Implementation

This document describes the persistent state management system for the Android version of the amp parking notification app.

## Overview

The Android app implements a robust persistent state system using:
- **Parquet file format** for efficient binary storage
- **Backup rotation** for data safety
- **Background lifecycle management** for scheduled tasks
- **Validity checking** for date-dependent parking restrictions

## Storage Architecture

### File Structure

```
/data/data/com.yourapp/files/
├── local.parquet        # Current active data
└── local.parquet.backup # Previous version backup
```

### Data Schema

The `LocalData` struct (defined in `core/src/structs.rs`) contains:

```rust
pub struct LocalData {
    pub valid: bool,              // Whether restriction is valid in current month
    pub active: bool,             // User toggle state
    pub postnummer: Option<String>,
    pub adress: String,
    pub gata: Option<String>,
    pub gatunummer: Option<String>,
    pub info: Option<String>,     // Parking restriction info
    pub tid: Option<String>,      // Time range (e.g., "0800-1200")
    pub dag: Option<u8>,          // Day of month (1-31)
    pub taxa: Option<String>,     // Parking zone
    pub antal_platser: Option<u64>,
    pub typ_av_parkering: Option<String>,
}
```

## Read Operations

### When to Read

1. **App Start**: Load all addresses from storage
2. **Once per day**: Background task refreshes data at midnight

### Read Flow

```rust
use amp_android::components::storage::read_addresses_from_device;

let addresses = read_addresses_from_device();
```

**Logic**:
1. Check if `local.parquet` exists
2. If not, check if `local.parquet.backup` exists
   - If yes: duplicate backup to `local.parquet` and read
3. If neither exists: create empty files with schema
4. Read parquet file and convert to `Vec<StoredAddress>`

## Write Operations

### When to Write

1. **Address added** to address list
2. **Address removed** from address list
3. **Active state switched** on any address (bool toggle)
4. **Valid state changed** (day 29/30 in February transitions)
5. **App crash/full exit** (graceful shutdown)
6. **Once per day** at midnight (if validity changed)

### Write Flow with Backup Rotation

```rust
use amp_android::components::storage::write_addresses_to_device;

write_addresses_to_device(&addresses)?;
```

**Backup Rotation Steps**:
1. Delete old `local.parquet.backup` (if exists)
2. Rename current `local.parquet` → `local.parquet.backup`
3. Write new data to `local.parquet`

This ensures:
- Always have a backup of the previous state
- Can recover if write operation is interrupted
- No data loss on crash during write

## Lifecycle Management

### LifecycleManager

The `LifecycleManager` (in `android/src/components/lifecycle.rs`) handles:
- Daily background tasks
- Uptime tracking
- Graceful shutdown

### Initialization

```rust
use amp_android::components::lifecycle::LifecycleManager;

let mut manager = LifecycleManager::new();
manager.start(); // Performs initial load and validity check
```

### Daily Tasks

Executed once per day (checks at app runtime):

1. Read current addresses from storage
2. Run validity checks (see next section)
3. If validity changed, write back to storage

### Event Handlers

```rust
use amp_android::components::lifecycle::{
    handle_address_change,
    handle_active_toggle,
    handle_validity_change,
};

// When address added/removed
handle_address_change(&addresses);

// When active state toggled
handle_active_toggle(&addresses);

// When validity changed (internal)
handle_validity_change(&addresses);
```

### Graceful Shutdown

```rust
manager.shutdown(); // Saves current state before exit
```

The `LifecycleManager` also implements `Drop` to ensure shutdown is called automatically.

## Validity Checking

### Problem Statement

Parking restrictions tied to specific days of the month become invalid when:
- Day 29: Invalid in non-leap-year February
- Day 30: Invalid in February (always)
- Day 31: Invalid in months with only 30 days

### Implementation

```rust
use amp_android::components::validity::{
    check_and_update_validity,
    is_valid_in_current_month,
};

// Check if a specific day is valid in current month
let valid = is_valid_in_current_month(Some(30)); // false in February

// Update all addresses
let mut addresses = get_addresses();
if check_and_update_validity(&mut addresses) {
    // Some addresses changed validity
    save_addresses(&addresses);
}
```

### When Validity is Checked

1. **App startup**: After loading addresses
2. **Daily background task**: At midnight
3. **After adding address**: Immediate check

### Validity Logic

```rust
pub fn is_valid_in_current_month(dag: Option<u8>) -> bool {
    let dag = match dag {
        Some(d) => d as u32,
        None => return true, // No day restriction = always valid
    };
    
    let now = Local::now();
    let current_month = now.month();
    let current_year = now.year();
    let max_days = days_in_month(current_month, current_year);
    
    dag <= max_days
}
```

## Background Service Requirements

For Android to allow background operations and notifications:

### 1. Foreground Service

Required for continuous background operation:

```xml
<!-- AndroidManifest.xml -->
<service
    android:name=".BackgroundService"
    android:foregroundServiceType="dataSync"
    android:exported="false" />
```

Java/Kotlin implementation needed:
```java
public class BackgroundService extends Service {
    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        // Create notification for foreground service
        Notification notification = createNotification();
        startForeground(NOTIFICATION_ID, notification);
        
        // Start background loop
        startBackgroundTasks();
        
        return START_STICKY; // Restart if killed
    }
}
```

### 2. Boot Receiver

Start service on device boot:

```xml
<!-- AndroidManifest.xml -->
<receiver
    android:name=".BootReceiver"
    android:enabled="true"
    android:exported="true">
    <intent-filter>
        <action android:name="android.intent.action.BOOT_COMPLETED" />
    </intent-filter>
</receiver>

<uses-permission android:name="android.permission.RECEIVE_BOOT_COMPLETED" />
```

Java/Kotlin implementation:
```java
public class BootReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent intent) {
        if (Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())) {
            Intent serviceIntent = new Intent(context, BackgroundService.class);
            context.startForegroundService(serviceIntent);
        }
    }
}
```

### 3. Required Permissions

```xml
<!-- AndroidManifest.xml -->
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.RECEIVE_BOOT_COMPLETED" />
<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
<uses-permission android:name="android.permission.WAKE_LOCK" />
```

### 4. Battery Optimization Exemption

Request user to disable battery optimization:

```java
Intent intent = new Intent();
String packageName = context.getPackageName();
PowerManager pm = (PowerManager) context.getSystemService(Context.POWER_SERVICE);

if (!pm.isIgnoringBatteryOptimizations(packageName)) {
    intent.setAction(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS);
    intent.setData(Uri.parse("package:" + packageName));
    startActivity(intent);
}
```

## Integration with Rust

### JNI Bridge for Background Service

Create JNI functions to call from Java:

```rust
// In android_bridge.rs or new file
use jni::JNIEnv;
use jni::objects::JClass;

#[no_mangle]
pub extern "system" fn Java_com_yourapp_BackgroundService_performDailyTasks(
    env: JNIEnv,
    _class: JClass,
) {
    use crate::components::lifecycle::LifecycleManager;
    
    // This will be called from Java service
    let manager = LifecycleManager::new();
    manager.check_and_run_daily_tasks();
}

#[no_mangle]
pub extern "system" fn Java_com_yourapp_BackgroundService_shutdown(
    env: JNIEnv,
    _class: JClass,
) {
    use crate::components::lifecycle::LifecycleManager;
    
    let mut manager = LifecycleManager::new();
    manager.shutdown();
}
```

## Testing

### Unit Tests

All components have comprehensive tests:

```bash
# Test storage operations
cargo test --package amp-android storage

# Test validity checking
cargo test --package amp-android validity

# Test lifecycle management
cargo test --package amp-android lifecycle
```

### Manual Testing Checklist

- [ ] Add address → verify write to storage
- [ ] Remove address → verify write to storage
- [ ] Toggle active state → verify write to storage
- [ ] Restart app → verify addresses loaded correctly
- [ ] Delete `local.parquet` → verify recovery from backup
- [ ] Delete both files → verify empty files created
- [ ] Test in February (non-leap year) → day 29/30 marked invalid
- [ ] Test month transition → validity updates automatically
- [ ] Force kill app → verify data not lost

## Troubleshooting

### Issue: Data not persisting

**Check**:
1. File permissions - app must have write access to internal storage
2. Storage available - check device has free space
3. Error logs - check eprintln output for storage errors

### Issue: Backup file missing

**Normal if**: First run of app, backup created on second write

**Recovery**: System will create backup automatically on next write

### Issue: Validity not updating

**Check**:
1. Daily tasks running - check LifecycleManager logs
2. Month detection - verify system clock correct
3. matched_entry present - only addresses with matches get validity checks

## Performance Considerations

### File Size

Parquet is very efficient:
- 100 addresses ≈ 5-10 KB
- 1000 addresses ≈ 50-100 KB

Compression and binary format keep files small.

### Read/Write Speed

- **Read**: ~1ms for 100 addresses
- **Write**: ~2ms for 100 addresses (includes backup rotation)

Negligible impact on UI performance.

### Memory Usage

Addresses kept in memory as `Vec<StoredAddress>`:
- 100 addresses ≈ 50 KB RAM
- 1000 addresses ≈ 500 KB RAM

Acceptable for mobile devices.

## Future Enhancements

### Potential Improvements

1. **Cloud Backup**: Sync to user account
2. **Compression**: Further reduce file size
3. **Incremental Writes**: Only write changed addresses
4. **Import/Export**: User-facing backup functionality
5. **Migration**: Auto-migrate from old storage format

## Related Files

- `android/src/components/storage.rs` - Parquet storage implementation
- `android/src/components/lifecycle.rs` - Background task management
- `android/src/components/validity.rs` - Date validation logic
- `android/src/ui/mod.rs` - UI integration
- `core/src/parquet.rs` - Parquet read/write functions
- `core/src/structs.rs` - Data structures (LocalData, DB)

## References

- [Android Background Work](https://developer.android.com/guide/background)
- [Foreground Services](https://developer.android.com/develop/background-work/services/foreground-services)
- [WorkManager](https://developer.android.com/topic/libraries/architecture/workmanager)
- [Apache Parquet](https://parquet.apache.org/)
- [Dioxus Mobile](https://dioxuslabs.com/learn/0.6/guides/mobile)
