# AMP Rust Android MVP - Build Instructions

## File Structure (Copy Exactly)

```
amp/
├── Cargo.toml                          (from Cargo.toml file)
├── rust-toolchain.toml                 (from rust-toolchain.toml file)
├── core/
│   ├── Cargo.toml                      (from core-Cargo.toml file)
│   └── src/
│       ├── lib.rs                      (from core-src-lib.rs file)
│       ├── error.rs                    (from core-src-error.rs file)
│       ├── models.rs                   (from core-src-models.rs file)
│       ├── correlation.rs              (from core-src-correlation.rs file)
│       ├── geolocation.rs              (from core-src-geolocation.rs file)
│       └── state.rs                    (from core-src-state.rs file)
└── android/
    ├── Cargo.toml                      (from android-Cargo.toml file)
    └── src/
        └── lib.rs                      (from android-src-lib.rs file)
```

## Step 1: Add Android Targets

```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
```

## Step 2: Build for Android ARM64

```bash
cargo build --release \
    -p amp-android \
    --target aarch64-linux-android
```

Output: `target/aarch64-linux-android/release/libamp_android.so`

## Step 3: Build for Android ARM32

```bash
cargo build --release \
    -p amp-android \
    --target armv7-linux-androideabi
```

Output: `target/armv7-linux-androideabi/release/libamp_android.so`

## Step 4: Verify Build Success

```bash
ls -lh target/aarch64-linux-android/release/libamp_android.so
ls -lh target/armv7-linux-androideabi/release/libamp_android.so
```

Expected: ~200-500 KB each (stripped release binary)

## Step 5: Use with Flutter

Copy the `.so` files to your Flutter Android project:

```bash
mkdir -p android/app/src/main/jniLibs/arm64-v8a
mkdir -p android/app/src/main/jniLibs/armeabi-v7a

cp target/aarch64-linux-android/release/libamp_android.so \
   android/app/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libamp_android.so \
   android/app/src/main/jniLibs/armeabi-v7a/
```

## Flutter Integration (Dart FFI)

```dart
import 'dart:ffi';

final DynamicLibrary nativeLib = Platform.isAndroid
    ? DynamicLibrary.open('libamp_android.so')
    : throw UnsupportedError('Unsupported platform');

typedef InitAppFunc = int Function();
typedef AddAddressFunc = int Function(String address);
typedef GetCountFunc = int Function();
typedef ClearAllFunc = int Function();

final initApp = nativeLib
    .lookup<NativeFunction<Int32 Function()>>('java_com_amp_main_activity_init_app')
    .asFunction<int Function()>();

final addAddress = nativeLib
    .lookup<NativeFunction<Int32 Function(Pointer<Utf8>)>>('java_com_amp_main_activity_add_address')
    .asFunction<int Function(Pointer<Utf8>)>();

// Usage
void main() {
  int count = initApp();
  print('Initialized with $count addresses');
}
```

## Troubleshooting

### Build fails: "Cannot find target"
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
```

### Build fails: "cc: not found"
Install NDK and set:
```bash
export NDK_HOME=/path/to/android-ndk-r25c
```

### Binary too large
Already handled with `strip = true` in release profile.

### Linking fails
Ensure you're using nightly 2026-01-12:
```bash
rustc --version
# Should show: rustc 1.XX.0-nightly (date)
```

## What's Included

✅ **Core Library** - High-precision GPS, cleaning schedules, analysis  
✅ **Android FFI** - JNI bindings for Flutter  
✅ **Release Optimized** - LTO, stripped, minimal size  
✅ **Multi-target** - ARM64 and ARM32 support  
✅ **Zero allocations** - Where possible  

## API Reference (Exported to Android)

All functions prefixed with `Java_com_amp_MainActivity_`:

- `initApp()` → int (0 = success)
- `addAddress(String)` → int (0 = success, -1 = error)
- `getAddressCount()` → int (number of stored addresses)
- `clearAll()` → int (0 = success)

## What's TODO (Server)

- ✗ Axum HTTP server
- ✗ Python API bindings
- ✗ Background scheduler
- ✗ Data persistence

(These are left as TODO for now - Android app is ready to compile!)
