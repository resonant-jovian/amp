#!/bin/bash

echo "🎨 Generating icons..."
node generate-icons.js

echo ""
echo "📦 Moving icons to project..."
mv icons/icon-mdpi.png ~/Documents/amp/android/assets/icon/
mv icons/icon-hdpi.png ~/Documents/amp/android/assets/icon/
mv icons/icon-xhdpi.png ~/Documents/amp/android/assets/icon/
mv icons/icon-xxhdpi.png ~/Documents/amp/android/assets/icon/
mv icons/icon-xxxhdpi.png ~/Documents/amp/android/assets/icon/
mv icons/icon-512.png ~/Documents/amp/android/assets/icon/

echo ""
echo "✅ Icons moved successfully!"
echo ""
ls -lh ~/Documents/amp/android/assets/icon/icon-*.png

