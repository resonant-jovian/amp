#!/usr/bin/env bash
# Build script for Amp Android app
# Handles custom Kotlin file injection into Dioxus-generated Android project

set -e  # Exit on error

echo "🏗️  Building Amp Android app..."
echo ""

# Step 1: Clean previous build (optional)
echo "📦 Step 1: Cleaning previous build..."
rm -rf gen/android/release 2>/dev/null || true
echo "✅ Clean complete"
echo ""

# Step 2: Run Dioxus build
echo "🦀 Step 2: Building with Dioxus CLI..."
dx build --android --release
echo "✅ Dioxus build complete"
echo ""

# Step 3: Inject custom Kotlin files into generated project
echo "📝 Step 3: Injecting custom Kotlin files..."

# Ensure destination directory exists
mkdir -p gen/android/release/app/src/main/kotlin/se/malmo/skaggbyran/amp

# Copy NotificationPermissionHelper.kt
if [ -f "kotlin/se/malmo/skaggbyran/amp/NotificationPermissionHelper.kt" ]; then
    cp kotlin/se/malmo/skaggbyran/amp/NotificationPermissionHelper.kt \
       gen/android/release/app/src/main/kotlin/se/malmo/skaggbyran/amp/
    echo "  ✓ Copied NotificationPermissionHelper.kt"
else
    echo "  ⚠️  Warning: NotificationPermissionHelper.kt not found, skipping"
fi

# Copy any other custom Kotlin files if they exist
if [ -d "kotlin/se/malmo/skaggbyran/amp" ]; then
    for file in kotlin/se/malmo/skaggbyran/amp/*.kt; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "NotificationPermissionHelper.kt" ]; then
            cp "$file" gen/android/release/app/src/main/kotlin/se/malmo/skaggbyran/amp/
            echo "  ✓ Copied $(basename "$file")"
        fi
    done
fi

echo "✅ Kotlin files injected"
echo ""

# Step 4: Rebuild with Gradle (includes injected Kotlin files)
echo "🔨 Step 4: Running Gradle build with injected files..."
cd gen/android/release
./gradlew assembleRelease
cd ../../..
echo "✅ Gradle build complete"
echo ""

# Step 5: Copy APK to convenient location
echo "📲 Step 5: Preparing APK for installation..."
cp gen/android/release/app/build/outputs/apk/release/app-release.apk ./app-release.apk
echo "✅ APK ready: ./app-release.apk"
echo ""

echo "🎉 Build complete!"
echo ""
echo "To install:"
echo "  adb install -r app-release.apk"
echo ""
echo "To install and run:"
echo "  adb install -r app-release.apk && adb shell am start -n se.malmo.skaggbyran.amp/.MainActivity"
