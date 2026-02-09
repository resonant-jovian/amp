# Kotlin Source Directory Registration in Android Gradle Builds

## Overview

This document explains how to properly register additional Kotlin source directories in Android Gradle builds, specifically for the Amp parking app's NotificationHelper integration.

## Problem Statement

When copying Kotlin source files into `src/main/kotlin/` directory after a Gradle project is generated, the Gradle build system doesn't automatically discover or compile them unless:

1. **The source directory is explicitly registered** in the build configuration
2. **Gradle runs from the correct project root**
3. **The Kotlin compilation task is triggered**

Without these steps, Kotlin files become "dead code" — physically present but never compiled into the APK's `classes.dex`.

## Solution: sourceSets Configuration

### What sourceSets Does

The `sourceSets` configuration in `build.gradle.kts` tells Gradle which directories contain source code:

```kotlin
android {
    sourceSets {
        getByName("main") {
            // Tell Gradle to compile BOTH java and kotlin directories
            java.srcDirs("src/main/java", "src/main/kotlin")
        }
    }
}
```

This registers both:
- ✅ `src/main/java/` - Java sources (default)
- ✅ `src/main/kotlin/` - Kotlin sources (additional)

### Critical Requirements

#### 1. Location: Inside `android {}` Block

```kotlin
// ❌ WRONG - Outside android block
sourceSets {
    getByName("main") {
        java.srcDirs("src/main/java", "src/main/kotlin")
    }
}

android {
    // Other config...
}

// ✅ CORRECT - Inside android block
android {
    sourceSets {
        getByName("main") {
            java.srcDirs("src/main/java", "src/main/kotlin")
        }
    }
    // Other config...
}
```

#### 2. Plugin Requirement: kotlin-android

The `kotlin-android` plugin must be applied:

```kotlin
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")  // ← REQUIRED for Kotlin compilation
}
```

Without this plugin, Kotlin files won't be compiled even if registered.

#### 3. Gradle Execution Directory

Gradle must run from the **project root**, not a subdirectory:

```bash
# ❌ WRONG - Running from app/ module directory
cd app/
./gradlew assembleRelease

# ✅ CORRECT - Running from project root
cd app/  # (where settings.gradle.kts is located)
./gradlew assembleRelease
```

For Dioxus Android builds with nested structure:
```bash
# Dioxus creates: target/dx/amp/release/android/app/app/
# The Gradle project root is: target/dx/amp/release/android/app/
# So run:
target/dx/amp/release/android/app/gradlew assembleRelease
```

## Implementation in Amp

### File Injection Process

The `scripts/build.sh` injects sourceSets configuration in the `setup_notifications()` function:

```bash
setup_notifications() {
    # 1. Copy Kotlin source to correct package directory
    KOTLIN_DIR="$ANDROID_DIR/src/main/kotlin/se/malmo/skaggbyran/amp"
    cp "$KOTLIN_SOURCE" "$KOTLIN_DIR/NotificationHelper.kt"
    
    # 2. Read build.gradle.kts
    BUILD_GRADLE="$ANDROID_DIR/build.gradle.kts"
    
    # 3. Inject sourceSets block if not present
    if ! grep -q "sourceSets {" "$BUILD_GRADLE"; then
        sed -i '/^android {$/a\    sourceSets {\n        getByName("main") {\n            java.srcDirs("src/main/java", "src/main/kotlin")\n        }\n    }' "$BUILD_GRADLE"
    fi
    
    # 4. Verify injection was successful
    if grep -q "src/main/kotlin" "$BUILD_GRADLE"; then
        echo "✅ sourceSets registered successfully"
    else
        echo "❌ Failed to register sourceSets"
        exit 1
    fi
}
```

### Verification

The build script verifies Kotlin compilation succeeded:

```bash
# Use dexdump to check if class was compiled into DEX
dexdump -l plain "$APK_PATH" | grep "NotificationHelper"

# Expected output:
# Class descriptor  : 'Lse/malmo/skaggbyran/amp/NotificationHelper;'
# Access flags      : 0x4011 (PUBLIC FINAL)
# SourceFile        : "NotificationHelper.kt"
```

## Troubleshooting

### Issue: sourceSets Injection Fails

**Symptom**: sed command reports no matches
**Cause**: `android {` is on same line as other code
**Solution**: Adjust sed pattern to match actual format

```bash
# Try different patterns:
sed -i '/android\s*{/a\    sourceSets { ... }' build.gradle.kts
sed -i '/android {$/a\    sourceSets { ... }' build.gradle.kts
```

### Issue: Kotlin File Not Compiled

**Symptom**: dexdump shows class NOT in DEX
**Causes**:
1. sourceSets not registered → Run `grep "src/main/kotlin" build.gradle.kts`
2. Gradle ran from wrong directory → Check gradle execution path
3. kotlin-android plugin missing → Check `build.gradle.kts` plugins block

**Verification**:
```bash
# 1. Check sourceSets registered
grep -A 5 "sourceSets" build.gradle.kts

# 2. Check plugin applied
grep "kotlin.android" build.gradle.kts

# 3. Check Kotlin files exist
find src/main/kotlin -name "*.kt"

# 4. Run Gradle from correct directory
pwd  # Must be the Gradle project root
./gradlew assembleRelease
```

### Issue: ClassNotFoundException at Runtime

**Symptom**: `java.lang.ClassNotFoundException: se.malmo.skaggbyran.amp.NotificationHelper`
**Root Cause**: Class not compiled into classes.dex
**Solution**:
1. Verify sourceSets configuration in build.gradle.kts
2. Run `dexdump` to check if class exists
3. Re-run build script to apply fixes
4. Clear Gradle cache: `rm -rf build/ .gradle/`

## Gradle Task Order

When compiling Kotlin and Java together:

```
compileReleaseJavaWithJavac   # Compile Java files
    ↓
compileReleaseKotlin          # Compile Kotlin files (NEW when sourceSets registered)
    ↓
mergeReleaseJavaResource      # Merge resources
    ↓
minifyReleaseWithR8           # Optimize with R8
    ↓
assembleRelease               # Package APK
```

If you see `compileReleaseKotlin` task in the build output, Kotlin compilation is enabled.

## References

- [Android Gradle Plugin Documentation](https://developer.android.com/build)
- [Gradle sourcesets documentation](https://docs.gradle.org/current/userguide/building_java_projects.html#sec:java_source_sets)
- [Kotlin Android Plugin](https://kotlinlang.org/docs/gradle.html#targeting-android)
- [Dioxus Android Build](https://dioxuslabs.com/learn/0.7/guide/03-advanced/build-a-mobile-app)

## Key Takeaways

✅ **Always register additional source directories** in sourceSets  
✅ **sourceSets goes INSIDE the android {} block**  
✅ **kotlin-android plugin MUST be applied**  
✅ **Run Gradle from the project root, not subdirectories**  
✅ **Verify compilation with dexdump** to confirm classes in DEX  
✅ **Use relative paths** (relative to build.gradle.kts location)  
