# AMP Testing Interface - Current Status ✅

## Commit: 93bbd0d0
**Date:** 2026-01-26 16:25 CET

## What's Working ✅

1. **Auto-load on page load** ✅
   - Correlation address pre-filled in search box
   - Map auto-loads with one search (no more double searches)
   - Container dimensions: 482×491px ✅

2. **API Integration** ✅
   - Malmö geo API returns coordinates correctly
   - WKT parsing works (both `POINT(X Y)` and `POINT (X Y)` formats)
   - Address search succeeds

3. **StadsAtlas Map Loading** ✅
   - Iframe loads successfully
   - Map controls visible (zoom, layers, etc.)
   - Blue pin marker displays at coordinates
   - Layers panel loads
   - SVG icons load

4. **UI Layout** ✅
   - Map section: 550px fixed height
   - Tab container: Proper flex layout
   - Data tab active by default
   - Instructions and Debug tabs available

## Known Issue ⚠️

### Background map and miljödata layer not rendering

**What we see:**
- Iframe loads (HTTP 304)
- CSS loads (HTTP 200)
- JS loads (HTTP 200)
- Map controls visible
- Blue pin marker visible
- **BUT:** Background map tiles and miljödata layer not visible

**Logs show:**
```
GET https://stadsatlas.malmo.se/stadsatlas/#center=...
GET https://stadsatlas.malmo.se/stadsatlas/css/style.css [HTTP/2 200]
GET https://stadsatlas.malmo.se/stadsatlas/js/origo.js [HTTP/2 200]
GET https://stadsatlas.malmo.se/wms/fgk.qgs?...LAYER=miljoparkering_l [HTTP/2 200]
```

All requests succeed, but content isn't rendering in iframe.

## Root Cause Analysis

This is likely a **StadsAtlas/Origo rendering issue in iframes**:

1. **file:// context:** Test runs from `file:///tmp/amp_test_0.html`
   - CORS doesn't apply, but iframe rendering may be constrained
   - Some browsers restrict iframe rendering from file:// URLs

2. **Iframe context:** Origo/OpenLayers in nested iframe
   - May need explicit initialization
   - Canvas rendering may not work properly
   - SVG rendering may be blocked

3. **Data loading:** API calls succeed but visual rendering fails
   - Tiles are requested but not rendered
   - WMS layer is requested but not rendered
   - UI elements render but map content doesn't

## Recommendation

**Option 1: Serve via HTTP (RECOMMENDED)**
- Instead of `file://`, run a local HTTP server
- Fixes: iframe rendering, CORS issues, browser security restrictions
- Quick fix: `python3 -m http.server` or `npx serve`

**Option 2: Direct embedding**
- Instead of iframe, embed StadsAtlas directly in the page
- More complex but would avoid iframe limitations
- Would require re-architecting the interface

**Option 3: Wait for StadsAtlas update**
- May be a known issue with Origo/OpenLayers versions
- Could be fixed in a newer release

## Test Command

```bash
rm -f /tmp/amp_test_*.html
cargo run --release -- test -a kdtree -c 20 -w 1
```

## Files Modified

- `server/src/assets/stadsatlas_interface.html` - CSS dimensions and layout
- `server/src/assets/stadsatlas_interface.js` - Auto-load, parsing, double-search fix
- `server/src/assets/stadsatlas_interface.css` - External styling

## All 16 Commits

1. `acee09c` - Initial CSS fix
2. `97e9f18` - Add inline CSS
3. `9fee398` - Documentation
4. `dc6f22c` - Aggressive !important CSS
5. `825bd2b` - CSS consistency
6. `f2bfa71` - Update docs
7. `dc0a07f` - Debug logging
8. `248f48a` - Comprehensive docs
9. `317b8e5` - URLSearchParams fix
10. `187263d` - Width constraints
11. `b41f89b` - API debugging
12. `cab5984` - Remove hardcoded defaults
13. `c5e594a` - Update instructions
14. `878efb2` - Restore search input
15. `9168ea2` - Auto-loading support
16. `75523f4` - WKT regex fix
17. `e2200720` - Aggressive width fixes
18. `93bbd0d` - Double search fix ← Current

## Next Steps

1. **Test with HTTP server** to confirm map rendering works outside file:// context
2. If successful, consider running test mode with `--serve` flag for HTTP
3. If that works, full testing can proceed with proper map visualization

---

**Status:** Interface is functionally complete, but map visualization needs HTTP context or iframe fix.
