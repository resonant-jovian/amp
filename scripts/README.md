# Build Scripts

Automation scripts for building and configuring the Amp Android app.

## build.sh - Android Notification Setup

### Purpose

Automatically configures the Dioxus Android project for notification support by:
- Copying `NotificationHelper.kt` to the Android project
- Adding required permissions to `AndroidManifest.xml`
- Creating necessary package directories
- Verifying the setup

### Usage

#### Basic Usage (Default Path)

```bash
cd /path/to/amp
chmod +x scripts/build.sh
./scripts/build.sh
```

This assumes your Android project is at the default Dioxus path:
```
target/dx/amp/release/android/app/app/
```

#### Custom Android Path

If your Android project is in a different location:

```bash
# Method 1: Command-line argument
./scripts/build.sh --android-path /home/albin/Documents/amp/target/dx/amp/release/android/app/app

# Method 2: Environment variable
ANDROID_PROJECT_PATH="/custom/path/to/android" ./scripts/build.sh
```

#### Get Help

```bash
./scripts/build.sh --help
```

### What It Does

1. **Verifies Source Files**
   - Checks that `android/kotlin/NotificationHelper.kt` exists

2. **Checks Android Project**
   - Verifies the Android project directory exists
   - If not found, provides instructions to build it

3. **Sets Up Directories**
   - Creates `src/main/java/com/amp/` if needed

4. **Copies Kotlin Files**
   - Copies `NotificationHelper.kt` to the Android project
   - Destination: `app/src/main/java/com/amp/NotificationHelper.kt`

5. **Updates AndroidManifest.xml**
   - Adds notification permissions (Android 13+)
   - Adds foreground service permissions
   - Creates backup before modifying
   - Skips if permissions already present

### Permissions Added

The script adds these permissions to your `AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />
```

### Example Output

```
╔════════════════════════════════════════╗
║  Amp Android Notification Setup       ║
╚════════════════════════════════════════╝

Configuration:
  Project root: /home/albin/Documents/amp
  Android path: /home/albin/Documents/amp/target/dx/amp/release/android/app/app
  Manifest: /home/albin/Documents/amp/target/dx/amp/release/android/app/app/src/main/AndroidManifest.xml

[1/5] Verifying source files...
✓ Found: /home/albin/Documents/amp/android/kotlin/NotificationHelper.kt

[2/5] Checking Android project...
✓ Android project found

[3/5] Setting up directories...
✓ Directory exists: /home/albin/Documents/amp/target/dx/amp/release/android/app/app/src/main/java/com/amp

[4/5] Copying Kotlin files...
✓ NotificationHelper.kt copied successfully
  Destination: /home/albin/Documents/amp/target/dx/amp/release/android/app/app/src/main/java/com/amp/NotificationHelper.kt

[5/5] Updating AndroidManifest.xml...
Adding POST_NOTIFICATIONS permission...
  Backup created: AndroidManifest.xml.backup
✓ POST_NOTIFICATIONS added
Adding FOREGROUND_SERVICE permissions...
✓ FOREGROUND_SERVICE permissions added

Verifying manifest...
✓ All permissions present in manifest

╔════════════════════════════════════════╗
║  Setup Complete!                       ║
╚════════════════════════════════════════╝

Files modified:
  ✓ NotificationHelper.kt
  ✓ AndroidManifest.xml

Permissions added:
  ✓ POST_NOTIFICATIONS (Android 13+)
  ✓ FOREGROUND_SERVICE
  ✓ FOREGROUND_SERVICE_DATA_SYNC

Next steps:
  1. Build APK: dx build --platform android --release
  2. Run on device: dx serve --platform android
  3. Monitor: adb logcat | grep -E '(Notifications|AmpNotifications)'

Setup successful! 🎉
```

### Workflow

Typical development workflow:

```bash
# 1. Build the Android project initially
dx build --platform android --release

# 2. Run the setup script
./scripts/build.sh

# 3. Rebuild with updated files
dx build --platform android --release

# 4. Deploy to device
dx serve --platform android

# 5. Monitor notifications
adb logcat | grep -E '(Notifications|amp_)'
```

### Troubleshooting

#### Android Project Not Found

**Error**: `Android project not found at: target/dx/amp/release/android/app/app`

**Solution**: Build the Android project first:
```bash
dx build --platform android --release
```

#### Kotlin Source Not Found

**Error**: `File not found: android/kotlin/NotificationHelper.kt`

**Solution**: Run from project root or ensure you're on the `feature/android` branch:
```bash
cd /path/to/amp
git checkout feature/android
```

#### Permission Denied

**Error**: `Permission denied: ./scripts/build.sh`

**Solution**: Make script executable:
```bash
chmod +x scripts/build.sh
```

#### Manifest Not Updated

**Issue**: Permissions not appearing after script runs

**Solution**: 
1. Check if backup was created: `*.backup` files
2. Manually verify permissions in AndroidManifest.xml
3. Check script output for errors
4. Try with sudo if permission issues

### Re-running the Script

The script is **idempotent** - safe to run multiple times:
- Skips adding permissions if already present
- Overwrites `NotificationHelper.kt` (always uses latest version)
- Creates backup only on first modification

### Manual Setup (Alternative)

If you prefer to set up manually:

1. **Copy Kotlin file**:
   ```bash
   cp android/kotlin/NotificationHelper.kt \
      target/dx/amp/release/android/app/app/src/main/java/com/amp/
   ```

2. **Edit AndroidManifest.xml**:
   Add permissions after `<manifest>` tag:
   ```xml
   <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
   <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
   <uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />
   ```

### Files Modified

- **Added**: `app/src/main/java/com/amp/NotificationHelper.kt`
- **Modified**: `app/src/main/AndroidManifest.xml`
- **Backup**: `app/src/main/AndroidManifest.xml.backup` (on first run)

### Related Documentation

- [Android Notification Guide](../docs/android-notifications.md)
- [JNI Integration Guide](../android/kotlin/README.md)
- [Implementation Summary](../android/NOTIFICATIONS_IMPLEMENTATION.md)

### Testing Setup

After running the script, verify setup:

```bash
# 1. Check Kotlin file was copied
ls -la target/dx/amp/release/android/app/app/src/main/java/com/amp/NotificationHelper.kt

# 2. Verify permissions in manifest
grep "POST_NOTIFICATIONS" target/dx/amp/release/android/app/app/src/main/AndroidManifest.xml

# 3. Build and test
dx serve --platform android

# 4. On device, test notification channels
adb shell dumpsys notification | grep amp_
```

### Advanced Usage

#### Continuous Integration

Add to CI pipeline:

```yaml
# .github/workflows/android.yml
- name: Setup Android notifications
  run: |
    chmod +x scripts/build.sh
    ./scripts/build.sh
    
- name: Build APK
  run: dx build --platform android --release
```

#### Custom Package Name

If using a different package name (not `com.amp`):

1. Update `NotificationHelper.kt` package declaration
2. Modify `JAVA_DIR` in script to match your package
3. Update JNI calls in `android_bridge.rs` to use new package path

### Support

For issues or questions:
- Check documentation in `docs/` and `android/kotlin/`
- Review commit history on `feature/android` branch
- See `android/NOTIFICATIONS_IMPLEMENTATION.md` for implementation details

---

## storage-obs.sh - APK Storage Structure Observer

Investigates internal storage structure of packaged APK and deployed app.

### Purpose

Debug asset loading issues, verify build outputs, and understand the app's storage footprint for Android platform development. This script inspects the APK file structure, examines deployed app storage on a connected device, and verifies that resources are correctly packaged.

### Usage

```bash
# Analyze latest built APK
./scripts/storage-obs.sh

# Analyze specific APK
./scripts/storage-obs.sh --apk-path /path/to/amp.apk

# Save report to file
./scripts/storage-obs.sh --output storage-report.txt

# Device inspection only (requires connected device)
./scripts/storage-obs.sh --device-only

# Show help
./scripts/storage-obs.sh --help
```

### What It Inspects

1. **APK Internal Structure**
   - DEX files (classes.dex) with sizes and count
   - Native libraries per architecture (lib/ directory)
   - Assets directory with color-coded file types
   - Resources (res/) directory summary
   - Full file listing with compressed/uncompressed sizes

2. **APK Metadata (via aapt)**
   - Android permissions (color-coded by category)
   - Package name, version, SDK targets
   - App label and other manifest metadata

3. **Asset Packaging Verification**
   - Confirms `data/adress_info.parquet` is packaged
   - Verifies `style.css` inclusion
   - Lists all font files (TTF/OTF/WOFF)
   - Compares APK file sizes against source files

4. **Device Storage Paths**
   - Internal app storage (`/data/data/se.malmo.skaggbyran.amp/`)
   - External storage (`/storage/emulated/0/Android/data/...`)
   - Standard directories: cache, databases, shared_prefs, files
   - Package info from device dumpsys

### Options

| Flag | Description |
|---|---|
| `--apk-path <path>` | Analyze a specific APK file instead of auto-detecting |
| `--output <file>` | Save report to a file (also prints to stdout) |
| `--device-only` | Skip APK analysis, only inspect connected device |
| `--help` | Show usage information |

### Prerequisites

- `unzip` - Required for APK listing (usually pre-installed)
- `aapt` - Optional, from Android SDK build-tools (enables detailed metadata)
- `adb` - Required for device inspection (from Android SDK platform-tools)

### Example Output

```
╔════════════════════════════════════════════════════════════════╗
║       Amp APK Storage Structure Observer                      ║
╚════════════════════════════════════════════════════════════════╝

Project root:   /home/albin/Documents/amp
Package:        se.malmo.skaggbyran.amp
APK:            .../build/outputs/apk/release/app-release.apk

╔════════════════════════════════════════════════════════════╗
║ ════════════ APK Structure Analysis ════════════          ║
╚════════════════════════════════════════════════════════════╝

APK: .../app-release.apk
Size: 24M

── DEX Files (Compiled Code) ──

  📦 classes.dex  4.21 MB
  📦 classes2.dex  1.83 MB

  DEX count: 2
  DEX total: 6.04 MB

── Assets Directory ──

  📊 assets/data/adress_info.parquet  8.52 MB
  🎨 assets/style.css  12.3 KB
  🔤 assets/fonts/Inter-Regular.ttf  312.0 KB

── Asset Verification ──

  ✅ assets/data/adress_info.parquet  (8.52 MB)
  ✅ assets/style.css  (12.3 KB)
  ✅ All asset verifications passed ✅
```

### Testing

After creation, verify the script works:

```bash
# 1. Build APK
./scripts/build.sh

# 2. Run APK analysis
./scripts/storage-obs.sh

# 3. Deploy to device and inspect storage
adb install <path-to-apk>
./scripts/storage-obs.sh --device-only

# 4. Save full report
./scripts/storage-obs.sh --output scripts/storage-report.txt
```

### Related

- [Issue #33 - Android Platform Development](https://github.com/resonant-jovian/amp/issues/33)
- [Feature branch](https://github.com/resonant-jovian/amp/tree/feature/android)
- [build.sh documentation](#buildsh---android-notification-setup)
