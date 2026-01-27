# AMP Server - CLI Integration Guide

## Quick Start

### Build

```bash
# From workspace root
cd amp
cargo build --release

# Binary location
./target/release/amp-server
```

### Help

```bash
./target/release/amp-server --help
./target/release/amp-server correlate --help
./target/release/amp-server test --help
./target/release/amp-server benchmark --help
./target/release/amp-server check-updates --help
./target/release/amp-server tui --help
```

---

## Commands

### 1. Interactive TUI

```bash
./target/release/amp-server tui
```

**Features:**
- 6 main views (Dashboard, Correlate, Results, Benchmark, Updates, Help)
- Algorithm multi-selection with checkboxes
- Live log panel with color-coded entries
- Performance visualization (bar + line charts)
- Theme switcher (Light/Dark/Auto)
- Responsive layouts for different terminal sizes
- Full keyboard navigation

**Keyboard Shortcuts:**
- `[1-6]` - Jump to tab
- `[←→]` - Tab navigation
- `[↑↓]` - Scroll content
- `[Space]` - Select algorithm
- `[a]` - Select all algorithms
- `[c]` - Clear all algorithms
- `[+/-]` - Adjust cutoff distance
- `[t]` - Toggle theme
- `[Enter]` - Run operation
- `[?]` - Show help
- `[Ctrl+C]` or `[q]` - Quit

---

### 2. Correlate (CLI)

Run correlation with specified algorithm.

```bash
# Default: KDTree, 50m cutoff
./target/release/amp-server correlate

# Custom algorithm
./target/release/amp-server correlate --algorithm kdtree --cutoff 50

# Available algorithms:
# - distance-based
# - raycasting
# - overlapping-chunks
# - rtree
# - kdtree (default)
# - grid
```

**Output:**
- Dataset information
- Progress bars for each dataset correlation
- Execution time
- Result statistics (total matches, by dataset, percentages)
- 10 random matches
- Top 10 addresses with largest distances
- Threshold verification

**Example:**
```
📋 Dataset Information:
   Correlating with: Miljödata + Parkering (dual dataset)
   Addresses: 12345
   Miljödata zones: 5678
   Parkering zones: 9012
   Distance threshold: 50 meters

🚀 Running correlation with KDTree algorithm
[████████████████████] 12345/12345 100% ✓ Completed in 45.23s

📊 Results:
   Addresses processed: 12345
   Total matches: 9876 (80.0%)
   ├─ Both datasets: 5000 (40.5%)
   ├─ Miljödata only: 3000 (24.3%)
   ├─ Parkering only: 1876 (15.2%)
   └─ No match: 2469 (20.0%)
```

---

### 3. Test Mode (CLI)

Generate HTML test pages and open in browser for visual verification.

```bash
# Open 10 browser windows with random matching addresses
./target/release/amp-server test --windows 10

# With custom algorithm and cutoff
./target/release/amp-server test \
  --algorithm kdtree \
  --cutoff 50 \
  --windows 5
```

**What Happens:**
1. Loads address and zone data
2. Runs correlation
3. Finds matching addresses
4. Randomly selects N addresses
5. Generates complete HTML test pages with:
   - Tab 1: Address search with embedded Origo map
   - Tab 2: Step-by-step instructions
   - Tab 3: Correlation data visualization
   - Tab 4: Debug console
6. Opens each in a new browser window
7. Files saved as `amp_test_0.html`, `amp_test_1.html`, etc. in system temp

**Browser Requirements:**
- Firefox or Chrome/Chromium recommended
- JavaScript enabled
- Pop-ups allowed for multiple windows

**Output:**
```
📋 Test Mode Configuration:
   Algorithm: KDTree
   Distance threshold: 50 meters
   Browser windows to open: 5
   Total addresses available: 12345

📊 Correlation Results:
   Total matches found: 9876
   Windows to open: 5 (sample size from 9876 matches)

🌐 Opening 5 browser windows...
   Each window has 4 integrated tabs with nested StadsAtlas map:
   - Tab 1: Address search with nested StadsAtlas map
   - Tab 2: Step-by-step instructions
   - Tab 3: Correlation data visualization
   - Tab 4: Debug console with address search logs

   [1/5] Opening window for: Stortorget 1, 211 34 Malmö
   [2/5] Opening window for: Gustaf Paulssons väg 18, 211 52 Malmö
   [3/5] Opening window for: Nils Hanssons plats 5, 211 23 Malmö
   [4/5] Opening window for: Södra vägen 123, 211 43 Malmö
   [5/5] Opening window for: Friisgatan 2, 211 17 Malmö

✅ Test mode complete!
   Review the 5 opened windows to verify correlation accuracy.
```

---

### 4. Benchmark (CLI)

Benchmark all or selected algorithms.

```bash
# Default: 100 addresses, 50m cutoff
./target/release/amp-server benchmark

# Custom sample size
./target/release/amp-server benchmark --sample-size 500 --cutoff 75
```

**Interactive Selection:**
```
🔧 Algorithm Selection (Y/N to include, default is Y if just Enter is pressed):

   Include Distance-Based benchmark? [Y/n]: ✓ Distance-Based selected
   Include Raycasting benchmark? [Y/n]: n
   ✗ Raycasting skipped
   Include Overlapping Chunks benchmark? [Y/n]: ✓ Overlapping Chunks selected
   Include R-Tree benchmark? [Y/n]: ✓ R-Tree selected
   Include KD-Tree benchmark? [Y/n]: ✓ KD-Tree selected
   Include Grid benchmark? [Y/n]: ✓ Grid selected
```

**Benchmark Progress:**
```
🟁 Benchmarking 5 selected algorithm(s) with 100 samples (distance cutoff: 50m)

✅ [Distance-Based   ] [████████████████████] 100/100 ✓ 123.45ms
✅ [Overlapping Chun ] [████████████████████] 100/100 ✓ 89.23ms
✅ [R-Tree          ] [████████████████████] 100/100 ✓ 45.67ms
✅ [KD-Tree         ] [████████████████████] 100/100 ✓ 42.34ms
✅ [Grid            ] [████████████████████] 100/100 ✓ 38.92ms

📊 Benchmark Results (distance cutoff: 50m):

  Rank | Algorithm            | Time/Run | Total Time | Addresses | Matches
  -----|----------------------|----------|------------|-----------|--------
   1st | Grid                 |   0.39ms |  38.92ms   |       100 |    78
   2nd | KD-Tree              |   0.42ms |  42.34ms   |       100 |    78
   3rd | R-Tree               |   0.46ms |  45.67ms   |       100 |    78
   4th | Overlapping Chunks   |   0.89ms |  89.23ms   |       100 |    78
   5th | Distance-Based       |   1.23ms | 123.45ms   |       100 |    78
```

---

### 5. Check Updates (CLI)

Check for data updates from Mälmö open data portal.

```bash
# Default: saves to checksums.json
./target/release/amp-server check-updates

# Custom checksum file
./target/release/amp-server check-updates --checksum-file my-checksums.json
```

**Output (First Run):**
```
🔍 Checking for data updates...

🕐 Fetching remote data...
✓ Data fetched

✓ No previous checksums found - created new baseline
✓ Checksums saved to checksums.json
```

**Output (Subsequent Runs - No Changes):**
```
🔍 Checking for data updates...

🕐 Fetching remote data...
✓ Data fetched

✓ Data is up to date (no changes detected)
✓ Checksums saved to checksums.json
```

**Output (Data Changed):**
```
🔍 Checking for data updates...

🕐 Fetching remote data...
✓ Data fetched

✓ Data has changed!
  Old checksums from: 2026-01-20 14:30:45
  New checksums from: 2026-01-27 09:15:22
✓ Checksums saved to checksums.json
```

---

## Testing Sequence

### Test 1: Build Verification

```bash
cd amp
cargo build --release 2>&1 | grep -i "error\|warning"
echo "Build status: $?"
```

**Expected:** Exit code 0, no errors

### Test 2: CLI Help

```bash
./target/release/amp-server --help
./target/release/amp-server correlate --help
./target/release/amp-server test --help
./target/release/amp-server benchmark --help
./target/release/amp-server check-updates --help
```

**Expected:** All show help text without errors

### Test 3: Correlate Command

```bash
./target/release/amp-server correlate --algorithm kdtree --cutoff 50
```

**Expected:**
- Progress bars update
- Completes in ~30-60 seconds
- Shows results with statistics
- Shows 10 random matches
- Shows top 10 largest distances
- Verifies all within threshold

### Test 4: Test Mode (1 window)

```bash
./target/release/amp-server test --windows 1 --algorithm kdtree
```

**Expected:**
- Browser window opens with test page
- Page loads completely
- 4 tabs visible and clickable
- Map displays in Tab 1

### Test 5: Benchmark (Small Sample)

```bash
./target/release/amp-server benchmark --sample-size 50
```

**Expected:**
- Interactive prompt for algorithm selection
- Progress bars for each algorithm
- Comparative results table
- All algorithms complete in < 30 seconds

### Test 6: Check Updates

```bash
./target/release/amp-server check-updates
```

**Expected:**
- Fetches data successfully
- Creates or updates checksums.json
- Reports whether data changed

### Test 7: TUI Launch

```bash
./target/release/amp-server tui
```

**Expected:**
- Terminal clears and displays UI
- All tabs visible and selectable
- Responds to keyboard input
- No crash on operations
- Can quit with Ctrl+C

---

## Troubleshooting

### Build Fails

**Error:** `cannot find module cli`

**Solution:** 
```bash
# Run cargo clean and rebuild
cargo clean
cargo build --release
```

### Asset Files Not Found

**Error:** "Could not find asset file: stadsatlas_interface.html"

**Solution:** Ensure you're in the right directory:
```bash
# Must be in workspace root
pwd  # Should end with /amp
ls server/src/assets/

# Then run from here
cargo run --release -- test --windows 1
```

### Browser Won't Open

**Linux:** 
```bash
# Set BROWSER environment variable
export BROWSER=firefox
cargo run --release -- test --windows 1

# Or install chromium
sudo apt-get install chromium-browser
```

**Mac:**
```bash
# Should work automatically with Safari
# Or install Firefox/Chrome
```

**Windows:**
```bash
# Should detect Chrome automatically
# If not, set BROWSER environment variable
set BROWSER=chrome
```

### Terminal Issues

**Error:** "Failed to enter raw mode"

**Solution:**
```bash
# TUI requires proper terminal support
# SSH without -t flag doesn't work
ssh -t user@host
cargo run --release -- tui
```

### Partial Matches

**Issue:** Some algorithms return different match counts

**This is normal.** Different spatial algorithms may handle edge cases differently.

---

## Performance Notes

### Typical Execution Times

| Operation | Time | Note |
|-----------|------|------|
| Correlate (full dataset) | 30-90s | Depends on algorithm |
| Benchmark (100 addresses) | 30-45s | All 6 algorithms |
| Test mode (1 window) | 60-120s | Including browser startup |
| Check updates | 10-20s | Network dependent |
| TUI startup | < 100ms | Instant |

### Memory Usage

- Typical: 30-50MB
- Peak (full benchmark): ~100-150MB
- Should always stay < 500MB

### CPU Usage

- CLI commands: High (parallel processing)
- TUI mode: Low (only updates on changes)
- Benchmark: Very high (uses all cores)

---

## Next Steps

1. Test all commands locally
2. Verify HTML test pages in browser
3. Benchmark on your system
4. Deploy to production
5. Create automation scripts if needed

---

**Happy correlating! 🊀**
