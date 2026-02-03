# Debug Mode

The Android app includes a debug mode feature that loads read-only example addresses from an embedded `debug.parquet` file.

## Overview

**Purpose**: Testing and development without affecting user data

**Key Characteristics**:
- Read-only: Never writes to storage
- Embedded: Addresses bundled in APK as `debug.parquet`
- Toggle-able: Can be enabled/disabled via settings
- Same UI: Uses identical active/inactive switching as regular addresses

## Usage

### Enable Debug Mode

1. Open app settings (gear icon in top bar)
2. Toggle "Debugläge (Exempeladresser)" switch to ON
3. App immediately loads debug addresses from embedded file

### Disable Debug Mode

1. Open app settings
2. Toggle "Debugläge" switch to OFF
3. App immediately loads user addresses from storage

### Behavior in Debug Mode

**Allowed**:
- Toggle active/inactive state on addresses
- View addresses in panels
- Test notification scheduling logic
- Test validity checking

**Not Allowed**:
- Add new addresses (button disabled)
- Remove addresses (delete button disabled)
- Persist changes to storage

**Active state changes**: Toggling active/inactive in debug mode modifies addresses in-memory only. Changes are lost when:
- Debug mode is toggled off
- App is restarted
- Debug mode is toggled on again (reloads from file)

## Implementation

### File Structure

```
android/
├── assets/
│   ├── db.parquet              # Parking restrictions database
│   └── debug.parquet           # Debug addresses (LocalData schema)
├── src/
│   ├── components/
│   │   └── debug.rs            # Load debug addresses
│   └── ui/
│       ├── mod.rs              # Debug mode state management
│       ├── settings_dropdown.rs # Debug toggle UI
│       └── top_bar.rs          # Pass debug state
└── ...
```

### Debug Addresses File

**Location**: `android/assets/debug.parquet`

**Schema**: Uses `LocalData` schema from `core/src/structs.rs`

**Generation**: Run the generation script:
```bash
cargo run --bin generate_debug_parquet
```

This creates `android/assets/debug.parquet` with example addresses.

### Code Components

#### `components/debug.rs`

```rust
use amp_android::components::debug;

// Load debug addresses from embedded file
let debug_addresses = debug::load_debug_addresses();
```

**Features**:
- Reads from embedded `debug.parquet` using `include_bytes!()`
- Converts `LocalData` to `StoredAddress`
- Returns empty vec if file missing or corrupt
- Thread-safe, can be called multiple times

#### `ui/mod.rs` - Debug State

```rust
let mut debug_mode = use_signal(|| false);

let handle_toggle_debug = move |_| {
    let new_debug_mode = !debug_mode();
    debug_mode.set(new_debug_mode);
    
    if new_debug_mode {
        // Load debug addresses
        let debug_addrs = load_debug_addresses();
        stored_addresses.set(debug_addrs);
    } else {
        // Load user addresses
        let loaded = read_addresses_from_device();
        stored_addresses.set(loaded);
    }
};
```

**State Management**:
- `debug_mode` signal tracks current state
- Toggling loads appropriate address set
- Prevents writes to storage when enabled

#### `ui/settings_dropdown.rs` - Toggle UI

```rust
SettingsDropdown {
    is_open: show_settings(),
    on_close: handle_close_settings,
    debug_mode: debug_mode(),
    on_toggle_debug: handle_toggle_debug,
}
```

**UI Element**: Standard toggle switch with label and hint

### Storage Behavior

| Operation | Normal Mode | Debug Mode |
|-----------|-------------|------------|
| **Add address** | ✅ Writes to `local.parquet` | ❌ Blocked |
| **Remove address** | ✅ Writes to `local.parquet` | ❌ Blocked |
| **Toggle active** | ✅ Writes to `local.parquet` | ⚠️ In-memory only |
| **App start** | ✅ Reads from `local.parquet` | ✅ Reads from `debug.parquet` |
| **Daily task** | ✅ Reads/writes `local.parquet` | ❌ No storage operations |

### Debug Addresses Content

The `debug.parquet` file contains 10-11 example addresses covering:

**Valid addresses** (9-10 addresses):
- Various postal codes in Malmö
- Different street types (gatan, vägen, plan)
- Mix of simple and complex street numbers ("10", "18C", "50 U1")
- Realistic Swedish address formats

**Invalid address** (1 address):
- Postal code: "11111" (non-existent)
- Street: "Låssasgatan 11A"
- Tests invalid address handling

## Testing

### Manual Testing Checklist

**Basic Functionality**:
- [ ] Enable debug mode → see debug addresses loaded
- [ ] Disable debug mode → see user addresses loaded
- [ ] Toggle multiple times → addresses swap correctly

**Read-Only Enforcement**:
- [ ] Try adding address in debug mode → blocked
- [ ] Try removing address in debug mode → blocked
- [ ] Toggle active state → works, changes lost on disable

**Persistence**:
- [ ] Enable debug mode → restart app → still using user addresses (debug mode doesn't persist)
- [ ] Toggle active in debug mode → disable debug → user addresses unchanged

**Edge Cases**:
- [ ] Empty user storage + enable debug → debug addresses load
- [ ] Corrupted `debug.parquet` → empty list, no crash
- [ ] Missing `debug.parquet` → empty list, no crash

### Unit Tests

```bash
# Test debug address loading
cargo test --package amp-android debug
```

Tests in `android/src/components/debug.rs`:
- `test_load_debug_addresses` - Verifies loading succeeds
- `test_debug_addresses_have_postal_codes` - Validates address data

## Regenerating Debug Addresses

### When to Regenerate

- Adding/removing debug addresses
- Changing address data
- Updating LocalData schema

### Steps

1. **Edit the script**: Modify `scripts/generate_debug_parquet.rs`
   - Add/remove addresses in the `debug_addresses` vec
   - Update address fields as needed

2. **Regenerate file**:
   ```bash
   cargo run --bin generate_debug_parquet
   ```

3. **Verify file created**:
   ```bash
   ls -lh android/assets/debug.parquet
   ```

4. **Rebuild app**:
   ```bash
   cd android
   dx build --platform android --release
   ```

5. **Test**:
   - Install APK
   - Enable debug mode
   - Verify new addresses appear

## Troubleshooting

### Issue: Debug addresses not loading

**Check**:
1. File exists: `ls android/assets/debug.parquet`
2. File embedded in APK: `unzip -l android/dist/amp-android.apk | grep debug.parquet`
3. Console logs: `adb logcat | grep Debug`

**Solution**: Regenerate and rebuild

### Issue: Wrong addresses in debug mode

**Check**:
1. File is current: `ls -l android/assets/debug.parquet`
2. APK is current: `ls -l android/dist/amp-android.apk`

**Solution**: Clean build
```bash
cd android
rm -rf dist/
dx build --platform android --release
```

### Issue: Can still modify addresses in debug mode

**Check**: Debug mode toggle state in UI

**Debug**:
```rust
// Add logging in handle_add_address
if debug_mode() {
    warn!("Cannot add addresses in debug mode (read-only)");
    return;
}
```

## Production Considerations

### Security

- Debug addresses are visible to users
- Don't include sensitive data in debug.parquet
- Use realistic but fictitious addresses

### Performance

- `debug.parquet` embedded in APK (~5KB)
- Loaded on demand (not at app start)
- Minimal impact on APK size and runtime

### User Experience

- Debug mode doesn't persist across restarts (intentional)
- Users must re-enable after each app launch
- Prevents accidental permanent switch to debug mode

## Related Documentation

- [Persistent State](android-persistent-state.md) - Storage system
- [Architecture](architecture.md) - Overall system design
- `core/src/structs.rs` - LocalData schema
- `core/src/parquet.rs` - Parquet read/write functions

## Future Enhancements

### Potential Improvements

1. **Persist debug mode setting** - Remember across restarts
2. **Multiple debug sets** - Different scenarios (rush hour, weekend, etc.)
3. **Custom debug addresses** - User-editable debug set
4. **Debug mode indicators** - Visual banner when active
5. **Debug mode analytics** - Track usage for debugging
