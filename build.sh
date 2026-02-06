#!/usr/bin/env bash

# Amp Android Build Script
# Copies Kotlin files and updates AndroidManifest for notification support

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Amp Android Build Script ===${NC}"

# Determine Android project path
# Dioxus generates Android project at: target/dx/amp/release/android/app/app
ANDROID_PROJECT_BASE="target/dx/amp/release/android/app/app"
ANDROID_SRC="$ANDROID_PROJECT_BASE/src/main"
JAVA_DIR="$ANDROID_SRC/java/com/amp"
MANIFEST="$ANDROID_SRC/AndroidManifest.xml"

# Source files
KOTLIN_SRC="android/kotlin/NotificationHelper.kt"

# Function to check if file exists
check_file() {
    if [ ! -f "$1" ]; then
        echo -e "${RED}Error: File not found: $1${NC}"
        return 1
    fi
    return 0
}

# Function to create directory if it doesn't exist
ensure_dir() {
    if [ ! -d "$1" ]; then
        echo -e "${YELLOW}Creating directory: $1${NC}"
        mkdir -p "$1"
    fi
}

# Step 1: Check if source files exist
echo -e "\n${GREEN}[1/4] Checking source files...${NC}"
if ! check_file "$KOTLIN_SRC"; then
    echo -e "${RED}Kotlin source file not found. Please ensure you're in the project root.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Source files found${NC}"

# Step 2: Create Android project directories
echo -e "\n${GREEN}[2/4] Setting up Android project directories...${NC}"
if [ ! -d "$ANDROID_PROJECT_BASE" ]; then
    echo -e "${YELLOW}Android project not found at: $ANDROID_PROJECT_BASE${NC}"
    echo -e "${YELLOW}Building Android project first...${NC}"
    
    # Try to build with dx first
    if command -v dx &> /dev/null; then
        echo -e "${GREEN}Building with Dioxus CLI...${NC}"
        dx build --platform android --release
    else
        echo -e "${RED}Error: Dioxus CLI (dx) not found${NC}"
        echo -e "${YELLOW}Install with: cargo install dioxus-cli${NC}"
        exit 1
    fi
fi

ensure_dir "$JAVA_DIR"
echo -e "${GREEN}✓ Directories ready${NC}"

# Step 3: Copy Kotlin files
echo -e "\n${GREEN}[3/4] Copying Kotlin files...${NC}"
cp -v "$KOTLIN_SRC" "$JAVA_DIR/"
echo -e "${GREEN}✓ Kotlin files copied to: $JAVA_DIR${NC}"

# Step 4: Update AndroidManifest.xml
echo -e "\n${GREEN}[4/4] Updating AndroidManifest.xml...${NC}"

if [ ! -f "$MANIFEST" ]; then
    echo -e "${RED}Error: AndroidManifest.xml not found at: $MANIFEST${NC}"
    exit 1
fi

# Check if permissions already exist
if grep -q "android.permission.POST_NOTIFICATIONS" "$MANIFEST"; then
    echo -e "${YELLOW}Notification permissions already present in manifest${NC}"
else
    echo -e "${GREEN}Adding notification permissions to manifest...${NC}"
    
    # Create backup
    cp "$MANIFEST" "$MANIFEST.backup"
    
    # Add permissions after <manifest> tag
    # This uses sed to insert after the first <manifest> line
    sed -i '/<manifest/a\    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />\n    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />\n    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />' "$MANIFEST"
    
    echo -e "${GREEN}✓ Permissions added (backup saved as AndroidManifest.xml.backup)${NC}"
fi

# Verify the manifest contains our permissions
if grep -q "POST_NOTIFICATIONS" "$MANIFEST"; then
    echo -e "${GREEN}✓ Manifest updated successfully${NC}"
else
    echo -e "${RED}Warning: Could not verify permissions in manifest${NC}"
fi

# Step 5: Display summary
echo -e "\n${GREEN}=== Build Preparation Complete ===${NC}"
echo -e "${GREEN}Files copied:${NC}"
echo -e "  - NotificationHelper.kt → $JAVA_DIR"
echo -e "\n${GREEN}Manifest updated:${NC}"
echo -e "  - POST_NOTIFICATIONS permission added"
echo -e "  - FOREGROUND_SERVICE permissions added"

echo -e "\n${GREEN}Next steps:${NC}"
echo -e "  1. Build the APK: ${YELLOW}dx build --platform android --release${NC}"
echo -e "  2. Or run on device: ${YELLOW}dx serve --platform android${NC}"
echo -e "  3. Check logs: ${YELLOW}adb logcat | grep -E '(Notifications|AmpNotifications)'${NC}"

echo -e "\n${GREEN}Done!${NC}"
