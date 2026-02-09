# Android Build: ClassNotFoundException Fix

## Problem Summary

```
java.lang.ClassNotFoundException: se.malmo.skaggbyran.amp.WebViewConfigurator
```

The app crashes immediately on launch because **Kotlin source files are not being compiled into the APK**.

## Root Cause Analysis

### Timeline of Events

1. **`dx build`** generates Android project structure
2. **Gradle scans** `src/main/kotlin` for source files (finds NONE)
3. **Build script copies** Kotlin files (TOO LATE - Gradle already scanned)
4. **Gradle compiles** empty set of Kotlin files
5. **R8 processes** APK (no Kotlin classes exist to keep)
6. **APK deployed** without WebViewConfigurator/NotificationHelper
7. **App launches** and crashes with ClassNotFoundException

### Why `-dontobfuscate` Doesn't Help

ProGuard rules prevent:
- ✅ **Obfuscation** (renaming classes)
- ✅ **Dead code elimination** (removing unused classes)

But they **CANNOT** create classes that were never compiled in the first place.

### Evidence from Build Output

```bash
# Build script shows Kotlin files being copied:
✓ WebViewConfigurator.kt copied (fixes blank screen)
✓ Custom MainActivity.kt installed
✓ sourceSets block injected

# But logcat shows they don't exist at runtime:
java.lang.ClassNotFoundException: se.malmo.skaggbyran.amp.WebViewConfigurator
```

## Solutions

### Option 1: Gradle Build Hook (RECOMMENDED)

Create a Gradle task that runs **before** `preBuild` to copy Kotlin files:

```kotlin
// In build.gradle.kts
tasks.register<Copy>("syncKotlinSources") {
    description = "Copy custom Kotlin files before build"
    
    from("../../../../../android/kotlin") {
        include("**/*.kt")
    }
    
    into("src/main/kotlin")
    
    doLast {
        println("✅ Synced Kotlin sources")
    }
}

// Make preBuild depend on our sync task
tasks.named("preBuild") {
    dependsOn("syncKotlinSources")
}
```

**Pros:**
- Kotlin files copied BEFORE Gradle source scan
- Works with Gradle incremental builds
- Automatic on every build

**Cons:**
- Requires modifying auto-generated build.gradle.kts
- Must be re-applied after each `dx build`

### Option 2: Pre-Build Directory Structure

Create Kotlin source directory structure BEFORE running `dx build`:

```bash
# In build script, BEFORE dx build:
PREBUILD_KOTLIN="$REPO_ROOT/target/dx/amp/release/android/app/app/src/main/kotlin"
mkdir -p "$PREBUILD_KOTLIN/se/malmo/skaggbyran/amp"
mkdir -p "$PREBUILD_KOTLIN/dev/dioxus/main"

# Copy files immediately
cp "$REPO_ROOT/android/kotlin/NotificationHelper.kt" \
   "$PREBUILD_KOTLIN/se/malmo/skaggbyran/amp/"
cp "$REPO_ROOT/android/kotlin/WebViewConfigurator.kt" \
   "$PREBUILD_KOTLIN/se/malmo/skaggbyran/amp/"
cp "$REPO_ROOT/android/kotlin/MainActivity.kt" \
   "$PREBUILD_KOTLIN/dev/dioxus/main/"

# THEN run dx build
dx build --android --release
```

**Pros:**
- Simple bash script modification
- Files exist when Gradle scans sources

**Cons:**
- `dx build` might clean the directory first
- Race condition with Dioxus generation

### Option 3: Disable R8 Minification (WORKAROUND)

If other solutions fail, disable minification entirely:

```kotlin
// In build.gradle.kts
buildTypes {
    getByName("release") {
        isMinifyEnabled = false  // Disable R8/ProGuard
        isShrinkResources = false
        signingConfig = signingConfigs.getByName("release")
    }
}
```

**Pros:**
- Guarantees no class removal
- Easier debugging

**Cons:**
- Larger APK size (~2-5MB increase)
- Slightly slower app startup
- Exposed class names (minor security concern)

### Option 4: Gradle `buildSrc` Plugin (ADVANCED)

Create a custom Gradle plugin that copies files as part of source set configuration:

```kotlin
// buildSrc/src/main/kotlin/KotlinSyncPlugin.kt
import org.gradle.api.Plugin
import org.gradle.api.Project

class KotlinSyncPlugin : Plugin<Project> {
    override fun apply(project: Project) {
        project.afterEvaluate {
            // Copy Kotlin sources after evaluation but before compilation
            android.sourceSets.getByName("main") {
                val kotlinSrc = project.file("../../../../../android/kotlin")
                if (kotlinSrc.exists()) {
                    java.srcDirs(kotlinSrc)
                }
            }
        }
    }
}
```

**Pros:**
- Most robust solution
- Integrates deeply with Gradle
- Survives `dx build` regeneration

**Cons:**
- Complex setup
- Requires maintaining `buildSrc` directory

## Recommended Implementation

**Use Option 1 (Gradle Build Hook)** with build script automation:

1. Update `build.sh` to inject the `syncKotlinSources` task into generated `build.gradle.kts`
2. Make task run before `preBuild`
3. Keep existing ProGuard rules for safety

```bash
# In build.sh, after Gradle file fixes:
if [ -f "$ANDROID_DIR/build.gradle.kts" ]; then
    # Inject sync task at end of file
    cat >> "$ANDROID_DIR/build.gradle.kts" << 'GRADLE_TASK'

// CRITICAL: Sync Kotlin sources before build
tasks.register<Copy>("syncKotlinSources") {
    from("../../../../../android/kotlin") {
        include("**/*.kt")
    }
    into("src/main/kotlin")
    doLast {
        println("✅ Kotlin sources synced")
    }
}

tasks.named("preBuild") {
    dependsOn("syncKotlinSources")
}
GRADLE_TASK
    echo "✅ Injected syncKotlinSources task"
fi
```

## Verification Steps

After implementing the fix:

### 1. Check Build Logs

Look for:
```
✅ Kotlin sources synced
> Task :app:compileReleaseKotlin
```

### 2. Verify DEX Contains Classes

```bash
dexdump -l plain app-release.apk | grep -E '(WebViewConfigurator|NotificationHelper|MainActivity)'
```

Should show:
```
Class descriptor : 'Lse/malmo/skaggbyran/amp/WebViewConfigurator;'
Class descriptor : 'Lse/malmo/skaggbyran/amp/NotificationHelper;'
Class descriptor : 'Ldev/dioxus/main/MainActivity;'
```

### 3. Test App Launch

```bash
adb install -r app-release.apk
adb shell am start -n se.malmo.skaggbyran.amp/dev.dioxus.main.MainActivity
adb logcat | grep -E '(ClassNotFoundException|WebViewConfigurator)'
```

Should show:
```
amp_MainActivity: onCreate called
amp_WebViewConfig: Configuring WebView
```

NOT:
```
ClassNotFoundException: se.malmo.skaggbyran.amp.WebViewConfigurator
```

## Alternative: Two-Stage Build

If Gradle hooks prove unreliable:

```bash
# Stage 1: Generate structure
dx build --android --release

# Stage 2: Inject Kotlin files
cp android/kotlin/*.kt target/dx/amp/release/android/app/app/src/main/kotlin/

# Stage 3: Rebuild with Gradle directly
cd target/dx/amp/release/android/app
./gradlew clean assembleRelease
```

This guarantees files exist before Gradle compilation.

## Related Issues

- [Android Build Script](../scripts/build.sh)
- [ProGuard Rules](../android/proguard/proguard-rules.pro)
- [Custom MainActivity](../android/kotlin/MainActivity.kt)
- [WebViewConfigurator](../android/kotlin/WebViewConfigurator.kt)

## References

- [Gradle Task Dependencies](https://docs.gradle.org/current/userguide/task_dependencies.html)
- [Android SourceSets](https://developer.android.com/studio/build/build-variants#sourcesets)
- [ProGuard -keep Options](https://www.guardsquare.com/manual/configuration/usage#keepoptions)
