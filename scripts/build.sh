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
    
    # ========== NEW: Copy WebViewConfigurator.kt ==========
    if [ -f "$WEBVIEW_SOURCE" ]; then
        echo "  📄 Copying WebViewConfigurator.kt to kotlin/ directory..."
        cp "$WEBVIEW_SOURCE" "$KOTLIN_DIR/WebViewConfigurator.kt"
        echo "  ✓ WebViewConfigurator.kt copied (fixes blank screen)"
    else
        echo "  ⚠️  WebViewConfigurator.kt not found at $WEBVIEW_SOURCE"
        echo "     App may show blank screen without DOM storage enabled"
    fi
    # ========== END WEBVIEW COPY ==========
    
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
    
    # ========== CRITICAL FIX: Register Kotlin source directory ==========
    echo ""
    echo "  🔧 CRITICAL FIX: Registering Kotlin source directory in build.gradle.kts..."
    echo "     This fixes ClassNotFoundException for NotificationHelper + WebViewConfigurator + MainActivity"
    
    if [ -f "$BUILD_GRADLE" ]; then
        # Check if sourceSets already exists
        if grep -q "sourceSets {" "$BUILD_GRADLE"; then
            echo "    ⚠️  sourceSets block already exists"
            echo "    Attempting to append kotlin directory to existing configuration..."
            
            # Try to modify existing java.srcDirs line to include kotlin
            if grep -q 'java\.srcDirs' "$BUILD_GRADLE"; then
                # Backup before modification
                cp "$BUILD_GRADLE" "$BUILD_GRADLE.backup"
                
                # Replace java.srcDirs line to include kotlin directory
                sed -i '/java\.srcDirs/ s/)$/, "src\/main\/kotlin")/' "$BUILD_GRADLE" 2>/dev/null || {
                    echo "    ⚠️  sed replacement failed, trying alternative approach..."
                    mv "$BUILD_GRADLE.backup" "$BUILD_GRADLE"
                    
                    # Alternative: add after the sourceSets line
                    sed -i '/sourceSets {/a\        getByName("main") {\n            java.srcDirs("src/main/java", "src/main/kotlin")\n        }' "$BUILD_GRADLE"
                }
                
                rm -f "$BUILD_GRADLE.backup"
            fi
        else
            echo "    📝 Injecting sourceSets block into android {} block..."
            
            # Insert sourceSets block after 'android {' line
            # Use a more robust sed pattern that works across different android block formats
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
        else
            echo "    ❌ CRITICAL FAILURE: Could not register Kotlin source directory"
            echo "    ❌ Build will fail with ClassNotFoundException at runtime"
            echo ""
            echo "    Please manually add to $BUILD_GRADLE:"
            echo "    android {"
            echo "        sourceSets {"
            echo "            getByName(\"main\") {"
            echo "                java.srcDirs(\"src/main/java\", \"src/main/kotlin\")"
            echo "            }"
            echo "        }"
            echo "    }"
            exit 1
        fi
        
        # Display relevant section for debugging
        echo ""
        echo "    📋 Current source directory configuration:"
        grep -B 2 -A 5 "src/main/kotlin" "$BUILD_GRADLE" | head -n 10 || {
            echo "    (Could not extract sourceSets block for display)"
        }
    else
        echo "    ❌ build.gradle.kts not found at $BUILD_GRADLE"
        echo "    Checked at: $BUILD_GRADLE"
        exit 1
    fi
    # ========== END CRITICAL FIX ==========
    
    # ========== CRITICAL FIX: Prevent R8 from stripping classes ==========
    echo ""
    echo "  🔒 CRITICAL FIX: Adding ProGuard rules to prevent R8 stripping..."
    echo "     R8 was removing NotificationHelper + WebViewConfigurator + MainActivity during minification"
    
    if [ -f "$PROGUARD_RULES" ]; then
        # Check if NotificationHelper rule already exists
        if ! grep -q "NotificationHelper" "$PROGUARD_RULES"; then
            echo "    📝 Injecting keep rule for NotificationHelper..."
            cat >> "$PROGUARD_RULES" << 'PROGUARD_EOF'

# Keep NotificationHelper for JNI access from Rust
-keep class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}
-keepclassmembers class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}
PROGUARD_EOF
            echo "    ✓ NotificationHelper ProGuard rule added"
        else
            echo "    ✓ NotificationHelper ProGuard rule already present"
        fi
        
        # ========== NEW: Add WebViewConfigurator ProGuard rule ==========
        if ! grep -q "WebViewConfigurator" "$PROGUARD_RULES"; then
            echo "    📝 Injecting keep rule for WebViewConfigurator..."
            cat >> "$PROGUARD_RULES" << 'PROGUARD_EOF'

# Keep WebViewConfigurator for MainActivity onCreate call
# Fixes blank screen by enabling DOM storage
-keep class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public *;
}
-keepclassmembers class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public static void configure(android.webkit.WebView);
}
PROGUARD_EOF
            echo "    ✓ WebViewConfigurator ProGuard rule added"
        else
            echo "    ✓ WebViewConfigurator ProGuard rule already present"
        fi
        # ========== END WEBVIEW PROGUARD ==========
        
        # ========== NEW: Add MainActivity ProGuard rule ==========
        if ! grep -q "dev.dioxus.main.MainActivity" "$PROGUARD_RULES"; then
            echo "    📝 Injecting keep rule for custom MainActivity..."
            cat >> "$PROGUARD_RULES" << 'PROGUARD_EOF'

# Keep custom MainActivity that extends WryActivity
# Fixes WebView configuration via onWebViewCreate() hook
-keep class dev.dioxus.main.MainActivity {
    public *;
}
-keepclassmembers class dev.dioxus.main.MainActivity {
    public void onWebViewCreate(android.webkit.WebView);
}
PROGUARD_EOF
            echo "    ✓ MainActivity ProGuard rule added"
        else
            echo "    ✓ MainActivity ProGuard rule already present"
        fi
        # ========== END MAINACTIVITY PROGUARD ==========
    else
        echo "    ⚠️  proguard-rules.pro not found, creating..."
        cat > "$PROGUARD_RULES" << 'PROGUARD_EOF'
# Keep NotificationHelper for JNI access from Rust
-keep class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}
-keepclassmembers class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}

# Keep WebViewConfigurator for MainActivity onCreate call
# Fixes blank screen by enabling DOM storage
-keep class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public *;
}
-keepclassmembers class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public static void configure(android.webkit.WebView);
}

# Keep custom MainActivity that extends WryActivity
# Fixes WebView configuration via onWebViewCreate() hook
-keep class dev.dioxus.main.MainActivity {
    public *;
}
-keepclassmembers class dev.dioxus.main.MainActivity {
    public void onWebViewCreate(android.webkit.WebView);
}
PROGUARD_EOF
        echo "    ✓ Created proguard-rules.pro with all keep rules"
    fi
    # ========== END R8 FIX ==========
    
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

# Build with Dioxus (generates fresh gradle files)
echo "📦 Building APK with Dioxus..."
if ! dx build --android --release --device HQ646M01AF --verbose; then
    echo ""
    echo "⚠️  First build failed, applying fixes and retrying..."
    echo ""

    # FIX: Update generated gradle files for Java 21
    echo "🔧 Fixing generated gradle files for Java 21..."

    if [ -d "$ANDROID_DIR" ]; then
        # Fix root build.gradle.kts (parent level)
        ROOT_BUILD_GRADLE="$(dirname "$ANDROID_DIR")/build.gradle.kts"
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

        # Fix Android manifest extractNativeLibs issue
        echo ""
        echo "🔧 Fixing Android manifest issues..."
        MANIFEST_FILE="$ANDROID_DIR/src/main/AndroidManifest.xml"
        if [ -f "$MANIFEST_FILE" ]; then
            if grep -q 'android:extractNativeLibs="false"' "$MANIFEST_FILE"; then
                echo "  Removing deprecated extractNativeLibs attribute..."
                sed -i 's/ android:extractNativeLibs="false"//g' "$MANIFEST_FILE"
                echo "✓ Fixed manifest extractNativeLibs"
            fi
            if grep -q 'android:extractNativeLibs="true"' "$MANIFEST_FILE"; then
                echo "  Removing extractNativeLibs=true attribute..."
                sed -i 's/ android:extractNativeLibs="true"//g' "$MANIFEST_FILE"
                echo "✓ Removed extractNativeLibs attribute"
            fi
        fi

        # ========== INJECT NOTIFICATION SETUP HERE ==========
        setup_notifications
        # ========== END NOTIFICATION SETUP ==========

        # ========== INJECT CUSTOM APP ICONS (AGGRESSIVE OVERRIDE) ==========
        echo ""
        echo "🎨 Injecting custom app icons..."

        RES_DIR="$ANDROID_DIR/src/main/res"
        ICON_SOURCE="$REPO_ROOT/android/assets/icon"

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
        if [ -f "$ICON_SOURCE/icon-mdpi.png" ]; then
            cp "$ICON_SOURCE/icon-mdpi.png" "$RES_DIR/mipmap-mdpi/ic_launcher.png"
            echo "  ✓ Copied mdpi icon (48x48)"
        fi

        if [ -f "$ICON_SOURCE/icon-hdpi.png" ]; then
            cp "$ICON_SOURCE/icon-hdpi.png" "$RES_DIR/mipmap-hdpi/ic_launcher.png"
            echo "  ✓ Copied hdpi icon (72x72)"
        fi

        if [ -f "$ICON_SOURCE/icon-xhdpi.png" ]; then
            cp "$ICON_SOURCE/icon-xhdpi.png" "$RES_DIR/mipmap-xhdpi/ic_launcher.png"
            echo "  ✓ Copied xhdpi icon (96x96)"
        fi

        if [ -f "$ICON_SOURCE/icon-xxhdpi.png" ]; then
            cp "$ICON_SOURCE/icon-xxhdpi.png" "$RES_DIR/mipmap-xxhdpi/ic_launcher.png"
            echo "  ✓ Copied xxhdpi icon (144x144)"
        fi

        if [ -f "$ICON_SOURCE/icon-xxxhdpi.png" ]; then
            cp "$ICON_SOURCE/icon-xxxhdpi.png" "$RES_DIR/mipmap-xxxhdpi/ic_launcher.png"
            echo "  ✓ Copied xxxhdpi icon (192x192)"
        fi

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
        # ========== END ICON INJECTION ==========

        # Create/update gradle.properties
        echo "🔧 Updating gradle.properties..."
        GRADLE_PROPS="$ANDROID_DIR/gradle.properties"
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

        # Clean gradle cache
        echo "🧹 Cleaning gradle cache..."
        rm -rf -- "$ANDROID_DIR/.gradle" 2>/dev/null || true
        pkill -9 gradle java 2>/dev/null || true
        sleep 2

        # Rebuild with gradle
        echo ""
        echo "📦 Rebuilding with fixed configuration..."
        # Navigate to the PARENT directory (where Gradle multi-build expects to be)
        GRADLE_ROOT="$(dirname "$ANDROID_DIR")"
        if ! "$GRADLE_ROOT/gradlew" -p "$GRADLE_ROOT" clean assembleRelease -x lintVitalAnalyzeRelease -x lintVitalRelease -x lintVitalReportRelease 2>&1 | tee /tmp/gradle_build.log; then
            echo ""
            echo "❌ Gradle build failed after fixes"
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
    else
        echo "❌ Android directory not created: $ANDROID_DIR"
        exit 1
    fi
else
    echo ""
    echo "✓ Dioxus build completed successfully on first try!"
    
    # Even on success, run notification setup if it hasn't been done
    if [ -d "$ANDROID_DIR" ]; then
        setup_notifications
    fi
fi

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
        
        if [ "$CLASSES_FOUND" -eq 3 ]; then
            echo ""
            echo "  ✅ SUCCESS: All Kotlin classes compiled successfully!"
            
            # Show class details for confirmation
            echo ""
            echo "  📋 Class details:"
            dexdump -l plain "$APK_PATH" 2>/dev/null | grep -E "(NotificationHelper|WebViewConfigurator|MainActivity)" | head -n 6
        else
            echo ""
            echo "  ❌ FATAL ERROR: Missing Kotlin classes in classes.dex"
            echo "  ❌ App will crash or show blank screen at runtime"
            echo ""
            echo "  Troubleshooting:"
            echo "  1. Check if src/main/kotlin is registered in build.gradle.kts"
            echo "  2. Verify kotlin-android plugin is applied"
            echo "  3. Check build logs for Kotlin compilation errors"
            echo "  4. Check ProGuard rules - R8 may be stripping the classes"
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

    # ========== VERIFY NO INTERNET PERMISSIONS ==========
    echo ""
    echo "🔒 Verifying no internet permissions (security requirement)..."
    
    # Method 1: Check AndroidManifest.xml directly
    TEMP_MANIFEST="/tmp/amp_manifest_$$.xml"
    if unzip -p "$APK_PATH" AndroidManifest.xml > "$TEMP_MANIFEST" 2>/dev/null; then
        # Binary XML, need to decode or check with aapt
        if command -v aapt &>/dev/null; then
            if aapt dump permissions "$APK_PATH" 2>/dev/null | grep -q "android.permission.INTERNET"; then
                echo "  ❌ SECURITY VIOLATION: INTERNET permission found in APK!"
                echo "  ❌ This app MUST NOT have network access"
                rm -f "$TEMP_MANIFEST"
                exit 1
            else
                echo "  ✅ No internet permissions (REQUIRED)"
            fi
        else
            # Fallback: just check string presence (less reliable)
            if strings "$TEMP_MANIFEST" 2>/dev/null | grep -q "android.permission.INTERNET"; then
                echo "  ⚠️  Possible INTERNET permission detected (unverified)"
                echo "  Install aapt for reliable verification"
            else
                echo "  ✅ No obvious internet permissions (basic check)"
            fi
        fi
        rm -f "$TEMP_MANIFEST"
    else
        echo "  ⚠️  Could not extract manifest for verification"
    fi
    # ========== END PERMISSION VERIFICATION ==========

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
echo "   3. Monitor with: adb logcat | grep -E '(amp_MainActivity|amp_WebViewConfig|Notifications)'"
echo ""
echo "🔍 If blank screen persists:"
echo "   - Check logcat for 'amp_MainActivity' logs"
echo "   - Verify onWebViewCreate() was called"
echo "   - Use Chrome DevTools: chrome://inspect"
echo "   - Test localStorage in Console: localStorage.setItem('test', 'works')"
echo ""
echo "🔍 If app crashes, check:"
echo "   - ClassNotFoundException → Classes not in DEX (run dexdump verification)"
echo "   - JNI errors → Check android_bridge.rs calls correct package"
echo "   - Build errors → Check gradle logs in /tmp/gradle_build.log"
