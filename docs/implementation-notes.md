# Implementation Notes

This document tracks technical implementation details for testing features and StadsAtlas integration.

## Testing Mode Implementation (feature/correlation-testing)

### New `test` Subcommand

Added comprehensive testing via browser automation:

```bash
amp-server test --algorithm kdtree --cutoff 50 --windows 10
```

**Parameters:**
- `--algorithm` (default: kdtree) - Correlation algorithm
- `--cutoff` (default: 50m) - Distance threshold
- `--windows` (default: 10) - Browser windows to open

**How it Works:**
1. Loads addresses and zone data from ArcGIS API
2. Runs correlation with specified parameters
3. Filters to matching addresses only
4. Randomly samples from matches (or all if count ≤ requested)
5. Opens N browser windows with 2 tabs each:
   - **Tab 1:** [StadsAtlas](https://stadsatlas.malmo.se/stadsatlas/) with embedded map
   - **Tab 2:** Correlation result details (HTML data URL)

### Distance Cutoff Configuration

Implemented cutoff parameter across all relevant commands:

```rust
fn correlate_dataset(
    algorithm: &AlgorithmChoice,
    addresses: &[AdressClean],
    zones: &[MiljoeDataClean],
    cutoff: f64,  // NEW
    pb: &ProgressBar,
) -> Result<Vec<(String, f64, String)>, Box<dyn std::error::Error>>
```

**Applied to:**
- `correlate` command
- `test` command
- `benchmark` command
- All 6 algorithm implementations

**Implementation:**
```rust
if dist > cutoff {
    return None;  // Excluded from results
}
```

### Default Algorithm: KD-Tree

Changed from R-Tree to KD-Tree for 2D spatial queries:
- Better performance on Malmö coordinate ranges
- Excellent reliability in benchmarks
- Intuitive for distance-based searches

### Browser Automation Functions

#### Cross-Platform Window Launch

**Windows:**
```rust
std::process::Command::new("cmd")
    .args(&["/C", &format!("start {} && start {}", url1, url2)])
```

**macOS:**
```rust
std::process::Command::new("open")
    .args(&["-n", url])
```

**Linux:**
```rust
std::process::Command::new("xdg-open")
    .arg(url)
```

#### HTML Data URL Generation

Result details encoded as data URL for Tab 2:
```
data:text/html;charset=utf-8,[urlencoded HTML]
```

**Benefits:**
- No temporary files needed
- Self-contained tab data
- Automatic cleanup on browser close

**Implementation** uses [urlencoding](https://crates.io/crates/urlencoding) crate.

#### Window Opening Timing

500ms delay between windows prevents:
- System resource exhaustion
- Browser launch race conditions
- Network congestion
- User interface overwhelm

### Sample Size Validation

Fixed benchmark to handle edge cases:

```rust
let actual_sample_size = sample_size.min(addresses.len());
let requested_msg = if sample_size > addresses.len() {
    format!(" (requested {} but only {} available)", sample_size, addresses.len())
} else {
    String::new()
};
```

**Features:**
- Progress bars show actual work, not inflated counts
- User feedback on size mismatches
- No false overflow on calculations

---

## StadsAtlas Integration

### Injection Script v2 - Phase-Based Approach

Completely redesigned injection strategy based on accessibility tree analysis.

**Previous Approach (v1 - Failed):**
- Auto-reload page
- Search for input field
- Type address

**Why it failed:**
- Search input doesn't exist on initial load
- Page requires user interaction before searching
- Cross-origin iframe restrictions
- No understanding of menu-driven architecture

**New Approach (v2 - Integrated):**

Phase-based execution with fallbacks:

```
Phase 1: Click Menu Button (by known ID)
         └─ Opens layer control panel
         
Phase 2: Find Miljöparkering Layer
         └─ Search DOM for text containing "miljö" AND "parkering"
         
Phase 3: Toggle Layer Visibility  
         └─ Click visibility toggle button near layer name
         
Phase 4: Find Search Input
         └─ Try 8 different CSS selectors (fallback chain)
         
Phase 5: Inject Address
         └─ Set value + fire keyboard events
```

### Known Button IDs

These may change if StadsAtlas updates:

```javascript
const BUTTON_IDS = {
  MENU: '#cmkv2kcmk000g206jepb76w85',      // Opens layer panel
  ZOOM_IN: '#cmkv2kcmj0004206jnnm2ygxd',   // Zoom in
  ZOOM_OUT: '#cmkv2kcmj0006206js5fq58t2',  // Zoom out
  HOME: '#cmkv2kcmk000d206je8eyd0w0',      // Reset view
  GEO: '#cmkv2kcmk001f206jzya5lsbh'        // Current location
};
```

**Maintenance:** If IDs change, update in `server/src/main.rs` (around line 850).

### Search Input Fallback Selectors

Tried in order (first match wins):

1. `input[placeholder*="Sök"]` - Swedish placeholder
2. `input[placeholder*="Search"]` - English placeholder
3. `input[type="search"]` - Type attribute
4. `input[class*="search"]` - Search in class name
5. `input[aria-label*="Sök"]` - ARIA label in Swedish
6. `.ol-search input` - OpenLayers search class
7. `[class*="search"] input` - Search class on parent
8. `input[name*="search"]` - Search in name attribute

**Maintenance:** Add selectors to array in `server/src/main.rs` (around line 920) if structure changes.

### Execution Timeline

Typical execution for one window:

```
Phase 1: 100-500ms  (menu click + retry)
Phase 2: 50-200ms   (text search through DOM)
Phase 3: 300-500ms  (button click + DOM update)
Phase 4: 100-300ms  (selector attempts)
Phase 5: 10-50ms    (injection + events)
         ──────────
Total:   600-1500ms (typical)
Max:     15 seconds (timeout limit)
```

### Debug Interface

Access via browser console:

```javascript
window.ampInjection.phase()    // Get current phase (1-5)
window.ampInjection.debug()    // Show page state
window.ampInjection.retry()    // Restart injection
```

All logs prefixed with `[AMP]` for easy filtering.

---

## StadsAtlas Map Container Rendering

### Problem

Map was not rendering in testing interface - showing blank white container with error:

```
No map visible because the map container's width or height are 0.
```

This prevented:
- Basemap (background) from loading
- Miljöparkering layer from displaying

### Root Causes

**Layer 1 - CSS/HTML:**
- `.map-container` div had fixed height without parent constraints
- No flex layout for proper dimension cascading
- External CSS files not loading in file:// test environment

**Layer 2 - Build/Execution:**
- Server generates HTML on-the-fly
- Asset files must reload for each test
- Old versions cached in `/tmp/` directory

### Solutions

#### 1. CSS Fix

Updated flex layout in `server/src/assets/stadsatlas_interface.css`:

```css
.map-section {
    height: 550px;
    display: flex;
    flex-direction: column;
}

.map-container {
    height: 100%;
    flex: 1;
    min-height: 0;  /* Critical for flex children */
}
```

**Key:** `min-height: 0` allows flex sizing to override default auto heights.

#### 2. Inline Critical Styles

Added inline `<style>` block with `!important` flags for robustness:

```css
<style>
    html, body {
        height: 100% !important;
        width: 100% !important;
    }
    .map-section {
        height: 550px !important;
        min-height: 550px !important;
        display: flex !important;
        flex-direction: column !important;
    }
    .map-container {
        height: 100% !important;
        flex: 1 !important;
        min-height: 0 !important;
    }
</style>
```

**Why `!important`:**
- Overrides conflicting CSS in all contexts
- Works in file:// URLs, iframes, and production
- Ensures styles apply even with external CSS conflicts

#### 3. JavaScript Enhancement

Added dimension logging and emergency fallback in `server/src/assets/stadsatlas_interface.js`:

```javascript
// Log computed dimensions for debugging
const rect = mapContainer.getBoundingClientRect();
console.log(`[AMP] Container dimensions: ${rect.width}×${rect.height}px`);

// Emergency fallback
if (rect.height === 0) {
    mapContainer.style.height = '550px';
}
```

#### 4. Default Tab Change

Data tab (Tab 2) now active by default:
- Shows correlation results immediately
- Users can switch to Instructions or Debug as needed

### Verification

✅ **Background Map:**
```
GET https://gis.malmo.se/arcgis/rest/services/baskartor/Bakgrundskarta_nedtonad_3008_text/MapServer/tile/9/20979/875
[HTTP/2 200]
```

✅ **Miljöparkering Layer:**
```
GET https://stadsatlas.malmo.se/wms/fgk.qgs?REQUEST=GetMap...LAYERS=miljoparkering_l
[HTTP/2 200]
```

✅ **Container Dimensions:**
- 550px height maintained
- Map controls visible
- Pin marker displays
- No "zero-height" errors

### Lessons Learned

**For Web Mapping:**
1. Always ensure parent container has explicit dimensions
2. Use `min-height: 0` on flex children for proper sizing
3. Test in all contexts (file://, http://, iframes)

**For Dynamic HTML Generation:**
1. Inline critical styles - external CSS may not load
2. Use `!important` for non-negotiable dimensions
3. Separate concerns: template HTML, external CSS, interactive JS

---

## Dependencies Added

**File:** `server/Cargo.toml`

```toml
urlencoding = "2.1"
```

**Purpose:** Encodes HTML content as data URLs for test result display tab.

---

## Files Modified

**Core Implementation:**
- `server/src/main.rs` - Added test command, distance cutoff, browser automation
- `server/Cargo.toml` - Added urlencoding dependency

**Assets:**
- `server/src/assets/stadsatlas_interface.html` - CSS dimensions and layout
- `server/src/assets/stadsatlas_interface.css` - External styling
- `server/src/assets/stadsatlas_interface.js` - Auto-load, parsing, dimension fixes

---

## Testing Recommendations

### Compare Algorithms

```bash
# Test different algorithms on same addresses
amp-server test --algorithm kdtree --cutoff 50 --windows 5
amp-server test --algorithm rtree --cutoff 50 --windows 5
# Manually compare results in StadsAtlas
```

### Validate Distance Thresholds

```bash
# Conservative (25m) - fewer but more accurate
amp-server test --cutoff 25 --windows 3

# Standard (50m)
amp-server test --cutoff 50 --windows 3

# Permissive (100m) - more matches but may include false positives
amp-server test --cutoff 100 --windows 3
```

### Test Edge Cases

```bash
# Request more windows than available matches
amp-server test --algorithm kdtree --cutoff 10 --windows 100
# Should open only available matches
```

### Benchmark with Realistic Data

```bash
cargo run -- benchmark --sample-size 1000 --cutoff 50
# Verify all algorithms respect cutoff
```

---

## Troubleshooting

### Map Not Rendering

**Cause:** File:// context limitations with iframe rendering

**Solution:** Delete temp files and rebuild:

```bash
rm -f /tmp/amp_test_*.html
cargo run --release -- test -a kdtree -c 20 -w 1
```

### StadsAtlas Elements Not Found

**Cause:** StadsAtlas updated their UI

**Solution:** Update button IDs or selectors in `server/src/main.rs`:

1. Open browser DevTools
2. Inspect the element in question
3. Find new ID/selector
4. Update in code
5. Rebuild: `cargo build`

### Windows Not Opening

**Windows:** Ensure default browser is configured  
**macOS:** Grant terminal permission to control applications  
**Linux:** Verify `xdg-open` is installed

---

## Related Documentation

- [Testing Guide](testing.md) - Testing procedures
- [CLI Usage](cli-usage.md) - Command reference
- [Architecture](architecture.md) - System design
