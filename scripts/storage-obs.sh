#!/bin/bash
set -e

# ============================================================================
# storage-obs.sh - APK Storage Structure Observer
# ============================================================================
# Investigates internal storage structure of packaged APK and deployed app.
# Examines directory layouts, file locations, asset paths, and verifies that
# resources (parquet files, CSS, fonts) are correctly packaged.
#
# Usage:
#   ./scripts/storage-obs.sh                         # Analyze latest built APK
#   ./scripts/storage-obs.sh --apk-path /path/to.apk # Analyze specific APK
#   ./scripts/storage-obs.sh --output report.txt     # Save report to file
#   ./scripts/storage-obs.sh --device-only            # Device inspection only
#   ./scripts/storage-obs.sh --help                   # Show usage
# ============================================================================

# ===== Configuration =====
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APK_DIR="$REPO_ROOT/target/dx/amp/release/android/app/app/build/outputs/apk/release"
PACKAGE_NAME="se.malmo.skaggbyran.amp"
OUTPUT_FILE=""
DEVICE_ONLY=false
APK_PATH_OVERRIDE=""

# ===== Colors =====
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ===== Logging helpers =====
log_success() { echo -e "  ${GREEN}✅ $1${NC}"; }
log_error()   { echo -e "  ${RED}❌ $1${NC}"; }
log_warn()    { echo -e "  ${YELLOW}⚠️  $1${NC}"; }
log_info()    { echo -e "  ${CYAN}ℹ️  $1${NC}"; }

# ===== Tee output to file if --output is set =====
# We capture all output in a variable and optionally write to file at the end
OUTPUT_BUFFER=""

# Print and optionally buffer output
out() {
    echo -e "$1"
}

section_header() {
    local title="$1"
    local width=60
    local pad=$(( (width - ${#title} - 2) / 2 ))
    local pad_left=$(printf '═%.0s' $(seq 1 $pad))
    local pad_right=$(printf '═%.0s' $(seq 1 $(( width - ${#title} - 2 - pad ))))
    out ""
    out "${BOLD}${BLUE}╔$(printf '═%.0s' $(seq 1 $width))╗${NC}"
    out "${BOLD}${BLUE}║${NC} ${BOLD}${pad_left} ${title} ${pad_right}${NC} ${BOLD}${BLUE}║${NC}"
    out "${BOLD}${BLUE}╚$(printf '═%.0s' $(seq 1 $width))╝${NC}"
    out ""
}

subsection_header() {
    out ""
    out "${BOLD}${CYAN}── $1 ──${NC}"
    out ""
}

human_size() {
    local bytes=$1
    if [ "$bytes" -ge 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f MB\", $bytes/1048576}")" 
    elif [ "$bytes" -ge 1024 ]; then
        echo "$(awk "BEGIN {printf \"%.1f KB\", $bytes/1024}")" 
    else
        echo "${bytes} B"
    fi
}

# ===== Usage / Help =====
show_help() {
    cat << 'EOF'
Usage: ./scripts/storage-obs.sh [OPTIONS]

Investigates internal storage structure of the Amp Android APK and
deployed app on a connected device.

Options:
  --apk-path <path>    Analyze a specific APK file instead of auto-detecting
  --output <file>      Save the report to a file (in addition to stdout)
  --device-only        Only run device storage inspection (skip APK analysis)
  --help               Show this help message

Examples:
  # Analyze latest built APK
  ./scripts/storage-obs.sh

  # Analyze specific APK
  ./scripts/storage-obs.sh --apk-path /path/to/amp.apk

  # Save report to file
  ./scripts/storage-obs.sh --output storage-report.txt

  # Device inspection only (requires connected device via adb)
  ./scripts/storage-obs.sh --device-only

  # Combine options
  ./scripts/storage-obs.sh --apk-path ./my.apk --output report.txt

Prerequisites:
  - Android SDK build-tools (for aapt, optional but recommended)
  - adb (for device inspection)
  - unzip (for APK listing)
EOF
    exit 0
}

# ===== Parse arguments =====
while [[ $# -gt 0 ]]; do
    case $1 in
        --apk-path)
            APK_PATH_OVERRIDE="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --device-only)
            DEVICE_ONLY=true
            shift
            ;;
        --help|-h)
            show_help
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run with --help for usage information."
            exit 1
            ;;
    esac
done

# If --output is set, tee all output to file
if [ -n "$OUTPUT_FILE" ]; then
    # Use process substitution to tee stdout to file
    exec > >(tee -a "$OUTPUT_FILE") 2>&1
    echo "" > "$OUTPUT_FILE"  # Clear file first
fi

# ===== find_latest_apk() =====
find_latest_apk() {
    local search_dir="$1"
    
    if [ -n "$APK_PATH_OVERRIDE" ]; then
        if [ -f "$APK_PATH_OVERRIDE" ]; then
            echo "$APK_PATH_OVERRIDE"
            return 0
        else
            log_error "Specified APK not found: $APK_PATH_OVERRIDE"
            return 1
        fi
    fi
    
    local apk
    apk="$(
        find "$search_dir" -maxdepth 2 -type f -name '*.apk' -printf '%T@ %p\n' 2>/dev/null \
        | sort -nr \
        | head -n 1 \
        | cut -d' ' -f2-
    )"
    
    if [ -n "$apk" ] && [ -f "$apk" ]; then
        echo "$apk"
        return 0
    else
        log_error "No APK found in $search_dir"
        log_info "Build first with: ./scripts/build.sh"
        return 1
    fi
}

# ===== analyze_apk_structure() =====
analyze_apk_structure() {
    local apk="$1"
    
    section_header "APK Structure Analysis"
    
    out "${BOLD}APK:${NC} $apk"
    out "${BOLD}Size:${NC} $(ls -lh "$apk" | awk '{print $5}')"
    out ""
    
    # --- DEX files ---
    subsection_header "DEX Files (Compiled Code)"
    
    local dex_info
    dex_info=$(unzip -l "$apk" '*.dex' 2>/dev/null | grep '\.dex$' || true)
    
    if [ -n "$dex_info" ]; then
        local dex_count=0
        local dex_total_bytes=0
        
        while IFS= read -r line; do
            local size name
            size=$(echo "$line" | awk '{print $1}')
            name=$(echo "$line" | awk '{print $NF}')
            
            if [ -n "$size" ] && [ "$size" -gt 0 ] 2>/dev/null; then
                out "  📦 ${BOLD}$name${NC}  $(human_size "$size")"
                dex_count=$((dex_count + 1))
                dex_total_bytes=$((dex_total_bytes + size))
            fi
        done <<< "$dex_info"
        
        out ""
        out "  DEX count: ${BOLD}$dex_count${NC}"
        out "  DEX total: ${BOLD}$(human_size $dex_total_bytes)${NC}"
    else
        log_warn "No DEX files found in APK"
    fi
    
    # --- Native Libraries ---
    subsection_header "Native Libraries (lib/)"
    
    local lib_info
    lib_info=$(unzip -l "$apk" 'lib/*' 2>/dev/null | grep 'lib/' | grep -v '/$' || true)
    
    if [ -n "$lib_info" ]; then
        local current_arch=""
        while IFS= read -r line; do
            local size name arch
            size=$(echo "$line" | awk '{print $1}')
            name=$(echo "$line" | awk '{print $NF}')
            arch=$(echo "$name" | cut -d'/' -f2)
            
            if [ "$arch" != "$current_arch" ]; then
                out ""
                out "  ${BOLD}Architecture: $arch${NC}"
                current_arch="$arch"
            fi
            
            local basename
            basename=$(basename "$name")
            if [ -n "$size" ] && [ "$size" -gt 0 ] 2>/dev/null; then
                out "    📚 $basename  $(human_size "$size")"
            fi
        done <<< "$lib_info"
    else
        log_warn "No native libraries found in APK"
    fi
    
    # --- Assets ---
    subsection_header "Assets Directory"
    
    local assets_info
    assets_info=$(unzip -l "$apk" 'assets/*' 2>/dev/null | grep 'assets/' | grep -v '  0 ' || true)
    
    if [ -n "$assets_info" ]; then
        while IFS= read -r line; do
            local size name
            size=$(echo "$line" | awk '{print $1}')
            name=$(echo "$line" | awk '{print $NF}')
            
            if [ -n "$size" ] && [ "$size" -gt 0 ] 2>/dev/null; then
                # Color-code by type
                if echo "$name" | grep -qE '\.parquet$'; then
                    out "  ${GREEN}📊 $name${NC}  $(human_size "$size")"
                elif echo "$name" | grep -qE '\.css$'; then
                    out "  ${BLUE}🎨 $name${NC}  $(human_size "$size")"
                elif echo "$name" | grep -qE '\.(ttf|otf|woff|woff2)$'; then
                    out "  ${CYAN}🔤 $name${NC}  $(human_size "$size")"
                elif echo "$name" | grep -qE '\.(png|jpg|jpeg|svg|webp|ico)$'; then
                    out "  🖼️  $name  $(human_size "$size")"
                else
                    out "  📄 $name  $(human_size "$size")"
                fi
            fi
        done <<< "$assets_info"
    else
        log_warn "No assets found in APK"
    fi
    
    # --- Resources ---
    subsection_header "Resources (res/)"
    
    local res_dirs
    res_dirs=$(unzip -l "$apk" 'res/*' 2>/dev/null | grep 'res/' | awk '{print $NF}' | cut -d'/' -f1-2 | sort -u || true)
    
    if [ -n "$res_dirs" ]; then
        while IFS= read -r dir; do
            local count
            count=$(unzip -l "$apk" "${dir}/*" 2>/dev/null | grep "${dir}/" | grep -vc '/$' || echo "0")
            out "  📁 $dir/  ($count files)"
        done <<< "$res_dirs"
    else
        log_info "No res/ directory in APK (or minimal resources)"
    fi
    
    # --- Full file listing summary ---
    subsection_header "Full APK Contents Summary"
    
    local total_files
    total_files=$(unzip -l "$apk" 2>/dev/null | tail -1 | awk '{print $2}')
    local total_size
    total_size=$(unzip -l "$apk" 2>/dev/null | tail -1 | awk '{print $1}')
    
    out "  Total files:           ${BOLD}$total_files${NC}"
    if [ -n "$total_size" ] && [ "$total_size" -gt 0 ] 2>/dev/null; then
        out "  Uncompressed total:    ${BOLD}$(human_size "$total_size")${NC}"
    fi
    out "  Compressed APK size:   ${BOLD}$(ls -lh "$apk" | awk '{print $5}')${NC}"
}

# ===== analyze_apk_with_aapt() =====
analyze_apk_with_aapt() {
    local apk="$1"
    
    section_header "APK Metadata (aapt)"
    
    if ! command -v aapt &>/dev/null; then
        log_warn "aapt not found - install Android SDK build-tools for detailed metadata"
        log_info "Falling back to basic analysis..."
        
        # Fallback: try to extract basic info from unzip listing
        out ""
        out "  ${BOLD}AndroidManifest.xml present:${NC}"
        if unzip -l "$apk" 2>/dev/null | grep -q 'AndroidManifest.xml'; then
            log_success "AndroidManifest.xml found"
        else
            log_error "AndroidManifest.xml NOT found"
        fi
        return 0
    fi
    
    # --- Permissions ---
    subsection_header "Permissions"
    
    local permissions
    permissions=$(aapt dump permissions "$apk" 2>/dev/null || true)
    
    if [ -n "$permissions" ]; then
        while IFS= read -r line; do
            if echo "$line" | grep -q "uses-permission"; then
                local perm
                perm=$(echo "$line" | sed "s/.*name='\([^']*\)'.*/\1/")
                
                # Color-code known permissions
                case "$perm" in
                    *INTERNET*)
                        out "  ${YELLOW}🌐 $perm${NC}"
                        ;;
                    *NOTIFICATION*|*FOREGROUND*)
                        out "  ${GREEN}🔔 $perm${NC}"
                        ;;
                    *LOCATION*)
                        out "  ${BLUE}📍 $perm${NC}"
                        ;;
                    *)
                        out "  🔒 $perm"
                        ;;
                esac
            fi
        done <<< "$permissions"
    else
        log_info "No permissions extracted (or aapt failed)"
    fi
    
    # --- Package info ---
    subsection_header "Package Information"
    
    local badging
    badging=$(aapt dump badging "$apk" 2>/dev/null || true)
    
    if [ -n "$badging" ]; then
        local pkg_name version_name version_code sdk_min sdk_target
        pkg_name=$(echo "$badging" | grep "^package:" | sed "s/.*name='\([^']*\)'.*/\1/" | head -1)
        version_name=$(echo "$badging" | grep "^package:" | sed "s/.*versionName='\([^']*\)'.*/\1/" | head -1)
        version_code=$(echo "$badging" | grep "^package:" | sed "s/.*versionCode='\([^']*\)'.*/\1/" | head -1)
        sdk_min=$(echo "$badging" | grep "sdkVersion" | sed "s/.*'\([^']*\)'.*/\1/" | head -1)
        sdk_target=$(echo "$badging" | grep "targetSdkVersion" | sed "s/.*'\([^']*\)'.*/\1/" | head -1)
        
        out "  ${BOLD}Package:${NC}        $pkg_name"
        out "  ${BOLD}Version Name:${NC}   $version_name"
        out "  ${BOLD}Version Code:${NC}   $version_code"
        out "  ${BOLD}Min SDK:${NC}        $sdk_min"
        out "  ${BOLD}Target SDK:${NC}     $sdk_target"
        
        # Check app label
        local app_label
        app_label=$(echo "$badging" | grep "application-label:" | sed "s/.*'\([^']*\)'.*/\1/" | head -1)
        if [ -n "$app_label" ]; then
            out "  ${BOLD}App Label:${NC}      $app_label"
        fi
    else
        log_warn "Could not extract badging info"
    fi
}

# ===== inspect_device_storage() =====
inspect_device_storage() {
    section_header "Device Storage Inspection"
    
    # Check adb availability
    if ! command -v adb &>/dev/null; then
        log_error "adb not found - install Android SDK platform-tools"
        return 1
    fi
    
    # Check device connection
    local devices
    devices=$(adb devices 2>/dev/null | grep -v 'List' | grep -v '^$' | grep 'device$' || true)
    
    if [ -z "$devices" ]; then
        log_error "No Android device connected"
        log_info "Connect a device with USB debugging enabled and try again"
        log_info "Run 'adb devices' to verify connection"
        return 1
    fi
    
    local device_id
    device_id=$(echo "$devices" | head -1 | awk '{print $1}')
    out "  ${BOLD}Connected device:${NC} $device_id"
    out ""
    
    # Check if app is installed
    if ! adb shell pm list packages 2>/dev/null | grep -q "$PACKAGE_NAME"; then
        log_error "Package $PACKAGE_NAME not installed on device"
        log_info "Install with: adb install <path-to-apk>"
        return 1
    fi
    
    log_success "Package $PACKAGE_NAME is installed"
    out ""
    
    # --- Internal app storage ---
    subsection_header "Internal App Storage (/data/data/$PACKAGE_NAME/)"
    
    # Note: requires root or run-as for debuggable apps
    local internal_tree
    internal_tree=$(adb shell "run-as $PACKAGE_NAME find . -type f -exec ls -la {} \;" 2>/dev/null || true)
    
    if [ -n "$internal_tree" ]; then
        local file_count=0
        while IFS= read -r line; do
            local size path
            size=$(echo "$line" | awk '{print $5}')
            path=$(echo "$line" | awk '{print $NF}')
            
            if [ -n "$path" ]; then
                # Categorize
                if echo "$path" | grep -q 'cache'; then
                    out "  ${YELLOW}🗑️  $path${NC}  ($size bytes)"
                elif echo "$path" | grep -q 'databases'; then
                    out "  ${BLUE}🗃️  $path${NC}  ($size bytes)"
                elif echo "$path" | grep -q 'shared_prefs'; then
                    out "  ${GREEN}⚙️  $path${NC}  ($size bytes)"
                elif echo "$path" | grep -qE '\.(parquet|css|ttf|otf|woff)'; then
                    out "  ${CYAN}📦 $path${NC}  ($size bytes)"
                else
                    out "  📄 $path  ($size bytes)"
                fi
                file_count=$((file_count + 1))
            fi
        done <<< "$internal_tree"
        
        out ""
        out "  Total files: ${BOLD}$file_count${NC}"
    else
        log_warn "Cannot access internal storage (may need root or debuggable build)"
        log_info "Try: adb shell run-as $PACKAGE_NAME ls -la"
        
        # Fallback: try ls via run-as
        out ""
        out "  ${BOLD}Attempting directory listing via run-as:${NC}"
        adb shell "run-as $PACKAGE_NAME ls -la" 2>/dev/null || log_warn "run-as also failed"
    fi
    
    # --- Standard subdirectories ---
    subsection_header "Standard Directories Check"
    
    local dirs_to_check=("cache" "databases" "shared_prefs" "files" "app_webview" "code_cache")
    
    for dir in "${dirs_to_check[@]}"; do
        local exists
        exists=$(adb shell "run-as $PACKAGE_NAME ls -d $dir" 2>/dev/null || echo "")
        
        if [ -n "$exists" ] && ! echo "$exists" | grep -q "No such file"; then
            local count
            count=$(adb shell "run-as $PACKAGE_NAME find $dir -type f 2>/dev/null | wc -l" 2>/dev/null || echo "?")
            log_success "$dir/ exists ($count files)"
        else
            log_info "$dir/ not present"
        fi
    done
    
    # --- External storage ---
    subsection_header "External Storage (/storage/emulated/0/Android/data/$PACKAGE_NAME/)"
    
    local ext_path="/storage/emulated/0/Android/data/$PACKAGE_NAME"
    local ext_tree
    ext_tree=$(adb shell "ls -la $ext_path/ 2>/dev/null" 2>/dev/null || true)
    
    if [ -n "$ext_tree" ] && ! echo "$ext_tree" | grep -q "No such file"; then
        out "$ext_tree"
    else
        log_info "No external storage directory (app may not use external storage)"
    fi
    
    # --- App info from device ---
    subsection_header "Package Info (from device)"
    
    local dumpsys
    dumpsys=$(adb shell dumpsys package "$PACKAGE_NAME" 2>/dev/null | head -40 || true)
    
    if [ -n "$dumpsys" ]; then
        # Extract key info
        local data_dir code_path first_install
        data_dir=$(echo "$dumpsys" | grep 'dataDir=' | awk -F= '{print $2}' | head -1)
        code_path=$(echo "$dumpsys" | grep 'codePath=' | awk -F= '{print $2}' | head -1)
        first_install=$(echo "$dumpsys" | grep 'firstInstallTime=' | awk -F= '{print $2}' | head -1)
        
        [ -n "$data_dir" ] && out "  ${BOLD}Data dir:${NC}         $data_dir"
        [ -n "$code_path" ] && out "  ${BOLD}Code path:${NC}        $code_path"
        [ -n "$first_install" ] && out "  ${BOLD}First installed:${NC}  $first_install"
    fi
}

# ===== verify_assets() =====
verify_assets() {
    local apk="$1"
    
    section_header "Asset Verification"
    
    local issues=0
    
    # Expected assets (from Dioxus.toml bundle resources)
    local expected_assets=(
        "assets/data/adress_info.parquet"
        "assets/style.css"
    )
    
    # Check each expected asset
    for asset in "${expected_assets[@]}"; do
        if unzip -l "$apk" "$asset" 2>/dev/null | grep -q "$asset"; then
            local size
            size=$(unzip -l "$apk" "$asset" 2>/dev/null | grep "$asset" | awk '{print $1}')
            log_success "$asset  ($(human_size "$size"))"
            
            # Compare with source if available
            local source_path="$REPO_ROOT/$asset"
            # Also check without 'assets/' prefix in source
            local alt_source="$REPO_ROOT/$(echo "$asset" | sed 's|^assets/||')"
            
            if [ -f "$source_path" ]; then
                local source_size
                source_size=$(stat -c %s "$source_path" 2>/dev/null || stat -f %z "$source_path" 2>/dev/null || echo "0")
                if [ "$size" = "$source_size" ]; then
                    out "          ${GREEN}Size matches source ($source_path)${NC}"
                else
                    log_warn "Size mismatch: APK=$(human_size "$size") vs source=$(human_size "$source_size")"
                    issues=$((issues + 1))
                fi
            elif [ -f "$alt_source" ]; then
                local source_size
                source_size=$(stat -c %s "$alt_source" 2>/dev/null || stat -f %z "$alt_source" 2>/dev/null || echo "0")
                if [ "$size" = "$source_size" ]; then
                    out "          ${GREEN}Size matches source ($alt_source)${NC}"
                else
                    log_warn "Size mismatch: APK=$(human_size "$size") vs source=$(human_size "$source_size")"
                    issues=$((issues + 1))
                fi
            else
                log_info "Source file not found locally for comparison"
            fi
        else
            log_error "MISSING: $asset"
            issues=$((issues + 1))
        fi
    done
    
    # Check fonts
    subsection_header "Font Files"
    
    local fonts
    fonts=$(unzip -l "$apk" 'assets/fonts/*' 2>/dev/null | grep -E '\.(ttf|otf|woff|woff2)$' || true)
    
    if [ -n "$fonts" ]; then
        while IFS= read -r line; do
            local size name
            size=$(echo "$line" | awk '{print $1}')
            name=$(echo "$line" | awk '{print $NF}')
            
            if [ -n "$size" ] && [ "$size" -gt 0 ] 2>/dev/null; then
                log_success "$name  ($(human_size "$size"))"
            fi
        done <<< "$fonts"
    else
        log_warn "No font files found in assets/fonts/"
        issues=$((issues + 1))
    fi
    
    # --- Summary ---
    out ""
    if [ "$issues" -eq 0 ]; then
        out "  ${GREEN}${BOLD}All asset verifications passed ✅${NC}"
    else
        out "  ${RED}${BOLD}$issues asset issue(s) detected ❌${NC}"
    fi
    
    return $issues
}

# ===== generate_report() =====
generate_report() {
    local apk="$1"
    
    out "${BOLD}╔════════════════════════════════════════════════════════════════╗${NC}"
    out "${BOLD}║       Amp APK Storage Structure Observer                      ║${NC}"
    out "${BOLD}║       $(date '+%Y-%m-%d %H:%M:%S')                                    ║${NC}"
    out "${BOLD}╚════════════════════════════════════════════════════════════════╝${NC}"
    out ""
    out "${BOLD}Project root:${NC}   $REPO_ROOT"
    out "${BOLD}Package:${NC}        $PACKAGE_NAME"
    
    if [ -n "$apk" ]; then
        out "${BOLD}APK:${NC}            $apk"
    fi
    
    if [ -n "$OUTPUT_FILE" ]; then
        out "${BOLD}Output file:${NC}    $OUTPUT_FILE"
    fi
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

if [ "$DEVICE_ONLY" = true ]; then
    generate_report ""
    inspect_device_storage
else
    # Find APK
    APK="$(find_latest_apk "$APK_DIR")" || exit 1
    
    generate_report "$APK"
    
    # APK analysis
    analyze_apk_structure "$APK"
    analyze_apk_with_aapt "$APK"
    verify_assets "$APK" || true
    
    # Device inspection (optional - only if device is connected)
    if command -v adb &>/dev/null; then
        local_devices=$(adb devices 2>/dev/null | grep -v 'List' | grep -v '^$' | grep 'device$' || true)
        if [ -n "$local_devices" ]; then
            inspect_device_storage
        else
            out ""
            log_info "No device connected - skipping device inspection"
            log_info "Connect a device and use --device-only for device analysis"
        fi
    fi
fi

# Final summary
out ""
out "${BOLD}╔════════════════════════════════════════════════════════════════╗${NC}"
out "${BOLD}║       Report Complete                                         ║${NC}"
out "${BOLD}╚════════════════════════════════════════════════════════════════╝${NC}"

if [ -n "$OUTPUT_FILE" ]; then
    out ""
    log_success "Report saved to: $OUTPUT_FILE"
fi

out ""
out "${BOLD}Tips:${NC}"
out "  - Use ${CYAN}--device-only${NC} for device-only inspection"
out "  - Use ${CYAN}--output report.txt${NC} to save this report"
out "  - Use ${CYAN}--apk-path${NC} to analyze a specific APK"
out "  - Install Android SDK build-tools for aapt metadata analysis"
