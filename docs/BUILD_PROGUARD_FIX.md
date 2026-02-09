# ProGuard/R8 Obfuscation Fix

## Problem Diagnosed

The R8 diagnostics revealed that **all 3 critical Kotlin classes were being obfuscated** despite ProGuard keep rules:

```
⚠️  NotificationHelper appears in mapping.txt (may be obfuscated)
⚠️  WebViewConfigurator appears in mapping.txt (may be obfuscated)
⚠️  MainActivity appears in mapping.txt (may be obfuscated)
⚠️  WARNING: 3 critical classes were obfuscated!
```

This causes `ClassNotFoundException` at runtime because:
- **JNI calls from Rust** use hardcoded class names like `se.malmo.skaggbyran.amp.NotificationHelper`
- **R8 renames classes** to short names like `a`, `b`, `c` during obfuscation
- At runtime, Rust tries to find `NotificationHelper` but it's been renamed to `a`

## Root Cause

R8 has **two optimization stages**:

1. **Shrinking** (dead code elimination) - removes unused classes/methods
2. **Obfuscation** (renaming) - renames classes/methods to shorter names

The `-keep` rules prevent **shrinking** but **NOT obfuscation**. You need:
- `-keep` → Prevents shrinking + obfuscation
- `-keepnames` → Only prevents obfuscation (allows shrinking)
- **BUT** in practice, `-keep` alone doesn't always prevent obfuscation

## Solution: Disable Obfuscation Entirely

The nuclear option that **guarantees** no obfuscation:

### Updated ProGuard Rules

The comprehensive ProGuard file at [`android/proguard/proguard-rules.pro`](../android/proguard/proguard-rules.pro) now includes:

```proguard
# ========== CRITICAL: Prevent R8 obfuscation of custom classes ==========
# Disable obfuscation entirely for our packages
-dontobfuscate

# Keep all critical classes
-keep public class se.malmo.skaggbyran.amp.NotificationHelper {
    public <methods>;
    public <fields>;
}

-keep public class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public <methods>;
    public static void configure(android.webkit.WebView);
}

-keep public class dev.dioxus.main.MainActivity {
    public <methods>;
    public void onWebViewCreate(android.webkit.WebView);
}

# Keep package names
-keeppackagenames se.malmo.skaggbyran.amp.**
-keeppackagenames dev.dioxus.main.**

# Enable R8 diagnostics
-printmapping mapping.txt
-printseeds seeds.txt
-printusage usage.txt
-printconfiguration configuration.txt
-verbose
```

### Key Directive: `-dontobfuscate`

This **completely disables obfuscation** while still allowing:
- ✅ Dead code elimination (shrinking)
- ✅ Code optimization
- ✅ Resource shrinking
- ❌ Class/method renaming (obfuscation)

## Manual Fix for Existing Builds

If you need to manually fix the ProGuard configuration:

### 1. Replace ProGuard Rules File

The build script currently creates ProGuard rules inline. You need to **replace** the inline creation with copying from the source file.

**Current code** (in `setup_notifications()` function, around line 280):

```bash
if [ -f "$PROGUARD_RULES" ]; then
    # Inline rule creation with cat >>
else
    cat > "$PROGUARD_RULES" << 'PROGUARD_EOF'
    # ... inline rules ...
PROGUARD_EOF
fi
```

**Replace with**:

```bash
# ========== COPY COMPREHENSIVE PROGUARD RULES ==========
echo ""
echo "  🔒 CRITICAL: Copying comprehensive ProGuard rules..."
echo "     This includes -dontobfuscate to prevent R8 renaming classes"

PROGUARD_SOURCE="$REPO_ROOT/android/proguard/proguard-rules.pro"

if [ -f "$PROGUARD_SOURCE" ]; then
    echo "    📄 Copying proguard-rules.pro from source..."
    cp "$PROGUARD_SOURCE" "$PROGUARD_RULES"
    echo "    ✅ SUCCESS: Comprehensive ProGuard rules installed"
    echo "       - Obfuscation disabled via -dontobfuscate"
    echo "       - All critical classes kept"
    echo "       - R8 diagnostics enabled"
else
    echo "    ❌ CRITICAL: ProGuard source file not found!"
    echo "       Expected at: $PROGUARD_SOURCE"
    exit 1
fi

# Verify the critical directive is present
if grep -q "\-dontobfuscate" "$PROGUARD_RULES"; then
    echo "    ✅ Verified: -dontobfuscate directive present"
else
    echo "    ❌ FATAL: -dontobfuscate NOT found in ProGuard rules!"
    echo "       Classes will be obfuscated causing ClassNotFoundException"
    exit 1
fi
# ========== END PROGUARD COPY ==========
```

### 2. Location in Build Script

Replace the ProGuard section in `scripts/build.sh`:
- **Function**: `setup_notifications()`
- **Approximate line**: 280-370
- **Section header**: `# ========== ENHANCED: ProGuard rules with R8 diagnostics ==========`

### 3. Verify After Build

After running the updated build, check the diagnostics:

```bash
🔍 POST-BUILD: Analyzing R8 output...
  ✅ No critical class obfuscation detected  # ← Should show this
  ✅ NotificationHelper kept by ProGuard rules
  ✅ WebViewConfigurator kept by ProGuard rules
  ✅ MainActivity kept by ProGuard rules
```

## Alternative: Manual ProGuard Rule Addition

If you can't modify the build script, manually add after the first build:

1. **Find the generated ProGuard file**:
   ```bash
   PROGUARD_FILE="$REPO_ROOT/target/dx/amp/release/android/app/app/proguard-rules.pro"
   ```

2. **Add at the top**:
   ```bash
   echo -e "-dontobfuscate\n\n$(cat "$PROGUARD_FILE")" > "$PROGUARD_FILE"
   ```

3. **Rebuild**:
   ```bash
   cd "$REPO_ROOT/target/dx/amp/release/android/app"
   ./gradlew clean assembleRelease
   ```

## Why This Fixes ClassNotFoundException

### Before (With Obfuscation)

**Kotlin source**:
```kotlin
package se.malmo.skaggbyran.amp

class NotificationHelper { ... }
```

**After R8 obfuscation** (in classes.dex):
```
class a { ... }  // NotificationHelper renamed to 'a'
```

**Rust JNI call**:
```rust
env.find_class("se/malmo/skaggbyran/amp/NotificationHelper")
```
**Result**: ❌ `ClassNotFoundException` - class `NotificationHelper` doesn't exist, it's now `a`

### After (With -dontobfuscate)

**After R8 optimization** (in classes.dex):
```
class se.malmo.skaggbyran.amp.NotificationHelper { ... }  // Name preserved!
```

**Rust JNI call**:
```rust
env.find_class("se/malmo/skaggbyran/amp/NotificationHelper")
```
**Result**: ✅ **Success** - class found with original name

## Impact on APK Size

Disabling obfuscation increases APK size slightly:
- **With obfuscation**: ~15-20% smaller (class names like `a`, `b`, `c`)
- **Without obfuscation**: Readable names, ~5-10% larger

For this app (offline-only, no sensitive algorithms), **security through obscurity from obfuscation is NOT needed**.

## Trade-offs

### Pros of -dontobfuscate
- ✅ **Fixes ClassNotFoundException** permanently
- ✅ **Better crash reports** (readable stack traces)
- ✅ **Easier debugging** with logcat
- ✅ **Simpler ProGuard rules** (don't need complex keep patterns)

### Cons
- ❌ Slightly larger APK (~5-10%)
- ❌ Less code obscurity (not a concern for this app)

## References

- [Android R8 Documentation](https://developer.android.com/build/shrink-code)
- [ProGuard Manual: -dontobfuscate](https://www.guardsquare.com/manual/configuration/usage#dontobfuscate)
- [JNI Class Loading Best Practices](https://developer.android.com/training/articles/perf-jni#faq_FindClass)
