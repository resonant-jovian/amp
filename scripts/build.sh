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
        
        # Verify it calls WebViewConfigurator
        if grep -q "WebViewConfigurator.configure" "$KOTLIN_SRC/MainActivity.kt"; then
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
asset_dir = "assets"

[bundle]
publisher = "Skäggbyrån Malmö"
identifier = "se.malmo.skaggbyran.amp"
icon = ["assets/icon/icon-512.png"]
resources = ["assets/data/adress_info.parquet"]

[bundle.android]
publisher = "Skäggbyrån Malmö"
identifier = "se.malmo.skaggbyran.amp"
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
        println("✅ Kotlin sources synced - NotificationHelper, WebViewConfigurator, MainActivity")
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
        
        # ========== CRITICAL: REMOVE INTERNET PERMISSION ==========
        echo ""
        echo "  🔒 SECURITY: Removing INTERNET permission (added by WRY/Dioxus)..."
        if grep -q "android.permission.INTERNET" "$MANIFEST"; then
            sed -i '/android.permission.INTERNET/d' "$MANIFEST"
            echo "  ✓ INTERNET permission removed (security requirement)"
        else
            echo "  ✓ INTERNET permission not present (good)"
        fi
        
        # Also remove network_security_config if present
        if grep -q "networkSecurityConfig" "$MANIFEST"; then
            sed -i 's/android:networkSecurityConfig="@xml\/network_security_config"//g' "$MANIFEST"
            echo "  ✓ networkSecurityConfig reference removed"
        fi
        # ========== END INTERNET REMOVAL ==========
        
        echo ""
        echo "  ✅ Notification system configured"
        echo "  ✅ WebView blank screen fix applied (MainActivity + WebViewConfigurator)"
    else
        echo "  ⚠️  Manifest not found at $MANIFEST"
    fi
}
# ========== END NOTIFICATION SETUP ==========

# [REST OF FILE CONTINUES UNCHANGED...]