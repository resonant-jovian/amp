# AMP Testing Mode - Quick Start Guide

## Overview

The new `test` subcommand allows you to visually verify correlation algorithm accuracy by opening browser windows to StadsAtlas alongside correlation result data.

## Quick Start

### 1. Basic Testing (Default Settings)

```bash
cd server
cargo run -- test
```

**What happens:**
- Opens 10 browser windows (default)
- Uses KD-Tree algorithm (default)
- 50 meter distance threshold (default)
- Randomly samples matching addresses

### 2. Custom Algorithm

```bash
cargo run -- test --algorithm raycasting
```

**Available algorithms:**
- `distance-based` - Simple distance calculation
- `raycasting` - Geometric raycasting method
- `overlapping-chunks` - Grid-based chunking
- `rtree` - R-Tree spatial indexing
- `kdtree` - KD-Tree spatial indexing (default)
- `grid` - Grid-based nearest neighbor

### 3. Custom Distance Cutoff

```bash
cargo run -- test --cutoff 75
```

**Effects:**
- Only matches within 75 meters included
- Changes which addresses appear in windows
- Affects match count and statistics

### 4. Custom Number of Windows

```bash
cargo run -- test --windows 20
```

**Behavior:**
- Opens up to 20 windows
- If fewer than 20 matches exist, opens all available
- Randomly samples if more matches than requested windows

### 5. Combine All Parameters

```bash
cargo run -- test \
  --algorithm rtree \
  --cutoff 100 \
  --windows 15
```

**Result:**
- 15 windows (or fewer if < 15 matches)
- R-Tree algorithm
- 100 meter distance threshold

## What Each Window Shows

### Tab 1: StadsAtlas Integration

**URL:** https://stadsatlas.malmo.se/stadsatlas/

**You must manually:**
1. Scroll down or look for the "Miljöparkering" option
2. Click the checkbox to enable it
3. Click in the search bar: "Sök adresser eller platser..."
4. Type the address from Tab 2
5. Review the parking regulations shown

### Tab 2: Correlation Result Details

**Shows:**
- Address tested
- Postal code
- Data source (Miljödata, Parkering, or both)
- Distance to nearest match
- Zone information

**Compare with Tab 1:**
- Verify StadsAtlas shows matching zone
- Check that distance seems reasonable
- Assess correlation accuracy

## Common Use Cases

### Testing Algorithm Performance

```bash
# Test KD-Tree
cargo run -- test --algorithm kdtree --windows 10

# Test R-Tree
cargo run -- test --algorithm rtree --windows 10

# Test Raycasting
cargo run -- test --algorithm raycasting --windows 10

# Manually compare results in StadsAtlas
```

### Validating Distance Threshold

```bash
# Conservative: 25m threshold
cargo run -- test --cutoff 25 --windows 5

# Standard: 50m threshold
cargo run -- test --cutoff 50 --windows 5

# Permissive: 100m threshold
cargo run -- test --cutoff 100 --windows 5

# Compare accuracy vs. match count
```

### Testing Specific Data Quality

```bash
# Test on random sample
cargo run -- test --windows 20

# If accuracy is low, try different algorithm
cargo run -- test --algorithm overlapping-chunks --windows 20

# If still low, increase cutoff
cargo run -- test --algorithm overlapping-chunks --cutoff 100 --windows 20
```

## Interpreting Results

### Good Correlation
- StadsAtlas zone matches Tab 2 information
- Distance shown (e.g., "15.3m away") seems reasonable
- Zone name and regulations align with address

### Poor Correlation
- StadsAtlas shows different zone
- Distance is at or very close to cutoff threshold (e.g., "49.8m away")
- Zone information doesn't match visible map features

### No Match
- Tab 2 shows "No matches found"
- Indicates address outside all zones or beyond cutoff
- Try increasing cutoff: `--cutoff 100`

## Troubleshooting

### "No matching addresses found for testing!"

**Cause:** Correlation found no addresses within cutoff distance

**Solutions:**
```bash
# Increase cutoff
cargo run -- test --cutoff 100 --windows 10

# Try different algorithm
cargo run -- test --algorithm raycasting --windows 10

# Check if data is loaded
cargo run -- correlate
```

### Windows Not Opening

**Windows OS:**
- Ensure default browser is configured
- Check browser is not blocked by admin policies

**macOS:**
- Grant terminal permission to control applications
- Or use: `open -n https://stadsatlas.malmo.se/stadsatlas/`

**Linux:**
- Ensure `xdg-open` is installed: `which xdg-open`
- Or set `BROWSER` environment variable: `export BROWSER=firefox`

### StadsAtlas Search Not Working

1. Verify you're in the correct region (Malmö)
2. Try clicking the location icon first
3. Zoom map to Malmö area
4. Try entering just street name without number

### Data Tab Not Showing

1. Browser might have blocked data URL
2. Try refreshing the page
3. Check browser console for errors (F12 -> Console)
4. Try different browser (Chrome, Firefox, etc.)

## Command Reference

```bash
# Show help
cargo run -- test --help

# Default test
cargo run -- test

# Short flags
cargo run -- test -a kdtree -c 50 -w 10

# Long flags
cargo run -- test \
  --algorithm kdtree \
  --cutoff 50 \
  --windows 10

# Also available
cargo run -- correlate --cutoff 75          # One-off correlation with cutoff
cargo run -- benchmark --sample-size 100 --cutoff 50  # Benchmark with cutoff
```

## Tips for Accurate Testing

1. **Start with Default Settings**
   ```bash
   cargo run -- test
   ```
   - Establishes baseline accuracy
   - Uses proven KD-Tree algorithm
   - Standard 50m cutoff

2. **Test Incrementally**
   ```bash
   cargo run -- test --windows 5    # Quick test
   # Review results
   cargo run -- test --windows 20   # Larger sample
   ```

3. **Compare Algorithms**
   - Test each algorithm with same cutoff
   - Use same number of windows for fair comparison
   - Document which performs best for your data

4. **Adjust Cutoff Based on Results**
   - If getting false positives: decrease cutoff
   - If missing valid addresses: increase cutoff
   - Find sweet spot for your use case

5. **Check Data Quality**
   - If all algorithms fail: data might be outdated
   - Run: `cargo run -- check-updates`
   - Verify checksums haven't changed

## Performance Notes

- **Window Opening:** ~500ms delay between each (system stability)
- **10 windows:** ~5 seconds to fully open all
- **20 windows:** ~10 seconds to fully open all
- **Correlation Runtime:** Depends on data size and algorithm
  - KD-Tree: Typically 2-5 seconds
  - R-Tree: Typically 2-5 seconds
  - Raycasting: Typically 3-8 seconds

## Next Steps After Testing

1. **Document Findings**
   - Record which algorithm performs best
   - Note optimal cutoff value
   - Document accuracy percentage

2. **Update Configuration**
   - Set default algorithm in code if preferred
   - Update default cutoff if needed
   - Store benchmark results

3. **Production Deployment**
   ```bash
   cargo run --release -- correlate --algorithm kdtree --cutoff 50
   ```
   - Release mode for performance
   - Use tested algorithm and cutoff

---

**Status:** ✅ Ready for testing

**Questions?** Check `TESTING_CHANGES.md` for detailed documentation
