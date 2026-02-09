# Android Build Critical Fixes

## Overview

This document explains two critical issues discovered during Android builds and their fixes:

1. **INTERNET Permission** - Added by default by WRY/Dioxus (security issue)
2. **R8 Minification** - Strips NotificationHelper even with correct sourceSets

---

## Issue 1: INTERNET Permission (Security Violation)

### The Problem

**WRY/Dioxus automatically adds INTERNET permission** to AndroidManifest.xml by default, even though Amp explicitly doesn't request it.

**Why this is critical**:
- Amp is a **security-sensitive offline-first app**
- NO network access is required or desired
- INTERNET permission enables potential data exfiltration
- Privacy policy specifies NO data leaves the device

### Root Cause

The WRY webview library (used by Dioxus) adds these by default:
```xml
<uses-permission android:name="android.permission.INTERNET" />
<application
    android:networkSecurityConfig="@xml/network_security_config">
```

This happens **after** Dioxus generates the Android project, so the permission appears even though it's not in your `Dioxus.toml`.

### The Fix

In `scripts/build.sh`, the `setup_notifications()` function now explicitly removes INTERNET permission:

```bash
# Remove INTERNET permission
if grep -q "android.permission.INTERNET" "$MANIFEST"; then
    sed -i '/android.permission.INTERNET/d' "$MANIFEST"
    echo "  ✓ INTERNET permission removed (security requirement)"
fi

# Remove networkSecurityConfig reference
if grep -q "networkSecurityConfig" "$MANIFEST"; then
    sed -i 's/android:networkSecurityConfig="@xml\/network_security_config"//g' "$MANIFEST"
    echo "  ✓ networkSecurityConfig reference removed"
fi
```

### Verification

After build completes, verify INTERNET permission is removed:

```bash
# Method 1: Using aapt (most reliable)
aapt dump permissions path/to/app-release.apk | grep INTERNET
# Expected: No output (permission not present)

# Method 2: Check manifest directly
unzip -p path/to/app-release.apk AndroidManifest.xml | strings | grep INTERNET
# Expected: No output

# Method 3: Build script automatic verification
# Look for this in build output:
#   ✅ No internet permissions (REQUIRED)
```

**CRITICAL**: If INTERNET permission is detected, the build script will **EXIT** with error code 1.

### When to Update

Check this fix if:
- Dioxus/WRY updates change manifest generation
- You see unexpected network activity in `adb logcat`
- Security audit flags network permissions

---

## Issue 2: R8 Minification Strips NotificationHelper

### The Problem

**Even with correct `sourceSets` configuration**, R8 (Android's code optimizer) was **removing NotificationHelper** during the `minifyReleaseWithR8` task.

**Build output showed**:
```
> Task :app:compileReleaseKotlin    ← Kotlin compiled successfully!
> Task :app:minifyReleaseWithR8     ← R8 strips NotificationHelper here
```

**Result**: `dexdump` showed NotificationHelper NOT in classes.dex, causing ClassNotFoundException at runtime.

### Root Cause

R8's aggressive optimization considers classes "unused" if:
1. No explicit Java/Kotlin code references them
2. No ProGuard keep rules protect them
3. They're only accessed via JNI/reflection

**NotificationHelper is accessed from Rust via JNI**, so R8 thinks it's unused and removes it.

### The Fix

In `scripts/build.sh`, the `setup_notifications()` function now injects ProGuard keep rules:

```bash
# Prevent R8 from stripping NotificationHelper
if [ -f "$PROGUARD_RULES" ]; then
    if ! grep -q "NotificationHelper" "$PROGUARD_RULES"; then
        cat >> "$PROGUARD_RULES" << 'PROGUARD_EOF'

# Keep NotificationHelper for JNI access from Rust
-keep class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}
-keepclassmembers class se.malmo.skaggbyran.amp.NotificationHelper {
    public *;
}
PROGUARD_EOF
        echo "    ✓ ProGuard rule added"
    fi
fi
```

### What These Rules Do

**`-keep class se.malmo.skaggbyran.amp.NotificationHelper { public *; }`**
- Prevents R8 from removing the class entirely
- Keeps all public methods
- Preserves method names (required for JNI)

**`-keepclassmembers class se.malmo.skaggbyran.amp.NotificationHelper { public *; }`**
- Ensures public methods aren't obfuscated
- Prevents R8 from changing method signatures
- Keeps JNI method names intact

### Verification

After build completes, verify NotificationHelper is in DEX:

```bash
# Method 1: Using dexdump (most reliable)
dexdump -l plain path/to/app-release.apk | grep NotificationHelper

# Expected output:
# Class descriptor  : 'Lse/malmo/skaggbyran/amp/NotificationHelper;'
# Access flags      : 0x4011 (PUBLIC FINAL)
# SourceFile        : "NotificationHelper.kt"
# Direct methods -
#   #0              : (in Lse/malmo/skaggbyran/amp/NotificationHelper;)
#     name          : '<init>'
#     public static showNotification(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)V

# Method 2: Build script automatic verification
# Look for this in build output:
#   ✅ SUCCESS: NotificationHelper found in classes.dex
#      The Kotlin source was successfully compiled!
```

**CRITICAL**: If NotificationHelper is NOT found, the build script will **EXIT** with error code 1.

### Why sourceSets Alone Wasn't Enough

Many developers assume registering Kotlin sources is sufficient:
```kotlin
android {
    sourceSets {
        getByName("main") {
            java.srcDirs("src/main/java", "src/main/kotlin")
        }
    }
}
```

**This is necessary but not sufficient** for JNI-accessed classes because:

1. ✅ **sourceSets** → Tells Gradle to **compile** Kotlin sources
2. ✅ **Compilation** → Creates `.class` files in `build/intermediates/`
3. ❌ **R8 Minification** → **Removes** "unused" classes from final DEX
4. ✅ **ProGuard rules** → **Prevents** R8 from removing kept classes

**Complete solution** = sourceSets + ProGuard keep rules

---

## Complete Verification Checklist

After building, verify both fixes:

```bash
APK_PATH="target/dx/amp/release/android/app/app/build/outputs/apk/release/app-release.apk"

# 1. Check NotificationHelper in DEX
echo "1. Verifying NotificationHelper compiled..."
if dexdump -l plain "$APK_PATH" 2>/dev/null | grep -q "NotificationHelper"; then
    echo "   ✅ NotificationHelper found in DEX"
else
    echo "   ❌ NotificationHelper NOT in DEX - BUILD FAILED"
    exit 1
fi

# 2. Check NO INTERNET permission
echo "2. Verifying no INTERNET permission..."
if aapt dump permissions "$APK_PATH" 2>/dev/null | grep -q "INTERNET"; then
    echo "   ❌ INTERNET permission detected - SECURITY VIOLATION"
    exit 1
else
    echo "   ✅ No INTERNET permission (required)"
fi

# 3. Check sourceSets configuration
echo "3. Verifying sourceSets..."
BUILD_GRADLE="target/dx/amp/release/android/app/app/build.gradle.kts"
if grep -q "src/main/kotlin" "$BUILD_GRADLE"; then
    echo "   ✅ Kotlin source directory registered"
else
    echo "   ❌ Kotlin source directory NOT registered"
    exit 1
fi

# 4. Check ProGuard rules
echo "4. Verifying ProGuard keep rules..."
PROGUARD_RULES="target/dx/amp/release/android/app/app/proguard-rules.pro"
if grep -q "NotificationHelper" "$PROGUARD_RULES"; then
    echo "   ✅ ProGuard keep rule present"
else
    echo "   ❌ ProGuard keep rule MISSING"
    exit 1
fi

echo ""
echo "✅ All verification checks passed!"
```

---

## Troubleshooting

### App crashes with ClassNotFoundException

**Error**:
```
java.lang.ClassNotFoundException: se.malmo.skaggbyran.amp.NotificationHelper
```

**Diagnosis**:
```bash
# Check if class is in DEX
dexdump -l plain app-release.apk | grep NotificationHelper
# If NO output → class was stripped by R8
```

**Solutions**:
1. Verify ProGuard rules are in `proguard-rules.pro`
2. Check build logs for R8 warnings about missing classes
3. Verify `build.gradle.kts` references `proguard-rules.pro`:
   ```kotlin
   buildTypes {
       release {
           proguardFiles(
               getDefaultProguardFile("proguard-android-optimize.txt"),
               "proguard-rules.pro"
           )
       }
   }
   ```

### App has network access (security violation)

**Diagnosis**:
```bash
# Check permissions
aapt dump permissions app-release.apk | grep INTERNET
# If output → permission present
```

**Solutions**:
1. Verify `setup_notifications()` ran during build
2. Check for INTERNET permission in generated manifest:
   ```bash
   cat target/dx/amp/release/android/app/app/src/main/AndroidManifest.xml | grep INTERNET
   ```
3. If permission still present, rebuild with clean:
   ```bash
   rm -rf target/dx/
   cd android && ../scripts/build.sh
   ```

### compileReleaseKotlin task not running

**Symptom**: Build logs don't show `> Task :app:compileReleaseKotlin`

**Diagnosis**:
```bash
# Check sourceSets configuration
grep -A 5 "sourceSets" target/dx/amp/release/android/app/app/build.gradle.kts
```

**Solution**: See `docs/kotlin-source-registration.md` for detailed sourceSets troubleshooting.

---

## Key Takeaways

✅ **INTERNET permission must be explicitly removed** - Dioxus/WRY adds it by default  
✅ **ProGuard keep rules are REQUIRED** - sourceSets alone doesn't prevent R8 stripping  
✅ **Verification is mandatory** - Both checks must pass before deployment  
✅ **JNI classes need special handling** - Reflection/JNI access is invisible to R8  
✅ **Security requirements drive architecture** - Offline-first means NO network access  

---

## References

- [Android ProGuard Documentation](https://developer.android.com/studio/build/shrink-code)
- [R8 Optimization](https://developer.android.com/studio/build/shrink-code#optimization)
- [JNI and ProGuard](https://developer.android.com/studio/build/shrink-code#native-crash-support)
- [Android Permissions Overview](https://developer.android.com/guide/topics/permissions/overview)
- `docs/kotlin-source-registration.md` - Kotlin compilation setup
- `docs/build-output-analysis.md` - Build path structure analysis
