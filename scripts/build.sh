#!/bin/bash
set -e

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "📍 Project root: $REPO_ROOT"

# ========== PLATFORM SELECTION ==========
PLATFORM="${1:-}"
if [ -z "$PLATFORM" ]; then
    echo "Select build target:"
    echo "  1) android"
    echo "  2) ios"
    echo "  3) both"
    read -r -p "Enter choice [1-3]: " CHOICE
    case "$CHOICE" in
        1) PLATFORM="android" ;;
        2) PLATFORM="ios" ;;
        3) PLATFORM="both" ;;
        *) echo "Invalid choice"; exit 1 ;;
    esac
fi
# ========== END PLATFORM SELECTION ==========

# ========== iOS BUILD FUNCTION ==========
build_ios() {
    echo ""
    echo "🍎 iOS build..."
    echo "⚠️  Full iOS build requires macOS + Xcode. Checking Rust compilation only."
    cd "$REPO_ROOT/ios" || exit 1
    if rustup target list --installed | grep -q "aarch64-apple-ios"; then
        cargo build --target aarch64-apple-ios --release && \
            echo "✅ iOS Rust compilation successful" || \
            echo "⚠️  iOS Rust compilation failed"
    else
        echo "⚠️  Target aarch64-apple-ios not installed."
        echo "   Run: rustup target add aarch64-apple-ios"
    fi
    cd "$REPO_ROOT"
}
# ========== END iOS BUILD FUNCTION ==========

# Dispatch to platform-specific build
case "$PLATFORM" in
    ios)
        build_ios
        exit 0
        ;;
    both)
        # Android runs below; iOS runs at end
        ;;
    android)
        # Android runs below
        ;;
    *)
        echo "Unknown platform: $PLATFORM"
        exit 1
        ;;
esac

echo "🔨 Building Dioxus Android APK..."

# Go to android directory
cd "$REPO_ROOT/android" || {
    echo "❌ android directory not found at $REPO_ROOT/android"
    exit 1
}

# ========== PRE-BUILD VERIFICATION ==========
verify_package_structure() {
    echo ""
    echo "🔍 PRE-BUILD: Verifying package structure consistency..."
    
    local KOTLIN_SRC="$REPO_ROOT/android/kotlin"
    local ISSUES=0
    
    # Check NotificationHelper
    if [ -f "$KOTLIN_SRC/NotificationHelper.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/NotificationHelper.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ NotificationHelper.kt: package=$PACKAGE"
        else
            echo "  ❌ NotificationHelper.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ NotificationHelper.kt not found"
        ISSUES=$((ISSUES + 1))
    fi
    
    # Check WebViewConfigurator
    if [ -f "$KOTLIN_SRC/WebViewConfigurator.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/WebViewConfigurator.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ WebViewConfigurator.kt: package=$PACKAGE"
        else
            echo "  ❌ WebViewConfigurator.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
        
        # Verify it has the configure method
        if grep -q "fun configure(webView: WebView)" "$KOTLIN_SRC/WebViewConfigurator.kt"; then
            echo "  ✅ WebViewConfigurator.configure() method present"
        else
            echo "  ❌ WebViewConfigurator.configure() method missing!"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ WebViewConfigurator.kt not found"
        ISSUES=$((ISSUES + 1))
    fi
    
    # Check MainActivity
    if [ -f "$KOTLIN_SRC/MainActivity.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/MainActivity.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "dev.dioxus.main" ]; then
            echo "  ✅ MainActivity.kt: package=$PACKAGE"
        else
            echo "  ❌ MainActivity.kt: WRONG PACKAGE ($PACKAGE != dev.dioxus.main)"
            ISSUES=$((ISSUES + 1))
        fi
        
        # Verify it extends WryActivity
        if grep -q "class MainActivity : WryActivity" "$KOTLIN_SRC/MainActivity.kt"; then
            echo "  ✅ MainActivity extends WryActivity"
        else
            echo "  ❌ MainActivity does not extend WryActivity!"
            ISSUES=$((ISSUES + 1))
        fi
        
        # Verify it calls WebViewConfigurator (direct or via constants)
        if grep -q "WebViewConfigurator\|CONFIGURATOR_CLASS" "$KOTLIN_SRC/MainActivity.kt"; then
            echo "  ✅ MainActivity calls WebViewConfigurator.configure()"
        else
            echo "  ❌ MainActivity does not call WebViewConfigurator!"
            ISSUES=$((ISSUES + 1))
        fi
        
        # Verify it has onWebViewCreate override
        if grep -q "override fun onWebViewCreate" "$KOTLIN_SRC/MainActivity.kt"; then
            echo "  ✅ MainActivity overrides onWebViewCreate()"
        else
            echo "  ❌ MainActivity missing onWebViewCreate() override!"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ MainActivity.kt not found"
        ISSUES=$((ISSUES + 1))
    fi
    
    # Check DormantService
    if [ -f "$KOTLIN_SRC/DormantService.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/DormantService.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ DormantService.kt: package=$PACKAGE"
        else
            echo "  ❌ DormantService.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ DormantService.kt not found"
        ISSUES=$((ISSUES + 1))
    fi

    # Check BootReceiver
    if [ -f "$KOTLIN_SRC/BootReceiver.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/BootReceiver.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ BootReceiver.kt: package=$PACKAGE"
        else
            echo "  ❌ BootReceiver.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ BootReceiver.kt not found"
        ISSUES=$((ISSUES + 1))
    fi

    # Check DormantBridge
    if [ -f "$KOTLIN_SRC/DormantBridge.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/DormantBridge.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ DormantBridge.kt: package=$PACKAGE"
        else
            echo "  ❌ DormantBridge.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ DormantBridge.kt not found"
        ISSUES=$((ISSUES + 1))
    fi

    # Check LocationHelper
    if [ -f "$KOTLIN_SRC/LocationHelper.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/LocationHelper.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ LocationHelper.kt: package=$PACKAGE"
        else
            echo "  ❌ LocationHelper.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ LocationHelper.kt not found"
        ISSUES=$((ISSUES + 1))
    fi

    # Check FilePickerHelper
    if [ -f "$KOTLIN_SRC/FilePickerHelper.kt" ]; then
        local PACKAGE=$(grep "^package " "$KOTLIN_SRC/FilePickerHelper.kt" | awk '{print $2}' | tr -d ';')
        if [ "$PACKAGE" = "se.malmo.skaggbyran.amp" ]; then
            echo "  ✅ FilePickerHelper.kt: package=$PACKAGE"
        else
            echo "  ❌ FilePickerHelper.kt: WRONG PACKAGE ($PACKAGE != se.malmo.skaggbyran.amp)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ FilePickerHelper.kt not found"
        ISSUES=$((ISSUES + 1))
    fi

    # Check ProGuard rules file exists
    if [ -f "$REPO_ROOT/android/proguard/proguard-rules.pro" ]; then
        echo "  ✅ ProGuard rules file found"

        # Verify it contains -dontobfuscate
        if grep -q "^-dontobfuscate" "$REPO_ROOT/android/proguard/proguard-rules.pro"; then
            echo "  ✅ ProGuard rules contain -dontobfuscate flag"
        else
            echo "  ⚠️  ProGuard rules missing -dontobfuscate flag (classes will be obfuscated!)"
            ISSUES=$((ISSUES + 1))
        fi
    else
        echo "  ❌ ProGuard rules file not found at $REPO_ROOT/android/proguard/proguard-rules.pro"
        ISSUES=$((ISSUES + 1))
    fi
    
    echo ""
    if [ "$ISSUES" -eq 0 ]; then
        echo "  ✅ Package structure verification PASSED"
        return 0
    else
        echo "  ❌ Package structure verification FAILED ($ISSUES issues)"
        echo ""
        echo "  Fix these issues before building:"
        echo "  1. Ensure all .kt files have correct package declarations"
        echo "  2. Ensure MainActivity extends WryActivity and overrides onWebViewCreate()"
        echo "  3. Ensure MainActivity calls WebViewConfigurator.configure()"
        echo "  4. Ensure ProGuard rules file exists with -dontobfuscate flag"
        return 1
    fi
}

# Run pre-build verification
if ! verify_package_structure; then
    exit 1
fi
# ========== END PRE-BUILD VERIFICATION ==========

# Load keystore settings
echo ""
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
asset_dir = "../assets"

[bundle]
publisher = "Skäggbyrån Malmö"
identifier = "se.malmo.skaggbyran.amp"
icon = ["assets/icon/icon-512.png"]
resources = [
    "assets/data/db.parquet",
    "assets/style.css",
    "assets/fonts/**/*"
]

[bundle.android]
publisher = "Skäggbyrån Malmö"
identifier = "se.malmo.skaggbyran.amp"
icon = ["assets/icon/icon-512.png"]
resources = [
    "assets/data/db.parquet",
    "assets/style.css",
    "assets/fonts/**/*"
]
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
# NOTE: Dioxus creates nested structure at .../android/app/app/
# The build.gradle.kts is at .../android/app/app/build.gradle.kts
ANDROID_DIR="$REPO_ROOT/target/dx/amp/release/android/app/app"
rm -rf -- "$ANDROID_DIR" 2>/dev/null || true
rm -rf -- "$REPO_ROOT/android/app/.gradle" 2>/dev/null || true
rm -rf -- "$REPO_ROOT/android/app/build" 2>/dev/null || true
pkill -9 gradle java 2>/dev/null || true
sleep 1

# ========== NOTIFICATION SETUP FUNCTION ==========
setup_notifications() {
    echo ""
    echo "🔔 Setting up notification system..."
    
    ANDROID_SRC="$ANDROID_DIR/src/main"
    KOTLIN_DIR="$ANDROID_SRC/kotlin/se/malmo/skaggbyran/amp"
    MANIFEST="$ANDROID_SRC/AndroidManifest.xml"
    BUILD_GRADLE="$ANDROID_DIR/build.gradle.kts"
    PROGUARD_RULES="$ANDROID_DIR/proguard-rules.pro"
    KOTLIN_SOURCE="$REPO_ROOT/android/kotlin/NotificationHelper.kt"
    WEBVIEW_SOURCE="$REPO_ROOT/android/kotlin/WebViewConfigurator.kt"
    DORMANT_SERVICE_SOURCE="$REPO_ROOT/android/kotlin/DormantService.kt"
    BOOT_RECEIVER_SOURCE="$REPO_ROOT/android/kotlin/BootReceiver.kt"
    DORMANT_BRIDGE_SOURCE="$REPO_ROOT/android/kotlin/DormantBridge.kt"
    LOCATION_HELPER_SOURCE="$REPO_ROOT/android/kotlin/LocationHelper.kt"
    FILEPICKER_SOURCE="$REPO_ROOT/android/kotlin/FilePickerHelper.kt"

    # Create Kotlin directory matching package structure
    if [ ! -d "$KOTLIN_DIR" ]; then
        echo "  📁 Creating directory: $KOTLIN_DIR"
        mkdir -p "$KOTLIN_DIR"
    fi
    
    # Copy NotificationHelper.kt if it exists
    if [ -f "$KOTLIN_SOURCE" ]; then
        echo "  📄 Copying NotificationHelper.kt to kotlin/ directory..."
        cp "$KOTLIN_SOURCE" "$KOTLIN_DIR/NotificationHelper.kt"
        echo "  ✓ NotificationHelper.kt copied to $KOTLIN_DIR"
    else
        echo "  ❌ NotificationHelper.kt not found at $KOTLIN_SOURCE"
        exit 1
    fi
    
    # Copy WebViewConfigurator.kt
    if [ -f "$WEBVIEW_SOURCE" ]; then
        echo "  📄 Copying WebViewConfigurator.kt to kotlin/ directory..."
        cp "$WEBVIEW_SOURCE" "$KOTLIN_DIR/WebViewConfigurator.kt"
        echo "  ✓ WebViewConfigurator.kt copied (fixes blank screen)"
    else
        echo "  ⚠️  WebViewConfigurator.kt not found at $WEBVIEW_SOURCE"
        echo "     App may show blank screen without DOM storage enabled"
    fi

    # Copy DormantService.kt
    if [ -f "$DORMANT_SERVICE_SOURCE" ]; then
        echo "  📄 Copying DormantService.kt to kotlin/ directory..."
        cp "$DORMANT_SERVICE_SOURCE" "$KOTLIN_DIR/DormantService.kt"
        echo "  ✓ DormantService.kt copied (background monitoring)"
    else
        echo "  ❌ DormantService.kt not found at $DORMANT_SERVICE_SOURCE"
        exit 1
    fi

    # Copy BootReceiver.kt
    if [ -f "$BOOT_RECEIVER_SOURCE" ]; then
        echo "  📄 Copying BootReceiver.kt to kotlin/ directory..."
        cp "$BOOT_RECEIVER_SOURCE" "$KOTLIN_DIR/BootReceiver.kt"
        echo "  ✓ BootReceiver.kt copied (auto-start on boot)"
    else
        echo "  ❌ BootReceiver.kt not found at $BOOT_RECEIVER_SOURCE"
        exit 1
    fi

    # Copy DormantBridge.kt
    if [ -f "$DORMANT_BRIDGE_SOURCE" ]; then
        echo "  📄 Copying DormantBridge.kt to kotlin/ directory..."
        cp "$DORMANT_BRIDGE_SOURCE" "$KOTLIN_DIR/DormantBridge.kt"
        echo "  ✓ DormantBridge.kt copied (JNI bridge to Rust)"
    else
        echo "  ❌ DormantBridge.kt not found at $DORMANT_BRIDGE_SOURCE"
        exit 1
    fi

    # Copy LocationHelper.kt
    if [ -f "$LOCATION_HELPER_SOURCE" ]; then
        echo "  📄 Copying LocationHelper.kt to kotlin/ directory..."
        cp "$LOCATION_HELPER_SOURCE" "$KOTLIN_DIR/LocationHelper.kt"
        echo "  ✓ LocationHelper.kt copied (GPS location reading)"
    else
        echo "  ❌ LocationHelper.kt not found at $LOCATION_HELPER_SOURCE"
        exit 1
    fi

    # Copy FilePickerHelper.kt
    if [ -f "$FILEPICKER_SOURCE" ]; then
        echo "  📄 Copying FilePickerHelper.kt to kotlin/ directory..."
        cp "$FILEPICKER_SOURCE" "$KOTLIN_DIR/FilePickerHelper.kt"
        echo "  ✓ FilePickerHelper.kt copied (SAF import/export)"
    else
        echo "  ❌ FilePickerHelper.kt not found at $FILEPICKER_SOURCE"
        exit 1
    fi

    # ========== CRITICAL: Replace auto-generated MainActivity ==========
    echo ""
    echo "  🔧 CRITICAL: Replacing auto-generated MainActivity with custom version..."
    echo "     This adds WebView configuration via onWebViewCreate() hook"
    
    MAINACTIVITY_SOURCE="$REPO_ROOT/android/kotlin/MainActivity.kt"
    DIOXUS_MAINACTIVITY_DIR="$ANDROID_SRC/kotlin/dev/dioxus/main"
    DIOXUS_MAINACTIVITY="$DIOXUS_MAINACTIVITY_DIR/MainActivity.kt"
    
    # Create dev.dioxus.main directory if it doesn't exist
    if [ ! -d "$DIOXUS_MAINACTIVITY_DIR" ]; then
        echo "    📁 Creating directory: $DIOXUS_MAINACTIVITY_DIR"
        mkdir -p "$DIOXUS_MAINACTIVITY_DIR"
    fi
    
    # Replace auto-generated MainActivity with our custom version
    if [ -f "$MAINACTIVITY_SOURCE" ]; then
        echo "    📄 Replacing MainActivity.kt in dev.dioxus.main package..."
        cp "$MAINACTIVITY_SOURCE" "$DIOXUS_MAINACTIVITY"
        echo "    ✓ Custom MainActivity.kt installed (calls WebViewConfigurator.configure())"
        echo "    ✓ Will use WryActivity.onWebViewCreate() hook for configuration"
    else
        echo "    ❌ MainActivity.kt not found at $MAINACTIVITY_SOURCE"
        echo "    ⚠️  Using auto-generated MainActivity (no WebView configuration)"
        exit 1
    fi
    
    # Verify the replacement
    if grep -q "WebViewConfigurator" "$DIOXUS_MAINACTIVITY"; then
        echo "    ✅ SUCCESS: Custom MainActivity verified (contains WebViewConfigurator call)"
    else
        echo "    ❌ FATAL: MainActivity does not call WebViewConfigurator!"
        echo "    App will show blank screen without DOM storage configuration"
        exit 1
    fi
    # ========== END MAINACTIVITY REPLACEMENT ==========
    
    # ========== ENHANCED: Validate source sets configuration ==========
    echo ""
    echo "  🔧 CRITICAL FIX: Registering Kotlin source directory in build.gradle.kts..."
    echo "     This fixes ClassNotFoundException for NotificationHelper + WebViewConfigurator + MainActivity"
    
    if [ -f "$BUILD_GRADLE" ]; then
        # Check if sourceSets already exists
        if grep -q "sourceSets {" "$BUILD_GRADLE"; then
            echo "    ⚠️  sourceSets block already exists"
            echo "    Verifying kotlin directory is included..."
            
            if ! grep -q "src/main/kotlin" "$BUILD_GRADLE"; then
                echo "    ❌ kotlin directory NOT registered!"
                echo "    Attempting to add kotlin directory..."
                
                # Backup before modification
                cp "$BUILD_GRADLE" "$BUILD_GRADLE.backup"
                
                # Try to modify existing java.srcDirs line to include kotlin
                if grep -q 'java\.srcDirs' "$BUILD_GRADLE"; then
                    sed -i '/java\.srcDirs/ s/)$/, "src\/main\/kotlin")/' "$BUILD_GRADLE" 2>/dev/null || {
                        echo "    ⚠️  sed replacement failed, restoring backup"
                        mv "$BUILD_GRADLE.backup" "$BUILD_GRADLE"
                    }
                fi
                
                rm -f "$BUILD_GRADLE.backup"
            else
                echo "    ✓ kotlin directory already registered"
            fi
        else
            echo "    📝 Injecting sourceSets block into android {} block..."
            
            # Insert sourceSets block after 'android {' line
            if grep -q '^android {' "$BUILD_GRADLE"; then
                sed -i '/^android {$/a\    sourceSets {\n        getByName("main") {\n            java.srcDirs("src/main/java", "src/main/kotlin")\n        }\n    }\n' "$BUILD_GRADLE"
            else
                # Fallback for different formatting
                sed -i '/android\s*{/a\    sourceSets {\n        getByName("main") {\n            java.srcDirs("src/main/java", "src/main/kotlin")\n        }\n    }\n' "$BUILD_GRADLE"
            fi
            
            echo "    ✓ sourceSets block injected"
        fi
        
        # Verify the fix was applied
        echo ""
        echo "    🔍 Verifying Kotlin source directory registration..."
        if grep -q "src/main/kotlin" "$BUILD_GRADLE"; then
            echo "    ✅ SUCCESS: Kotlin source directory registered in build.gradle.kts"
            echo "       All Kotlin classes (NotificationHelper, WebViewConfigurator, MainActivity) will be compiled"
            
            # Show the actual configuration
            echo ""
            echo "    📋 Current source directory configuration:"
            grep -B 2 -A 5 "src/main/kotlin" "$BUILD_GRADLE" | head -n 10 || echo "    (Could not extract for display)"
        else
            echo "    ❌ CRITICAL FAILURE: Could not register Kotlin source directory"
            echo "    ❌ Build will fail with ClassNotFoundException at runtime"
            exit 1
        fi
        
        # ========== CRITICAL: Inject Gradle syncKotlinSources task ==========
        echo ""
        echo "    🔧 ULTIMATE FIX: Injecting Gradle task to sync Kotlin files BEFORE compilation..."
        echo "       This ensures files exist when Gradle scans sources (fixes race condition)"
        
        # Check if task already exists
        if grep -q 'tasks.register<Copy>("syncKotlinSources")' "$BUILD_GRADLE"; then
            echo "    ✅ syncKotlinSources task already exists"
        else
            # Append task at end of build.gradle.kts
            cat >> "$BUILD_GRADLE" << 'GRADLE_TASK'

// ========== CRITICAL: Sync Kotlin sources before build ==========
// This fixes ClassNotFoundException by ensuring Kotlin files exist
// when Gradle scans source directories (before compilation)
tasks.register<Copy>("syncKotlinSources") {
    description = "Copy custom Kotlin files before build (fixes ClassNotFoundException)"
    group = "build setup"
    
    // Source: our custom Kotlin files
    from("../../../../../android/kotlin") {
        include("**/*.kt")
    }
    
    // Destination: Gradle source directory (package structure will be created)
    into("src/main/kotlin")
    
    doFirst {
        println("🔄 Syncing Kotlin sources from android/kotlin/...")
    }
    
    doLast {
        println("✅ Kotlin sources synced - NotificationHelper, WebViewConfigurator, MainActivity, DormantService, DormantBridge, BootReceiver, FilePickerHelper")
    }
}

// Make preBuild depend on our sync task
tasks.named("preBuild") {
    dependsOn("syncKotlinSources")
}
GRADLE_TASK
            
            echo "    ✅ syncKotlinSources task injected successfully"
            echo "    ✅ preBuild now depends on syncKotlinSources"
            echo ""
            echo "    📋 Task details:"
            echo "       - Runs BEFORE preBuild (before Gradle source scan)"
            echo "       - Copies android/kotlin/*.kt to src/main/kotlin/"
            echo "       - Ensures files exist during compilation phase"
            echo "       - Automatic on every build"
        fi
        
        # Verify task was added
        if grep -q 'syncKotlinSources' "$BUILD_GRADLE"; then
            echo ""
            echo "    ✅ VERIFICATION: syncKotlinSources task present in build.gradle.kts"
        else
            echo ""
            echo "    ❌ FATAL: Failed to inject syncKotlinSources task"
            exit 1
        fi
        # ========== END GRADLE TASK INJECTION ==========
        
        # Additional check: Verify both packages will be included
        echo ""
        echo "    📦 Verifying package inclusion..."
        if [ -d "$ANDROID_SRC/kotlin/se/malmo/skaggbyran/amp" ] && [ -d "$ANDROID_SRC/kotlin/dev/dioxus/main" ]; then
            echo "    ✅ Both packages present:"
            echo "       - se.malmo.skaggbyran.amp (NotificationHelper, WebViewConfigurator)"
            echo "       - dev.dioxus.main (MainActivity)"
        else
            echo "    ⚠️  Package directories not fully created yet"
        fi
    else
        echo "    ❌ build.gradle.kts not found at $BUILD_GRADLE"
        exit 1
    fi
    # ========== END SOURCE SETS VALIDATION ==========
    
    # ========== CRITICAL: Copy comprehensive ProGuard rules ==========
    echo ""
    echo "  🔒 CRITICAL FIX: Installing comprehensive ProGuard rules..."
    echo "     Prevents R8 obfuscation of NotificationHelper + WebViewConfigurator + MainActivity"
    
    PROGUARD_SOURCE="$REPO_ROOT/android/proguard/proguard-rules.pro"
    
    if [ -f "$PROGUARD_SOURCE" ]; then
        echo "    📄 Copying ProGuard rules from $PROGUARD_SOURCE"
        cp "$PROGUARD_SOURCE" "$PROGUARD_RULES"
        echo "    ✅ ProGuard rules installed"
        
        # Verify critical rules are present
        if grep -q "^-dontobfuscate" "$PROGUARD_RULES"; then
            echo "    ✅ -dontobfuscate flag present (disables all obfuscation)"
        else
            echo "    ⚠️  -dontobfuscate not found (classes may be obfuscated)"
        fi
        
        if grep -q "NotificationHelper" "$PROGUARD_RULES"; then
            echo "    ✅ NotificationHelper keep rule present"
        fi
        
        if grep -q "WebViewConfigurator" "$PROGUARD_RULES"; then
            echo "    ✅ WebViewConfigurator keep rule present"
        fi
        
        if grep -q "dev.dioxus.main.MainActivity" "$PROGUARD_RULES"; then
            echo "    ✅ MainActivity keep rule present"
        fi
        
        if grep -q "printmapping" "$PROGUARD_RULES"; then
            echo "    ✅ R8 diagnostics enabled (mapping.txt, seeds.txt, usage.txt)"
        fi
    else
        echo "    ❌ ProGuard rules source not found at $PROGUARD_SOURCE"
        echo "    This will cause ClassNotFoundException at runtime!"
        exit 1
    fi
    # ========== END PROGUARD ==========


    # Add notification permissions to manifest if not already present
    if [ -f "$MANIFEST" ]; then
        HAS_POST_NOTIF=$(grep -c "android.permission.POST_NOTIFICATIONS" "$MANIFEST" || true)
        HAS_FOREGROUND=$(grep -c "android.permission.FOREGROUND_SERVICE" "$MANIFEST" || true)
        
        if [ "$HAS_POST_NOTIF" -eq 0 ]; then
            echo "  📝 Adding POST_NOTIFICATIONS permission..."
            sed -i '/<manifest/a\    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />' "$MANIFEST"
            echo "  ✓ POST_NOTIFICATIONS added"
        else
            echo "  ✓ POST_NOTIFICATIONS already present"
        fi
        
        if [ "$HAS_FOREGROUND" -eq 0 ]; then
            echo "  📝 Adding FOREGROUND_SERVICE permissions..."
            sed -i '/<uses-permission.*POST_NOTIFICATIONS/a\    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />\n    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_DATA_SYNC" />' "$MANIFEST"
            echo "  ✓ FOREGROUND_SERVICE permissions added"
        else
            echo "  ✓ FOREGROUND_SERVICE permissions already present"
        fi

        # Add RECEIVE_BOOT_COMPLETED permission
        HAS_BOOT=$(grep -c "android.permission.RECEIVE_BOOT_COMPLETED" "$MANIFEST" || true)
        if [ "$HAS_BOOT" -eq 0 ]; then
            echo "  📝 Adding RECEIVE_BOOT_COMPLETED permission..."
            sed -i '/<uses-permission.*FOREGROUND_SERVICE_DATA_SYNC/a\    <uses-permission android:name="android.permission.RECEIVE_BOOT_COMPLETED" />' "$MANIFEST"
            echo "  ✓ RECEIVE_BOOT_COMPLETED added"
        else
            echo "  ✓ RECEIVE_BOOT_COMPLETED already present"
        fi

        # Add GPS location permissions
        HAS_FINE_LOC=$(grep -c "android.permission.ACCESS_FINE_LOCATION" "$MANIFEST" || true)
        if [ "$HAS_FINE_LOC" -eq 0 ]; then
            echo "  📝 Adding GPS location permissions..."
            sed -i '/<uses-permission.*RECEIVE_BOOT_COMPLETED/a\    <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />\n    <uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />' "$MANIFEST"
            echo "  ✓ ACCESS_FINE_LOCATION and ACCESS_COARSE_LOCATION added"
        else
            echo "  ✓ GPS location permissions already present"
        fi

        # Register DormantService and BootReceiver in manifest
        HAS_DORMANT_SERVICE=$(grep -c "DormantService" "$MANIFEST" || true)
        if [ "$HAS_DORMANT_SERVICE" -eq 0 ]; then
            echo "  📝 Registering DormantService and BootReceiver in manifest..."
            sed -i '/<\/application>/i\        <service\n            android:name="se.malmo.skaggbyran.amp.DormantService"\n            android:foregroundServiceType="dataSync"\n            android:exported="false" />\n\n        <receiver\n            android:name="se.malmo.skaggbyran.amp.BootReceiver"\n            android:exported="true">\n            <intent-filter>\n                <action android:name="android.intent.action.BOOT_COMPLETED" />\n            </intent-filter>\n        </receiver>' "$MANIFEST"
            echo "  ✓ DormantService and BootReceiver registered"
        else
            echo "  ✓ DormantService already registered"
        fi
        
#        # ========== CRITICAL: REMOVE INTERNET PERMISSION ==========
#        echo ""
#        echo "  🔒 SECURITY: Removing INTERNET permission (added by WRY/Dioxus)..."
#        if grep -q "android.permission.INTERNET" "$MANIFEST"; then
#            sed -i '/android.permission.INTERNET/d' "$MANIFEST"
#            echo "  ✓ INTERNET permission removed (security requirement)"
#        else
#            echo "  ✓ INTERNET permission not present (good)"
#        fi
#
#        # Also remove network_security_config if present
#        if grep -q "networkSecurityConfig" "$MANIFEST"; then
#            sed -i 's/android:networkSecurityConfig="@xml\/network_security_config"//g' "$MANIFEST"
#            echo "  ✓ networkSecurityConfig reference removed"
#        fi
#        # ========== END INTERNET REMOVAL ==========
        
        echo ""
        echo "  ✅ Notification system configured"
        echo "  ✅ WebView blank screen fix applied (MainActivity + WebViewConfigurator)"
    else
        echo "  ⚠️  Manifest not found at $MANIFEST"
    fi

    # Fix app name to lowercase "amp" (Dioxus auto-capitalizes to "Amp")
    local STRINGS_XML="$ANDROID_DIR/src/main/res/values/strings.xml"
    if [ -f "$STRINGS_XML" ]; then
        sed -i 's/<string name="app_name">.*<\/string>/<string name="app_name">amp<\/string>/g' "$STRINGS_XML"
        echo "  ✅ App name set to lowercase 'amp'"
    fi

    # Fix styles.xml to use plain AppCompat theme (no splash)
    local STYLES_XML="$ANDROID_DIR/src/main/res/values/styles.xml"
    if [ -f "$STYLES_XML" ]; then
        cat > "$STYLES_XML" << 'STYLES_EOF'
<resources>
    <style name="AppTheme" parent="Theme.AppCompat.Light.NoActionBar">
    </style>
</resources>
STYLES_EOF
        echo "  ✅ styles.xml updated"
    fi
}
# ========== END NOTIFICATION SETUP ==========

# ========== PATCH HELPER FUNCTIONS ==========
patch_java21_gradle() {
    echo "🔧 Fixing generated gradle files for Java 21..."

    # Fix root build.gradle.kts (parent level)
    local ROOT_BUILD_GRADLE="$(dirname "$ANDROID_DIR")/build.gradle.kts"
    if [ -f "$ROOT_BUILD_GRADLE" ]; then
        echo "  Patching: build.gradle.kts (root)"
        sed -i 's/VERSION_1_8/VERSION_21/g' "$ROOT_BUILD_GRADLE" 2>/dev/null || true
        sed -i 's/jvmTarget = "1.8"/jvmTarget = "21"/g' "$ROOT_BUILD_GRADLE" 2>/dev/null || true
        echo "✓ Fixed root build.gradle.kts"
    fi

    # Fix app/build.gradle.kts (CRITICAL - comprehensive fix)
    if [ -f "$ANDROID_DIR/build.gradle.kts" ]; then
        echo "  Patching: build.gradle.kts (app module)"

        # Fix ALL Java version references
        sed -i 's/VERSION_1_8/VERSION_21/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true
        sed -i 's/JavaVersion\.VERSION_1_8/JavaVersion.VERSION_21/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true

        # Fix Kotlin JVM target
        sed -i 's/jvmTarget = "1.8"/jvmTarget = "21"/g' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true

        # CRITICAL: Fix compileOptions block (AGP's Java compiler settings)
        if grep -q "compileOptions {" "$ANDROID_DIR/build.gradle.kts"; then
            sed -i '/compileOptions {/,/}/ {
                s/sourceCompatibility = JavaVersion\.VERSION_1_8/sourceCompatibility = JavaVersion.VERSION_21/g
                s/targetCompatibility = JavaVersion\.VERSION_1_8/targetCompatibility = JavaVersion.VERSION_21/g
            }' "$ANDROID_DIR/build.gradle.kts" 2>/dev/null || true
        fi
        if ! grep -q "compileOptions {" "$ANDROID_DIR/build.gradle.kts"; then
            echo "    📝 Injecting compileOptions block..."
            sed -i '/^android {/a\    compileOptions {\n        sourceCompatibility = JavaVersion.VERSION_21\n        targetCompatibility = JavaVersion.VERSION_21\n    }' "$ANDROID_DIR/build.gradle.kts"
            echo "    ✓ Injected compileOptions with Java 21"
        fi
        echo "✓ Fixed app/build.gradle.kts (Java + Kotlin)"
    fi

    # Verify the fixes worked
    echo ""
    echo "📋 Verifying fixes:"
    if [ -f "$ANDROID_DIR/build.gradle.kts" ]; then
        if grep -q "VERSION_21\|jvmTarget = \"21\"" "$ANDROID_DIR/build.gradle.kts"; then
            echo "✓ App build.gradle.kts uses Java 21"
        else
            echo "⚠️  App build.gradle.kts may not be fixed"
        fi
    fi
}

fix_manifest_extract_native() {
    echo "🔧 Fixing Android manifest extractNativeLibs..."
    local MANIFEST_FILE="$ANDROID_DIR/src/main/AndroidManifest.xml"
    if [ -f "$MANIFEST_FILE" ]; then
        sed -i 's/ android:extractNativeLibs="false"//g' "$MANIFEST_FILE"
        sed -i 's/ android:extractNativeLibs="true"//g' "$MANIFEST_FILE"
        echo "✓ Fixed manifest extractNativeLibs"
    fi
}

inject_icons() {
    echo "🎨 Injecting custom app icons..."

    local RES_DIR="$ANDROID_DIR/src/main/res"
    local ICON_SOURCE="$REPO_ROOT/assets/icon"
    local MANIFEST_FILE="$ANDROID_DIR/src/main/AndroidManifest.xml"

    # 1. CRITICAL: Remove ALL existing ic_launcher* files
    echo "  🗑️  Removing all existing ic_launcher* files..."
    find "$RES_DIR" -type f \
      \( -name "ic_launcher.png" -o \
         -name "ic_launcher.webp" -o \
         -name "ic_launcher_round.*" -o \
         -name "ic_launcher_foreground.*" -o \
         -name "ic_launcher_background.*" -o \
         -name "ic_launcher.xml" \) \
      -delete 2>/dev/null || true
    echo "  ✓ Removed all auto-generated launcher icons"

    # 2. Create mipmap directories
    mkdir -p "$RES_DIR/mipmap-mdpi" \
             "$RES_DIR/mipmap-hdpi" \
             "$RES_DIR/mipmap-xhdpi" \
             "$RES_DIR/mipmap-xxhdpi" \
             "$RES_DIR/mipmap-xxxhdpi"

    # 3. Copy PNG icons
    [ -f "$ICON_SOURCE/icon-mdpi.png" ] && cp "$ICON_SOURCE/icon-mdpi.png" "$RES_DIR/mipmap-mdpi/ic_launcher.png" && echo "  ✓ Copied mdpi icon (48x48)"
    [ -f "$ICON_SOURCE/icon-hdpi.png" ] && cp "$ICON_SOURCE/icon-hdpi.png" "$RES_DIR/mipmap-hdpi/ic_launcher.png" && echo "  ✓ Copied hdpi icon (72x72)"
    [ -f "$ICON_SOURCE/icon-xhdpi.png" ] && cp "$ICON_SOURCE/icon-xhdpi.png" "$RES_DIR/mipmap-xhdpi/ic_launcher.png" && echo "  ✓ Copied xhdpi icon (96x96)"
    [ -f "$ICON_SOURCE/icon-xxhdpi.png" ] && cp "$ICON_SOURCE/icon-xxhdpi.png" "$RES_DIR/mipmap-xxhdpi/ic_launcher.png" && echo "  ✓ Copied xxhdpi icon (144x144)"
    [ -f "$ICON_SOURCE/icon-xxxhdpi.png" ] && cp "$ICON_SOURCE/icon-xxxhdpi.png" "$RES_DIR/mipmap-xxxhdpi/ic_launcher.png" && echo "  ✓ Copied xxxhdpi icon (192x192)"

    echo "  ✅ Custom launcher icons injected"

    # 4. Force manifest to use mipmap
    echo ""
    echo "🔧 Forcing AndroidManifest.xml to use @mipmap/ic_launcher..."

    if [ -f "$MANIFEST_FILE" ]; then
        if grep -q 'android:icon=' "$MANIFEST_FILE"; then
            sed -i 's/android:icon="[^"]*"/android:icon="@mipmap\/ic_launcher"/' "$MANIFEST_FILE"
            echo "  ✓ Updated android:icon"
        else
            sed -i 's/<application /<application android:icon="@mipmap\/ic_launcher" /' "$MANIFEST_FILE"
            echo "  ✓ Added android:icon"
        fi

        if grep -q 'android:roundIcon=' "$MANIFEST_FILE"; then
            sed -i 's/ android:roundIcon="[^"]*"//g' "$MANIFEST_FILE"
            echo "  ✓ Removed roundIcon"
        fi

        echo ""
        echo "  📋 Manifest <application> tag:"
        grep -A 3 "<application" "$MANIFEST_FILE" | head -n 4
    fi

    echo ""
    echo "✅ Icon injection complete!"
}

update_gradle_properties() {
    echo "🔧 Updating gradle.properties..."
    local GRADLE_PROPS="$ANDROID_DIR/gradle.properties"
    cat >> "$GRADLE_PROPS" << 'GRADLE_EOF'

# Suppress Java 8 deprecation warnings
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
}
# ========== END PATCH HELPERS ==========

# Generate Gradle project — always continue even if dx fails
echo "📦 Running dx build to generate Gradle project..."
dx build --android --release --verbose || true

if [ ! -d "$ANDROID_DIR" ]; then
    echo "❌ Android directory not created by dx build"
    exit 1
fi

# ALWAYS apply all patches and inject
echo ""
patch_java21_gradle

echo ""
fix_manifest_extract_native

echo ""
echo "🔔 Setting up notification system..."
setup_notifications

echo ""
inject_icons

echo ""
update_gradle_properties

# Clean gradle cache before rebuild
echo "🧹 Cleaning gradle cache..."
rm -rf -- "$ANDROID_DIR/.gradle" 2>/dev/null || true
pkill -9 gradle java 2>/dev/null || true
sleep 2

# ALWAYS rebuild with Gradle (compiles injected Kotlin + our fixes)
echo ""
echo "📦 Building final APK with Gradle..."
GRADLE_ROOT="$(dirname "$ANDROID_DIR")"
if ! "$GRADLE_ROOT/gradlew" -p "$GRADLE_ROOT" clean assembleRelease -x lintVitalAnalyzeRelease -x lintVitalRelease -x lintVitalReportRelease 2>&1 | tee /tmp/gradle_build.log; then
    echo ""
    echo "❌ Gradle build failed"
    echo "⚠️  Build log saved to /tmp/gradle_build.log"

    if [ -n "$DIOXUS_BACKUP" ] && [ -f "$DIOXUS_BACKUP" ]; then
        echo ""
        echo "🔄 Restoring Dioxus.toml..."
        cp -- "$DIOXUS_BACKUP" "Dioxus.toml"
        rm -f -- "$DIOXUS_BACKUP"
    fi

    exit 1
fi
echo ""
echo "✅ BUILD SUCCESSFUL!"

# ========== POST-BUILD R8 DIAGNOSTICS ==========
analyze_r8_output() {
    echo ""
    echo "🔍 POST-BUILD: Analyzing R8 output..."
    
    local MAPPING_FILE="$ANDROID_DIR/build/outputs/mapping/release/mapping.txt"
    local SEEDS_FILE="$ANDROID_DIR/build/outputs/mapping/release/seeds.txt"
    local USAGE_FILE="$ANDROID_DIR/build/outputs/mapping/release/usage.txt"
    
    if [ -f "$MAPPING_FILE" ]; then
        echo "  📄 R8 mapping.txt found - analyzing..."
        
        # Check if our classes were obfuscated (they shouldn't be with -keep rules)
        local OBFUSCATED=0
        
        if grep -q "se.malmo.skaggbyran.amp.NotificationHelper" "$MAPPING_FILE"; then
            echo "  ⚠️  NotificationHelper appears in mapping.txt (may be obfuscated)"
            OBFUSCATED=$((OBFUSCATED + 1))
        fi
        
        if grep -q "se.malmo.skaggbyran.amp.WebViewConfigurator" "$MAPPING_FILE"; then
            echo "  ⚠️  WebViewConfigurator appears in mapping.txt (may be obfuscated)"
            OBFUSCATED=$((OBFUSCATED + 1))
        fi

        if grep -q "se.malmo.skaggbyran.amp.DormantService" "$MAPPING_FILE"; then
            echo "  ⚠️  DormantService appears in mapping.txt (may be obfuscated)"
            OBFUSCATED=$((OBFUSCATED + 1))
        fi

        if grep -q "se.malmo.skaggbyran.amp.DormantBridge" "$MAPPING_FILE"; then
            echo "  ⚠️  DormantBridge appears in mapping.txt (may be obfuscated)"
            OBFUSCATED=$((OBFUSCATED + 1))
        fi
        
        if grep -q "dev.dioxus.main.MainActivity" "$MAPPING_FILE"; then
            echo "  ⚠️  MainActivity appears in mapping.txt (may be obfuscated)"
            OBFUSCATED=$((OBFUSCATED + 1))
        fi
        
        if [ "$OBFUSCATED" -gt 0 ]; then
            echo "  ⚠️  WARNING: $OBFUSCATED critical classes were obfuscated!"
            echo "     This may cause ClassNotFoundException at runtime"
        else
            echo "  ✅ No critical class obfuscation detected"
        fi
    else
        echo "  ℹ️  mapping.txt not found (R8 may not have run or diagnostics not enabled)"
    fi
    
    if [ -f "$SEEDS_FILE" ]; then
        echo ""
        echo "  📄 R8 seeds.txt found - verifying ProGuard rules..."
        
        local KEPT=0
        
        if grep -q "se.malmo.skaggbyran.amp.NotificationHelper" "$SEEDS_FILE"; then
            echo "  ✅ NotificationHelper kept by ProGuard rules"
            KEPT=$((KEPT + 1))
        else
            echo "  ❌ NotificationHelper NOT in seeds.txt (ProGuard rule failed!)"
        fi
        
        if grep -q "se.malmo.skaggbyran.amp.WebViewConfigurator" "$SEEDS_FILE"; then
            echo "  ✅ WebViewConfigurator kept by ProGuard rules"
            KEPT=$((KEPT + 1))
        else
            echo "  ❌ WebViewConfigurator NOT in seeds.txt (ProGuard rule failed!)"
        fi
        
        if grep -q "dev.dioxus.main.MainActivity" "$SEEDS_FILE"; then
            echo "  ✅ MainActivity kept by ProGuard rules"
            KEPT=$((KEPT + 1))
        else
            echo "  ❌ MainActivity NOT in seeds.txt (ProGuard rule failed!)"
        fi
        
        if [ "$KEPT" -lt 3 ]; then
            echo ""
            echo "  ❌ CRITICAL: Only $KEPT/3 classes kept by ProGuard!"
            echo "     App will crash with ClassNotFoundException"
            return 1
        fi
    else
        echo "  ℹ️  seeds.txt not found"
    fi
    
    if [ -f "$USAGE_FILE" ]; then
        echo ""
        echo "  📄 R8 usage.txt available for manual inspection"
    fi
    
    echo ""
    echo "  ✅ R8 diagnostics complete"
    return 0
}

# Run R8 diagnostics
if [ -d "$ANDROID_DIR" ]; then
    analyze_r8_output || {
        echo ""
        echo "  ⚠️  R8 diagnostics detected issues"
        echo "     Check ProGuard rules in: $ANDROID_DIR/proguard-rules.pro"
    }
fi
# ========== END R8 DIAGNOSTICS ==========

# Show APK location
echo ""
echo "📍 APK location:"
APK_DIR="$ANDROID_DIR/build/outputs/apk/release"

APK_PATH="$(
  find "$APK_DIR" -maxdepth 1 -type f -name '*.apk' -printf '%T@ %p\n' 2>/dev/null \
  | sort -nr \
  | head -n 1 \
  | cut -d' ' -f2-
)"

if [ -n "$APK_PATH" ]; then
    ls -lh -- "$APK_PATH"

    # ========== VERIFY Kotlin CLASSES IN DEX ==========
    echo ""
    echo "🔍 CRITICAL: Verifying Kotlin classes compiled into classes.dex..."
    
    # Check if dexdump is available
    if command -v dexdump &>/dev/null; then
        echo "  Using dexdump to verify class compilation..."
        
        CLASSES_FOUND=0
        
        # Check NotificationHelper
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "NotificationHelper"; then
            echo "  ✅ NotificationHelper found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ NotificationHelper NOT found in classes.dex"
        fi
        
        # Check WebViewConfigurator
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "WebViewConfigurator"; then
            echo "  ✅ WebViewConfigurator found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ WebViewConfigurator NOT found in classes.dex"
            echo "  ⚠️  App will show BLANK SCREEN without DOM storage"
        fi
        
        # Check custom MainActivity
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "dev/dioxus/main/MainActivity"; then
            echo "  ✅ Custom MainActivity found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ Custom MainActivity NOT found in classes.dex"
            echo "  ⚠️  WebView configuration will not run"
        fi

        # Check DormantService
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "DormantService"; then
            echo "  ✅ DormantService found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ DormantService NOT found in classes.dex"
            echo "  ⚠️  Background monitoring will not work"
        fi

        # Check DormantBridge
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "DormantBridge"; then
            echo "  ✅ DormantBridge found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ DormantBridge NOT found in classes.dex"
            echo "  ⚠️  Dormant JNI bridge missing"
        fi

        # Check BootReceiver
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "BootReceiver"; then
            echo "  ✅ BootReceiver found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ BootReceiver NOT found in classes.dex"
            echo "  ⚠️  Auto-start on boot will not work"
        fi

        # Check FilePickerHelper
        if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "FilePickerHelper"; then
            echo "  ✅ FilePickerHelper found in classes.dex"
            CLASSES_FOUND=$((CLASSES_FOUND + 1))
        else
            echo "  ❌ FilePickerHelper NOT found in classes.dex"
            echo "  ⚠️  Import/export will not work"
        fi

        if [ "$CLASSES_FOUND" -eq 7 ]; then
            echo ""
            echo "  ✅ SUCCESS: All 7 Kotlin classes compiled successfully!"

            # Show class details for confirmation
            echo ""
            echo "  📋 Class details:"
            dexdump -l plain "$APK_PATH" 2>/dev/null | grep -E "(NotificationHelper|WebViewConfigurator|dev/dioxus/main/MainActivity|DormantService|DormantBridge|BootReceiver|FilePickerHelper)" | head -n 18
            
            # Verify methods exist
            echo ""
            echo "  🔍 Verifying critical methods..."
            if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "configure.*Landroid/webkit/WebView"; then
                echo "  ✅ WebViewConfigurator.configure(WebView) method present"
            else
                echo "  ⚠️  WebViewConfigurator.configure() method signature unclear"
            fi
            
            if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "onWebViewCreate"; then
                echo "  ✅ MainActivity.onWebViewCreate() method present"
            else
                echo "  ⚠️  MainActivity.onWebViewCreate() method not clearly visible"
            fi
        else
            echo ""
            echo "  ❌ FATAL ERROR: Missing Kotlin classes in classes.dex"
            echo "  ❌ App will crash or show blank screen at runtime"
            echo ""
            echo "  Troubleshooting:"
            echo "  1. Check if src/main/kotlin is registered in build.gradle.kts"
            echo "  2. Verify kotlin-android plugin is applied"
            echo "  3. Check build logs for Kotlin compilation errors"
            echo "  4. Review R8 diagnostics above - classes may have been stripped"
            exit 1
        fi
    else
        echo "  ⚠️  dexdump not available, using fallback verification..."
        
        # Fallback: Check if Kotlin runtime is present (indicates Kotlin was used)
        if unzip -l "$APK_PATH" 2>/dev/null | grep -q "kotlin"; then
            echo "  ✅ Kotlin runtime detected in APK (basic verification)"
            echo "     Install Android SDK build-tools for detailed DEX verification"
        else
            echo "  ⚠️  No Kotlin runtime detected - build may be incomplete"
            echo "     Cannot verify classes without dexdump"
        fi
    fi
    # ========== END DEX VERIFICATION ==========

#    # ========== VERIFY NO INTERNET PERMISSIONS ==========
#    echo ""
#    echo "🔒 Verifying no internet permissions (security requirement)..."
#
#    # Method 1: Check AndroidManifest.xml directly
#    TEMP_MANIFEST="/tmp/amp_manifest_$$.xml"
#    if unzip -p "$APK_PATH" AndroidManifest.xml > "$TEMP_MANIFEST" 2>/dev/null; then
#        # Binary XML, need to decode or check with aapt
#        if command -v aapt &>/dev/null; then
#            if aapt dump permissions "$APK_PATH" 2>/dev/null | grep -q "android.permission.INTERNET"; then
#                echo "  ❌ SECURITY VIOLATION: INTERNET permission found in APK!"
#                echo "  ❌ This app MUST NOT have network access"
#                rm -f "$TEMP_MANIFEST"
#                exit 1
#            else
#                echo "  ✅ No internet permissions (REQUIRED)"
#            fi
#        else
#            # Fallback: just check string presence (less reliable)
#            if strings "$TEMP_MANIFEST" 2>/dev/null | grep -q "android.permission.INTERNET"; then
#                echo "  ⚠️  Possible INTERNET permission detected (unverified)"
#                echo "  Install aapt for reliable verification"
#            else
#                echo "  ✅ No obvious internet permissions (basic check)"
#            fi
#        fi
#        rm -f "$TEMP_MANIFEST"
#    else
#        echo "  ⚠️  Could not extract manifest for verification"
#    fi
#    # ========== END PERMISSION VERIFICATION ==========

    echo ""
    echo "Ready to deploy! 🚀"
else
    echo "  APK not found at expected location"
fi

# Restore Dioxus.toml
echo ""
echo "🔄 Restoring original Dioxus.toml..."
if [ -n "$DIOXUS_BACKUP" ] && [ -f "$DIOXUS_BACKUP" ]; then
    cp -- "$DIOXUS_BACKUP" "Dioxus.toml"
    rm -f -- "$DIOXUS_BACKUP"
    echo "✓ Restored"
else
    echo "⚠️  No backup available"
fi

echo ""
echo "✅ Build complete!"
echo ""
echo "📝 Next steps:"
echo "   1. Uninstall old: adb uninstall se.malmo.skaggbyran.amp"
echo "   2. Install new: adb install \"$APK_PATH\""
echo "   3. Monitor with: adb logcat | grep -E '(amp_MainActivity|amp_WebViewConfig|Notifications|AndroidRuntime)'"
echo ""
echo "🔍 If blank screen persists:"
echo "   - Check logcat for 'amp_MainActivity' logs"
echo "   - Verify onWebViewCreate() was called"
echo "   - Use Chrome DevTools: chrome://inspect"
echo "   - Test localStorage in Console: localStorage.setItem('test', 'works')"
echo ""
echo "🔍 If app crashes, check:"
echo "   - ClassNotFoundException → Review R8 diagnostics above"
echo "   - Check R8 mapping files: $ANDROID_DIR/build/outputs/mapping/release/"
echo "   - JNI errors → Check android_bridge.rs calls correct package"
echo "   - Build errors → Check gradle logs in /tmp/gradle_build.log"

# If platform is "both", also run the iOS build
if [ "$PLATFORM" = "both" ]; then
    build_ios
fi
