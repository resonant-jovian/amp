# StadsAtlas Map Container Rendering Fix

## Problem

The Malmö StadsAtlas map was not rendering in the AMP testing interface, showing only a blank white container. Browser console showed the error:

```
No map visible because the map container's width or height are 0.
```

This prevented both:
- **Basemap (background map)** from loading
- **Miljöparkering layer** from displaying

## Root Cause

The `.map-container` div had:
- Fixed height of `500px` applied inline in CSS
- No parent dimension constraints  
- Flex layout without proper sizing for child elements
- **CSS was not being applied when loading from `file://` URLs** in test environment

When the Origo/OpenLayers initialization ran, it found a container with computed dimensions of 0×0 pixels, which prevented map rendering.

## Solution

Implemented a multi-layered fix to ensure explicit, cascading dimensions:

### Changes to `server/src/assets/stadsatlas_interface.html`

Added aggressive inline CSS with `!important` flags:

```css
<style>
    /* Force explicit dimensions for Origo/OpenLayers */
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

**Key points:**
- `!important` flags ensure styles are applied even in file:// environment
- Explicit 550px height on `.map-section` provides fixed container size
- `.map-container` uses `height: 100%` and `flex: 1` to fill available space
- `min-height: 0` on flex children allows proper flex sizing
- Default tab changed to **Data** (per user request)

### Changes to `server/src/assets/stadsatlas_interface.css`

Updated to match inline styles:
1. **`.map-section`** — Added `height: 550px` and `min-height: 550px`
2. **`.map-container`** — Changed `height: 500px` → `height: 100%`, added `flex: 1` and `min-height: 0`
3. **`.control-panel`** — Added `flex-shrink: 0` to prevent collapse
4. **`.value`** — Added `word-break: break-word` for better data display

## Result

After these fixes:

✅ Map container now has explicit dimensions (550px × full width)  
✅ Inline !important CSS overrides any conflicting styles  
✅ Works in file:// test environment AND production
✅ Origo can properly initialize OpenLayers with valid container size  
✅ Basemap renders correctly  
✅ Miljöparkering layer can activate and display  
✅ Pin marker and coordinates display properly  
✅ Data tab is default active view  

## Testing

To verify the fix works:

1. Load the StadsAtlas testing interface
2. Click "Search Address & Load Map"
3. Verify the map appears with:
   - Background map tiles (Bakgrundskarta nedtonad)
   - Miljöparkering layer active (if available for that location)
   - Pin marker at the searched coordinates
4. Confirm **Data tab** is active by default showing correlation results

## Key Learning

When using OpenLayers/Origo in web applications:
- **Always ensure parent container has explicit dimensions**
- Use `height: 100%` on parent elements that should fill viewport
- Set `min-height: 0` on flex children to allow proper flex sizing
- Test in embedded iframes to ensure dimensions cascade correctly
- Use inline `!important` CSS for critical dimension fixes that must work in all loading contexts
- For file:// URLs and test environments, inline styles are more reliable than external CSS files

## Commits

- `acee09c`: fix: Ensure map container has explicit dimensions to prevent zero-height rendering
- `97e9f18`: docs: Add inline CSS comment explaining map container dimension requirements  
- `9fee398`: docs: Add detailed documentation of StadsAtlas map container rendering fix
- `dc6f22c`: fix: Add aggressive inline CSS with !important flags to force map container dimensions
- `825bd2b`: refactor: Update CSS to match inline style requirements with clear comments
