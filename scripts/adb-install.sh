#!/bin/bash
# Install APK to connected Android device via ADB
# 
# Tries debug APK first, falls back to release APK if debug not found.

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

DEBUG_APK="$REPO_ROOT/target/dx/amp/release/android/app/app/build/outputs/apk/debug/app-debug.apk"
RELEASE_APK="$REPO_ROOT/target/dx/amp/release/android/app/app/build/outputs/apk/release/app-release.apk"

if [ -f "$DEBUG_APK" ]; then
    echo "📦 Installing debug APK..."
    adb install -r "$DEBUG_APK"
elif [ -f "$RELEASE_APK" ]; then
    echo "📦 Installing release APK..."
    adb install -r "$RELEASE_APK"
else
    echo "❌ No APK found at:"
    echo "   $DEBUG_APK"
    echo "   $RELEASE_APK"
    exit 1
fi
