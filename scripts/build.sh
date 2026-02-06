#!/usr/bin/env bash

# Amp Android Setup Script
# Prepares the Android project for notification support

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Amp Android Notification Setup       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KOTLIN_SOURCE="$PROJECT_ROOT/android/kotlin/NotificationHelper.kt"

# Default Dioxus Android path (can be overridden)
if [ -z "$ANDROID_PROJECT_PATH" ]; then
    ANDROID_PROJECT_PATH="$PROJECT_ROOT/target/dx/amp/release/android/app/app"
fi

ANDROID_SRC="$ANDROID_PROJECT_PATH/src/main"
JAVA_DIR="$ANDROID_SRC/java/com/amp"
MANIFEST="$ANDROID_SRC/AndroidManifest.xml"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --android-path)
            ANDROID_PROJECT_PATH="$2"
            ANDROID_SRC="$ANDROID_PROJECT_PATH/src/main"
            JAVA_DIR="$ANDROID_SRC/java/com/amp"
            MANIFEST="$ANDROID_SRC/AndroidManifest.xml"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --android-path PATH    Specify custom Android project path"
            echo "  --help                 Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0"
            echo "  $0 --android-path /home/albin/Documents/amp/target/dx/amp/release/android/app/app"
            echo "  ANDROID_PROJECT_PATH=/custom/path $0"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "\n${GREEN}Configuration:${NC}"
echo -e "  Project root: $PROJECT_ROOT"
echo -e "  Android path: $ANDROID_PROJECT_PATH"
echo -e "  Manifest: $ANDROID_SRC/AndroidManifest.xml"

# Function to check if file exists
check_file() {
    if [ ! -f "$1" ]; then
        echo -e "${RED}✗ File not found: $1${NC}"
        return 1
    fi
    echo -e "${GREEN}✓ Found: $1${NC}"
    return 0
}

# Function to create directory if it doesn't exist
ensure_dir() {
    if [ ! -d "$1" ]; then
        echo -e "${YELLOW}Creating directory: $1${NC}"
        mkdir -p "$1"
    else
        echo -e "${GREEN}✓ Directory exists: $1${NC}"
    fi
}

# Step 1: Verify source files
echo -e "\n${BLUE}[1/5] Verifying source files...${NC}"
if ! check_file "$KOTLIN_SOURCE"; then
    echo -e "${RED}Error: Kotlin source not found${NC}"
    echo -e "${YELLOW}Expected at: $KOTLIN_SOURCE${NC}"
    exit 1
fi

# Step 2: Check Android project
echo -e "\n${BLUE}[2/5] Checking Android project...${NC}"
if [ ! -d "$ANDROID_PROJECT_PATH" ]; then
    echo -e "${YELLOW}Android project not found at: $ANDROID_PROJECT_PATH${NC}"
    echo -e "${YELLOW}Please build the Android project first:${NC}"
    echo -e "  ${GREEN}cd $PROJECT_ROOT && dx build --platform android --release${NC}"
    echo -e "\n${YELLOW}Or specify a custom path:${NC}"
    echo -e "  ${GREEN}$0 --android-path /your/custom/path${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Android project found${NC}"

# Step 3: Create necessary directories
echo -e "\n${BLUE}[3/5] Setting up directories...${NC}"
ensure_dir "$JAVA_DIR"

# Step 4: Copy Kotlin files
echo -e "\n${BLUE}[4/5] Copying Kotlin files...${NC}"
if cp -v "$KOTLIN_SOURCE" "$JAVA_DIR/NotificationHelper.kt"; then
    echo -e "${GREEN}✓ NotificationHelper.kt copied successfully${NC}"
    echo -e "  Destination: $JAVA_DIR/NotificationHelper.kt"
else
    echo -e "${RED}✗ Failed to copy Kotlin file${NC}"
    exit 1
fi

# Step 5: Update AndroidManifest.xml
echo -e "\n${BLUE}[5/5] Updating AndroidManifest.xml...${NC}"

if [ ! -f "$MANIFEST" ]; then
    echo -e "${RED}✗ AndroidManifest.xml not found at: $MANIFEST${NC}"
    exit 1
fi

# Check if permissions already exist
HAS_POST_NOTIF=$(grep -c "android.permission.POST_NOTIFICATIONS" "$MANIFEST" || true)
HAS_FOREGROUND=$(grep -c "android.permission.FOREGROUND_SERVICE" "$MANIFEST" || true)

if [ "$HAS_POST_NOTIF" -gt 0 ]; then
    echo -e "${YELLOW}✓ POST_NOTIFICATIONS permission already present${NC}"
else
    echo -e "${GREEN}Adding POST_NOTIFICATIONS permission...${NC}"
    
    # Create backup
    cp "$MANIFEST" "$MANIFEST.backup"
    echo -e "${YELLOW}  Backup created: $MANIFEST.backup${NC}"
    
    # Add permissions after <manifest> opening tag
    sed -i '/<manifest/a\    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />' "$MANIFEST"
    echo -e "${GREEN}✓ POST_NOTIFICATIONS added${NC}"
fi

if [ "$HAS_FOREGROUND" -gt 0 ]; then
    echo -e "${YELLOW}✓ FOREGROUND_SERVICE permissions already present${NC}"
else
    echo -e "${GREEN}Adding FOREGROUND_SERVICE permissions...${NC}"
    
    # Add foreground service permissions if not already backed up
    if [ ! -f "$MANIFEST.backup" ]; then
        cp "$MANIFEST" "$MANIFEST.backup"
    fi
    
    sed -i '/<uses-permission.*POST_NOTIFICATIONS/a\    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />\n    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />' "$MANIFEST"
    echo -e "${GREEN}✓ FOREGROUND_SERVICE permissions added${NC}"
fi

# Verify permissions
echo -e "\n${GREEN}Verifying manifest...${NC}"
POST_NOTIF_COUNT=$(grep -c "POST_NOTIFICATIONS" "$MANIFEST" || true)
FOREGROUND_COUNT=$(grep -c "FOREGROUND_SERVICE" "$MANIFEST" || true)

if [ "$POST_NOTIF_COUNT" -gt 0 ] && [ "$FOREGROUND_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ All permissions present in manifest${NC}"
else
    echo -e "${RED}✗ Warning: Could not verify all permissions${NC}"
    echo -e "${YELLOW}Please check the manifest manually at: $MANIFEST${NC}"
fi

# Display manifest snippet
echo -e "\n${BLUE}Current manifest permissions:${NC}"
grep "uses-permission" "$MANIFEST" | head -n 10

# Summary
echo -e "\n${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Setup Complete!                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"

echo -e "\n${GREEN}Files modified:${NC}"
echo -e "  ✓ $JAVA_DIR/NotificationHelper.kt"
echo -e "  ✓ $MANIFEST"

echo -e "\n${GREEN}Permissions added:${NC}"
echo -e "  ✓ POST_NOTIFICATIONS (Android 13+)"
echo -e "  ✓ FOREGROUND_SERVICE"
echo -e "  ✓ FOREGROUND_SERVICE_DATA_SYNC"

echo -e "\n${YELLOW}Next steps:${NC}"
echo -e "  1. Build APK:"
echo -e "     ${GREEN}cd $PROJECT_ROOT && dx build --platform android --release${NC}"
echo -e ""
echo -e "  2. Run on device:"
echo -e "     ${GREEN}cd $PROJECT_ROOT && dx serve --platform android${NC}"
echo -e ""
echo -e "  3. Monitor notifications:"
echo -e "     ${GREEN}adb logcat | grep -E '(Notifications|AmpNotifications|amp_)'${NC}"
echo -e ""
echo -e "  4. Check channels:"
echo -e "     ${GREEN}adb shell dumpsys notification | grep amp_${NC}"

echo -e "\n${BLUE}Documentation:${NC}"
echo -e "  - Notification guide: docs/android-notifications.md"
echo -e "  - JNI integration: android/kotlin/README.md"
echo -e "  - Implementation: android/NOTIFICATIONS_IMPLEMENTATION.md"

echo -e "\n${GREEN}Setup successful! 🎉${NC}\n"
