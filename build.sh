#!/bin/bash
set -e

echo "🔨 Building Dioxus Android APK..."

# Get script directory (project root)
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
echo "📍 Project root: $SCRIPT_DIR"

# Go to android directory
cd "$SCRIPT_DIR/android" || {
    echo "❌ android directory not found at $SCRIPT_DIR/android"
    exit 1
}

# Load keystore settings
echo "📖 Loading keystore configuration..."
KEYSTORE_DIR="$SCRIPT_DIR"
storePassword=$(grep "^storePassword=" "$KEYSTORE_DIR/keystore.properties" | cut -d= -f2 | tr -d ' "')
keyPassword=$(grep "^keyPassword=" "$KEYSTORE_DIR/keystore.properties" | cut -d= -f2 | tr -d ' "')
keyAlias=$(grep "^keyAlias=" "$KEYSTORE_DIR/keystore.properties" | cut -d= -f2 | tr -d ' "')
storeFile=$(grep "^storeFile=" "$KEYSTORE_DIR/keystore.properties" | cut -d= -f2 | tr -d ' "')

echo "✓ Loaded: storeFile=$storeFile, keyAlias=$keyAlias"

# Verify keystore exists
if [ ! -f "$KEYSTORE_DIR/$storeFile" ]; then
    echo "❌ Keystore not found: $KEYSTORE_DIR/$storeFile"
    exit 1
fi
echo "✓ Keystore found"

# Update Dioxus.toml with signing configuration
echo "📝 Updating Dioxus.toml..."
cat > Dioxus.toml << EOF
[application]
name = "amp"
version = "1.0.0"
out_dir = "/home/albin/Documents/"
default_platform = "mobile"
asset_dir = "assets"

[bundle]
publisher = "Skäggbyrån Malmö"
icon = ["assets/icon/icon-512.png"]
resources = ["assets/data/adress_info.parquet"]

[bundle.android]
publisher = "Skäggbyrån Malmö"
icon = ["assets/icon/icon-512.png"]
resources = ["assets/data/adress_info.parquet"]
min_sdk_version = 21
target_sdk_version = 36
orientation = "portrait"
jks_password = "$storePassword"
key_password = "$keyPassword"
key_alias = "$keyAlias"
jks_file = "$storeFile"

[bundle.android.permissions]
android.permission.LOCATION_FINE = true
android.permission.NOTIFICATIONS = true

[profile.android-release]
inherits = "release"
opt-level = 3
strip = false
EOF

echo "✓ Dioxus.toml updated with signing config"

# CRITICAL: Clean previous build to avoid cached gradle files
echo "🧹 Cleaning previous build artifacts..."
ANDROID_DIR="$SCRIPT_DIR/target/dx/amp/release/android"
rm -rf "$ANDROID_DIR" 2>/dev/null || true
rm -rf "$SCRIPT_DIR/android/.gradle" 2>/dev/null || true
rm -rf "$SCRIPT_DIR/android/build" 2>/dev/null || true
pkill -9 gradle java 2>/dev/null || true
sleep 1

# Build with Dioxus (generates fresh gradle files)
echo "📦 Building APK with Dioxus..."
dx build --android --release --device HQ646M01AF

# Check if build succeeded, if not - apply fix and retry
if ! dx build --android --release --device HQ646M01AF; then
    echo ""
    echo "⚠️  First build failed, applying Java 21 fix and retrying..."
    echo ""

    # FIX: Update generated gradle files for Java 21
    echo "🔧 Fixing generated gradle files for Java 21..."

    if [ -d "$ANDROID_DIR" ]; then
        # Fix app/build.gradle.kts
        if [ -f "$ANDROID_DIR/app/build.gradle.kts" ]; then
            echo "  Patching: app/build.gradle.kts"
            sed -i 's/VERSION_1_8/VERSION_21/g' "$ANDROID_DIR/app/build.gradle.kts" 2>/dev/null || true
            sed -i 's/jvmTarget = "1.8"/jvmTarget = "21"/g' "$ANDROID_DIR/app/build.gradle.kts" 2>/dev/null || true
            echo "✓ Fixed app/build.gradle.kts"
        fi

        # Fix root build.gradle.kts
        if [ -f "$ANDROID_DIR/build.gradle.kts" ]; then
            echo "  Patching: build.gradle.kts"
            sed -i 's/VERSION_1_8/VERSION_21/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true
            echo "✓ Fixed root build.gradle.kts"
        fi

        # Verify the fixes worked
        echo ""
        echo "📋 Verifying fixes:"
        if grep -q "VERSION_21" "$ANDROID_DIR/app/build.gradle.kts" 2>/dev/null; then
            echo "✓ app/build.gradle.kts now uses Java 21"
        fi

        # Clean gradle cache
        echo "🧹 Cleaning gradle cache..."
        rm -rf "$ANDROID_DIR/.gradle" 2>/dev/null || true
        pkill -9 gradle java 2>/dev/null || true
        sleep 2

        # Rebuild with gradle directly
        echo ""
        echo "📦 Rebuilding with fixed gradle configuration..."
        cd "$ANDROID_DIR"
        ./gradlew clean assembleRelease

        if ! dx build --android --release --device HQ646M01AF; then
            echo ""
            echo "✅ BUILD SUCCESSFUL!"
        else
            echo ""
            echo "❌ Gradle build failed even after Java 21 fix"
            echo "This might be a different issue. Check the error above."
            exit 1
        fi
    else
        echo "❌ Android directory not created: $ANDROID_DIR"
        echo "This means dx build failed before generating gradle files."
        exit 1
    fi
else
    echo ""
    echo "✓ Dioxus build completed successfully on first try!"
fi

# Show APK location
echo ""
echo "📍 APK location:"
APK_DIR="$ANDROID_DIR/app/build/outputs/apk/release"

APK_PATH="$(
  find "$APK_DIR" -maxdepth 1 -type f -name '*.apk' -printf '%T@ %p\n' 2>/dev/null \
  | sort -nr \
  | head -n 1 \
  | cut -d' ' -f2-
)"

if [ -n "$APK_PATH" ]; then
    ls -lh "$APK_PATH"
    echo ""
    echo "Ready to deploy! 🚀"
else
    echo "  APK not found at expected location"
fi

# Restore original Dioxus.toml
echo "🔄 Restoring Dioxus.toml..."
cd "$SCRIPT_DIR/android"
git checkout Dioxus.toml 2>/dev/null || echo "  (Dioxus.toml not under git control)"

echo ""
echo "✅ Build complete!"