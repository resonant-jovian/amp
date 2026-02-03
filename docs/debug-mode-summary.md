# Debug Mode - Quick Reference

## What Was Implemented

Debug mode system for Android app that loads read-only example addresses from an embedded `debug.parquet` file.

## Files Created

| File | Purpose |
|------|----------|
| `scripts/generate_debug_parquet.rs` | Script to generate debug.parquet with example addresses |
| `android/src/components/debug.rs` | Component to load debug addresses from embedded file |
| `docs/debug-mode.md` | Complete documentation |
| `docs/debug-mode-summary.md` | This quick reference |

## Files Modified

| File | Changes |
|------|----------|
| `android/src/components/mod.rs` | Added `pub mod debug;` |
| `android/src/ui/settings_dropdown.rs` | Added debug mode toggle switch |
| `android/src/ui/top_bar.rs` | Added debug_mode and on_toggle_debug props |
| `android/src/ui/mod.rs` | Implemented debug mode state management |

## How to Use

### 1. Generate debug.parquet (First Time Only)

```bash
# From project root
cargo run --bin generate_debug_parquet
```

This creates `android/assets/debug.parquet` with 10-11 example addresses.

### 2. Build App

```bash
cd android
dx build --platform android --release
```

### 3. Enable in App

1. Open app
2. Tap settings icon (gear) in top bar
3. Toggle "Debugläge (Exempeladresser)" to ON
4. Debug addresses load immediately

## Key Features

✅ **Read-only**: Never writes to storage  
✅ **Embedded**: No external dependencies  
✅ **Toggle-able**: Enable/disable via settings  
✅ **Safe**: Cannot add/remove addresses in debug mode  
⚠️ **Active state**: Can toggle active/inactive (in-memory only, not persisted)  

## Debug Addresses Content

Default `debug.parquet` contains:
- 10 valid Malmö addresses (various postal codes)
- 1 invalid address (postal code 11111)
- Mix of street types and number formats
- Realistic Swedish address data

## Storage Behavior

| Action | Normal Mode | Debug Mode |
|--------|-------------|------------|
| Add address | ✅ Writes to storage | ❌ Blocked |
| Remove address | ✅ Writes to storage | ❌ Blocked |
| Toggle active | ✅ Persisted | ⚠️ In-memory only |
| App start | Loads from storage | Loads from debug.parquet |

## Testing Checklist

- [ ] Generate debug.parquet
- [ ] Rebuild app
- [ ] Enable debug mode → see 11 addresses
- [ ] Try adding address → blocked
- [ ] Try removing address → blocked  
- [ ] Toggle active state → works
- [ ] Disable debug mode → see user addresses
- [ ] Re-enable debug mode → active states reset

## Regenerating Debug Addresses

```bash
# 1. Edit scripts/generate_debug_parquet.rs
# 2. Run generator
cargo run --bin generate_debug_parquet

# 3. Rebuild app
cd android
dx build --platform android --release
```

## Troubleshooting

**Debug addresses not loading?**
```bash
# Check file exists
ls -lh android/assets/debug.parquet

# Rebuild
cd android
rm -rf dist/
dx build --platform android --release
```

**Wrong addresses showing?**
- Regenerate debug.parquet
- Clean build (remove dist/ folder)
- Reinstall APK

## Architecture

```
User toggles debug mode in settings
    ↓
handle_toggle_debug() in ui/mod.rs
    ↓
If enabled:
    load_debug_addresses() from components/debug.rs
        ↓
    Read embedded debug.parquet
        ↓
    Convert LocalData → StoredAddress
        ↓
    Set stored_addresses signal
If disabled:
    read_addresses_from_device() from components/storage.rs
        ↓
    Read local.parquet from storage
        ↓
    Set stored_addresses signal
```

## Related Documentation

- [`docs/debug-mode.md`](debug-mode.md) - Complete documentation
- [`docs/android-persistent-state.md`](android-persistent-state.md) - Storage system
- [`android/src/components/debug.rs`](../android/src/components/debug.rs) - Implementation
- [`scripts/generate_debug_parquet.rs`](../scripts/src/generate_debug_parquet.rs) - Generator script

## Next Steps

1. Generate debug.parquet: `cargo run --bin generate_debug_parquet`
2. Build app: `cd android && dx build --platform android --release`
3. Test on device
4. (Optional) Customize debug addresses in generator script
