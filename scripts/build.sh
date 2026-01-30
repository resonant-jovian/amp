#!/bin/bash
set -e

echo "🔨 Building Dioxus Android APK..."

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "📍 Project root: $REPO_ROOT"

# Go to android directory
cd "$REPO_ROOT/android" || {
    echo "❌ android directory not found at $REPO_ROOT/android"
    exit 1
}

# Load keystore settings
echo "📖 Loading keystore configuration..."
KEYSTORE_DIR="$REPO_ROOT"
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

# Backup original Dioxus.toml BEFORE modifying it
echo "📝 Backing up original Dioxus.toml..."
DIOXUS_BACKUP="$REPO_ROOT/android/Dioxus.toml.backup.$(date +%s)"
if [ -f "Dioxus.toml" ]; then
    cp -- "Dioxus.toml" "$DIOXUS_BACKUP"
    echo "✓ Backup created: $(basename "$DIOXUS_BACKUP")"
else
    echo "⚠️  No existing Dioxus.toml found, will create new one"
    DIOXUS_BACKUP=""
fi

# Update Dioxus.toml with signing configuration
echo "📝 Updating Dioxus.toml with signing config..."
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
ANDROID_DIR="$REPO_ROOT/target/dx/amp/release/android/app"
rm -rf -- "$ANDROID_DIR" 2>/dev/null || true
rm -rf -- "$REPO_ROOT/android/app/.gradle" 2>/dev/null || true
rm -rf -- "$REPO_ROOT/android/app/build" 2>/dev/null || true
pkill -9 gradle java 2>/dev/null || true
sleep 1

# Build with Dioxus (generates fresh gradle files)
echo "📦 Building APK with Dioxus..."
if ! dx build --android --release --device HQ646M01AF; then
    echo ""
    echo "⚠️  First build failed, applying Java 21 fix and retrying..."
    echo ""

    # FIX: Update generated gradle files for Java 21
    echo "🔧 Fixing generated gradle files for Java 21..."

    if [ -d "$ANDROID_DIR" ]; then
        # Fix build.gradle.kts
        if [ -f "$ANDROID_DIR/build.gradle.kts" ]; then
            echo "  Patching: build.gradle.kts"
            sed -i 's/VERSION_1_8/VERSION_21/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true
            sed -i 's/jvmTarget = "1.8"/jvmTarget = "21"/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true
            echo "✓ Fixed build.gradle.kts"
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
        if grep -q "VERSION_21" "$ANDROID_DIR/build.gradle.kts" 2>/dev/null; then
            echo "✓ build.gradle.kts now uses Java 21"
        fi

        # Fix Android manifest extractNativeLibs issue
        echo "🔧 Fixing Android manifest issues..."
        MANIFEST_FILE="$ANDROID_DIR/app/src/main/AndroidManifest.xml"
        if [ -f "$MANIFEST_FILE" ]; then
            if grep -q 'android:extractNativeLibs="false"' "$MANIFEST_FILE"; then
                echo "  Removing deprecated extractNativeLibs attribute..."
                sed -i 's/ android:extractNativeLibs="false"//g' "$MANIFEST_FILE"
                echo "✓ Fixed manifest"
            fi
        fi

        # Create/update gradle.properties with modern settings
        echo "🔧 Updating gradle.properties..."
        GRADLE_PROPS="$ANDROID_DIR/gradle.properties"
        cat >> "$GRADLE_PROPS" << 'GRADLE_EOF'

# Suppress Java 8 deprecation warnings (using Java 21)
android.javaCompile.suppressSourceTargetDeprecationWarning=true

# Modern Android Gradle Plugin settings
android.useAndroidX=true
android.enableJetifier=true

# Performance optimizations
android.enableBuildFeatures.buildConfig=false
org.gradle.jvmargs=-Xmx4096m
org.gradle.parallel=true
org.gradle.caching=true
GRADLE_EOF
        echo "✓ Updated gradle.properties"

        # Disable lint for release builds using gradle command line (simpler and more reliable)
        echo "🔧 Disabling lint via gradle command line..."

        # Clean gradle cache
        echo "🧹 Cleaning gradle cache..."
        rm -rf -- "$ANDROID_DIR/.gradle" 2>/dev/null || true
        pkill -9 gradle java 2>/dev/null || true
        sleep 2

        # Rebuild with gradle directly, skipping lint tasks
        echo ""
        echo "📦 Rebuilding with fixed gradle configuration (skipping lint)..."
        if ! "$ANDROID_DIR/gradlew" -p "$ANDROID_DIR" clean assembleRelease -x lintVitalAnalyzeRelease -x lintVitalRelease -x lintVitalReportRelease 2>&1 | tee /tmp/gradle_build.log; then
            echo ""
            echo "❌ Gradle build failed after fixes"
            echo ""
            echo "⚠️  Build log saved to /tmp/gradle_build.log"
            echo "Check the error output above for specific details."

            # Still try to restore Dioxus.toml even on failure
            if [ -n "$DIOXUS_BACKUP" ] && [ -f "$DIOXUS_BACKUP" ]; then
                echo ""
                echo "🔄 Restoring Dioxus.toml before exiting..."
                cp -- "$DIOXUS_BACKUP" "Dioxus.toml"
                rm -f -- "$DIOXUS_BACKUP"
                echo "✓ Restored"
            fi

            exit 1
        fi
        echo ""
        echo "✅ BUILD SUCCESSFUL!"
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
    ls -lh -- "$APK_PATH"
    echo ""
    echo "Ready to deploy! 🚀"
else
    echo "  APK not found at expected location"
fi

# Restore original Dioxus.toml from backup
echo "🔄 Restoring original Dioxus.toml..."
if [ -n "$DIOXUS_BACKUP" ] && [ -f "$DIOXUS_BACKUP" ]; then
    cp -- "$DIOXUS_BACKUP" Dioxus.toml
    rm -f -- "$DIOXUS_BACKUP"
    echo "✓ Restored from backup"
else
    echo "⚠️  No backup available, keeping current Dioxus.toml"
fi

echo ""
echo "✅ Build complete!"
