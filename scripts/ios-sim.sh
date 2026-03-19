#!/bin/bash
# Build and deploy AMP to iOS Simulator
# Usage: ./scripts/ios-sim.sh [--release]
#
# Workaround: dx build --ios has a bug where it uses the iPhoneOS (device) SDK
# instead of iPhoneSimulator SDK for x86_64-apple-ios on Intel Macs.
# This script builds with cargo directly and creates the app bundle manually.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLE_DIR="$PROJECT_ROOT/ios/dist/AMP.app"
BUNDLE_ID="se.malmo.skaggbyran.amp"
TARGET="x86_64-apple-ios"

PROFILE="debug"
CARGO_FLAGS=""
if [[ "${1:-}" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
fi

echo "==> Building amp-ios for $TARGET ($PROFILE)..."
cargo build -p amp-ios --target "$TARGET" $CARGO_FLAGS

echo "==> Creating app bundle..."
mkdir -p "$BUNDLE_DIR"
cp "$PROJECT_ROOT/target/$TARGET/$PROFILE/amp" "$BUNDLE_DIR/amp"

cat > "$BUNDLE_DIR/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
	<string>amp</string>
	<key>CFBundleIdentifier</key>
	<string>se.malmo.skaggbyran.amp</string>
	<key>CFBundleName</key>
	<string>AMP</string>
	<key>CFBundleDisplayName</key>
	<string>AMP</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0.0</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>iPhoneSimulator</string>
	</array>
	<key>MinimumOSVersion</key>
	<string>16.0</string>
	<key>UILaunchStoryboardName</key>
	<string></string>
	<key>UISupportedInterfaceOrientations</key>
	<array>
		<string>UIInterfaceOrientationPortrait</string>
		<string>UIInterfaceOrientationLandscapeLeft</string>
		<string>UIInterfaceOrientationLandscapeRight</string>
	</array>
	<key>DTPlatformName</key>
	<string>iphonesimulator</string>
</dict>
</plist>
PLIST

echo "==> Booting simulator..."
xcrun simctl boot booted 2>/dev/null || true

echo "==> Installing on simulator..."
xcrun simctl install booted "$BUNDLE_DIR"

echo "==> Launching..."
xcrun simctl launch booted "$BUNDLE_ID"

echo "==> Done! App running on simulator."
