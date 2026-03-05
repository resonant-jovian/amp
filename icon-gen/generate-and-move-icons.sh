#!/bin/bash

echo "🎨 Generating icons..."
node generate-icons.js

echo ""
echo "📦 Moving icons to project..."
DEST_ROOT=~/Documents/amp
mkdir -p "$DEST_ROOT/assets/icon"
mkdir -p "$DEST_ROOT/android/assets/icon"
mkdir -p "$DEST_ROOT/ios/assets/icon"

for DEST in "$DEST_ROOT/assets/icon" "$DEST_ROOT/android/assets/icon" "$DEST_ROOT/ios/assets/icon"; do
    mv -f icons/icon-mdpi.png "$DEST/" 2>/dev/null || cp icons/icon-mdpi.png "$DEST/"
    cp icons/icon-hdpi.png "$DEST/"
    cp icons/icon-xhdpi.png "$DEST/"
    cp icons/icon-xxhdpi.png "$DEST/"
    cp icons/icon-xxxhdpi.png "$DEST/"
    cp icons/icon-512.png "$DEST/"
done

echo ""
echo "✅ Icons moved successfully!"
echo ""
ls -lh "$DEST_ROOT/assets/icon/icon-*.png"
