# ========== CRITICAL: Prevent R8 obfuscation of custom classes ==========
# R8 has two stages: shrinking (removal) and obfuscation (renaming)
# -keep prevents BOTH shrinking and obfuscation
# -keepnames only prevents obfuscation but allows shrinking
# We need -keep to ensure classes stay AND keep their names

# ========== PRIMARY DEX: Force classes into main DEX file ==========
# This prevents ClassNotFoundException from DEX splitting
# Classes MUST be in same DEX as MainActivity for ClassLoader to find them
-keep,includecode class se.malmo.skaggbyran.amp.** { *; }
-keep,includecode class dev.dioxus.main.** { *; }

# Keep NotificationHelper - accessed via JNI from Rust
-keep public class se.malmo.skaggbyran.amp.NotificationHelper {
    public <methods>;
    public <fields>;
}

# Prevent obfuscation of NotificationHelper class name
-keepnames class se.malmo.skaggbyran.amp.NotificationHelper

# Keep WebViewConfigurator - called from MainActivity
-keep public class se.malmo.skaggbyran.amp.WebViewConfigurator {
    public <methods>;
    public static void configure(android.webkit.WebView);
}

# Prevent obfuscation of WebViewConfigurator class name
-keepnames class se.malmo.skaggbyran.amp.WebViewConfigurator

# Keep custom MainActivity - extends WryActivity
-keep public class dev.dioxus.main.MainActivity {
    public <methods>;
    public void onWebViewCreate(android.webkit.WebView);
}

# Prevent obfuscation of MainActivity class name
-keepnames class dev.dioxus.main.MainActivity

# ========== Keep package names (prevents ClassNotFoundException) ==========
# This ensures package names stay as-is during obfuscation
-keeppackagenames se.malmo.skaggbyran.amp.**
-keeppackagenames dev.dioxus.main.**

# ========== Additional safety: Keep all public APIs in our packages ==========
-keep public class se.malmo.skaggbyran.amp.** {
    public protected *;
}

-keep public class dev.dioxus.main.** {
    public protected *;
}

# ========== Prevent obfuscation of reflection/JNI accessed members ==========
# Keep native methods (for Rust JNI calls)
-keepclasseswithmembernames,includedescriptorclasses class * {
    native <methods>;
}

# ========== WRY/Dioxus framework compatibility ==========
# Keep WryActivity and related classes
-keep class dev.dioxus.main.WryActivity {
    *;
}

-keep class dev.dioxus.main.** extends dev.dioxus.main.WryActivity {
    *;
}

# Keep WebView-related interfaces
-keep class * implements android.webkit.WebViewClient {
    *;
}

-keep class * implements android.webkit.WebChromeClient {
    *;
}

# ========== Disable obfuscation entirely for our packages ==========
# This is the nuclear option - prevents ALL obfuscation of our code
# while still allowing dead code elimination
-dontobfuscate

# Alternatively, if you want shrinking but no obfuscation:
# -optimizations !code/simplification/arithmetic,!code/simplification/cast,!field/*,!class/merging/*
# -optimizationpasses 5
# -allowaccessmodification
# -dontpreverify

# ========== MULTIDEX: Disable secondary DEX files ==========
# Force everything into primary classes.dex to avoid ClassLoader issues
# This ensures MainActivity and WebViewConfigurator are in same DEX
-dontshrink
-dontoptimize

# If multidex is enabled, keep our classes in primary DEX
-keep class se.malmo.skaggbyran.amp.** {
    <init>();
}
-keep class dev.dioxus.main.** {
    <init>();
}

# ========== R8 diagnostics ==========
-printmapping mapping.txt
-printseeds seeds.txt
-printusage usage.txt
-printconfiguration configuration.txt
-verbose

# ========== Android SDK compatibility ==========
# Keep Android SDK classes that might be accessed via reflection
-keep class android.webkit.** { *; }
-keep class android.app.** { *; }

# ========== Warnings ==========
# Don't warn about missing classes (common in Android library dependencies)
-dontwarn javax.**
-dontwarn java.lang.instrument.**
-dontwarn sun.misc.**

# ========== General Android rules ==========
-keepattributes Signature
-keepattributes *Annotation*
-keepattributes SourceFile,LineNumberTable
-keepattributes InnerClasses
-keepattributes EnclosingMethod

# Keep Parcelable implementations
-keep class * implements android.os.Parcelable {
    public static final android.os.Parcelable$Creator *;
}

# Keep Serializable implementations
-keepclassmembers class * implements java.io.Serializable {
    static final long serialVersionUID;
    private static final java.io.ObjectStreamField[] serialPersistentFields;
    private void writeObject(java.io.ObjectOutputStream);
    private void readObject(java.io.ObjectInputStream);
    java.lang.Object writeReplace();
    java.lang.Object readResolve();
}
