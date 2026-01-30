# Scripts

Utility scripts for building, developing, and deploying the amp Android application.

## Available Scripts

### `build.sh`

Builds a signed release APK for Android.

**Usage:**
```bash
./scripts/build.sh
```

**What it does:**
- Loads keystore configuration from `keystore.properties`
- Backs up and modifies `Dioxus.toml` with signing configuration
- Cleans previous build artifacts
- Builds with `dx build --android --release`
- Applies Java 21 compatibility fixes if needed
- Rebuilds with gradle if initial build fails
- Restores original `Dioxus.toml`
- Outputs signed APK location

**Requirements:**
- Valid `keystore.properties` file in repository root
- Keystore file specified in `keystore.properties`
- `dx` (Dioxus CLI) installed
- Android SDK and Java 21 configured

**Output:**
- Signed release APK in `target/dx/amp/release/android/app/app/build/outputs/apk/release/`

---

### `serve.sh`

Starts development server with hot-reload for Android.

**Usage:**
```bash
./scripts/serve.sh
```

**What it does:**
- Launches `dx serve` with Android hot-reload enabled
- Connects to device `HQ646M01AF`
- Watches for code changes and automatically rebuilds

**Requirements:**
- Android device connected via ADB
- Device ID `HQ646M01AF` (or modify script for your device)
- `dx` (Dioxus CLI) installed

---

### `adb-install.sh`

Installs APK to connected Android device.

**Usage:**
```bash
./scripts/adb-install.sh
```

**What it does:**
- Searches for debug APK first
- Falls back to release APK if debug not found
- Installs via `adb install -r`

**Requirements:**
- Android device connected via ADB
- APK built (via `build.sh` or `dx build`)

---

### `fmt_fix_clippy.sh`

Formats, fixes, and lints all Rust code.

**Usage:**
```bash
./scripts/fmt_fix_clippy.sh
```

**What it does:**
1. Runs `cargo fmt` on all code
2. Runs `cargo clippy --fix` with auto-fixes
3. Runs final `cargo clippy` check treating warnings as errors

**Requirements:**
- Rust toolchain with `rustfmt` and `clippy`

---

## Common Workflows

### First-time Setup
```bash
# 1. Install dependencies (see main README.md)
# 2. Create keystore.properties (see main README.md)
# 3. Build release APK
./scripts/build.sh
```

### Development
```bash
# Start hot-reload development server
./scripts/serve.sh

# In another terminal, make code changes
# Changes will automatically rebuild and deploy
```

### Pre-commit
```bash
# Format and lint code
./scripts/fmt_fix_clippy.sh

# Run tests
cd android && cargo test
```

### Release Build
```bash
# Build signed release APK
./scripts/build.sh

# Install to device
./scripts/adb-install.sh
```

---

## Path Resolution

All scripts use dynamic path resolution to find the repository root:

```bash
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
```

This means:
- Scripts can be run from any directory
- Scripts work regardless of where the repository is cloned
- No hardcoded absolute paths

---

## Troubleshooting

### `build.sh` fails with Java version error

**Problem:** Gradle expects Java 21 but finds different version.

**Solution:** The script applies automatic Java 21 fixes. If it still fails:
1. Verify Java 21 is installed: `java -version`
2. Set `JAVA_HOME` to Java 21 installation
3. Clean and rebuild: `rm -rf target/dx && ./scripts/build.sh`

### `serve.sh` can't find device

**Problem:** Device ID `HQ646M01AF` not found.

**Solution:**
1. Check connected devices: `adb devices`
2. Update device ID in `serve.sh`
3. Or specify at runtime: `dx serve --android --device YOUR_DEVICE_ID`

### `adb-install.sh` says no APK found

**Problem:** APK not built yet.

**Solution:**
1. Build first: `./scripts/build.sh` or `dx build --android`
2. Check output location matches script paths

### `fmt_fix_clippy.sh` reports clippy errors

**Problem:** Code has linting issues.

**Solution:**
1. Review clippy output for specific issues
2. Fix manually or let script apply auto-fixes where possible
3. Some warnings require manual fixes

---

## Adding New Scripts

When adding new scripts:

1. **Use dynamic path resolution:**
   ```bash
   REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
   cd "$REPO_ROOT"
   ```

2. **Make executable:**
   ```bash
   chmod +x scripts/your_script.sh
   ```

3. **Add shebang:**
   ```bash
   #!/bin/bash
   set -e  # Exit on error
   ```

4. **Document in this README:**
   - Add section under "Available Scripts"
   - Explain what it does, usage, and requirements

5. **Follow naming conventions:**
   - Use lowercase with underscores: `my_script.sh`
   - Be descriptive: `build.sh` not `b.sh`
