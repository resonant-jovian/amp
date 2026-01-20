# Android Test Workflow - Fixes & Iteration Guide

## Changes Implemented (Commit 79dea6ec)

### Problem Analysis
The original workflow timed out at 60 minutes due to:
1. **Interactive SDK license prompts** blocking `sdkmanager` indefinitely
2. **API 36 system image not preinstalled** requiring full download (~500MB+)
3. **Insufficient job timeout** for all initialization + tests
4. **Poor error detection** - continued on errors instead of failing fast
5. **No caching** of SDK/Gradle artifacts

### Solutions Implemented

#### 1. Non-interactive License Acceptance
```bash
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
```
- Prevents hanging on Android license prompts
- Runs before SDK installation (5-minute timeout)
- Gracefully continues if already accepted

#### 2. Lower API Level (33→33/34 fallback)
- API 33 more likely preinstalled on `ubuntu-24.04`
- Faster to download if not cached
- Fallback to API 34 if 33 unavailable
- Saves ~10-15 minutes vs API 36

#### 3. Proper Caching
- **Gradle cache**: `.gradle/caches`, `~/.gradle/`
- **Android SDK cache**: `~/.android` (includes system images)
- Keyed by workflow file hash (for invalidation on changes)

#### 4. Fail-Fast Architecture
- Removed `continue-on-error: true` from critical steps
- Early exit if:
  - License acceptance fails
  - System image installation fails
  - Emulator doesn't start
  - Boot doesn't complete
  - APK build fails
  - Tests fail

#### 5. Timeouts
- Job: **90 minutes** (increased from 60)
- License acceptance: 5 min
- SDK setup: 20 min
- Emulator setup: 15 min
- Emulator start: 25 min
- APK build: 30 min
- Tests: 20 min
- Slack for overhead: ~5 min

#### 6. Better Error Diagnostics
- Check emulator PID immediately after launch
- Detect adb device with 120s timeout (fail if not found)
- Detect boot complete with 120s timeout (fail if not boots)
- Print emulator log on process death
- Print device state on any failure

## Next Steps if Tests Still Fail

### Scenario A: "Setup Android SDK" Step Times Out (>20 min)
**Likely cause**: System image download blocked or slow

**Fix**:
```yaml
- name: Preinstall system image
  run: |
    # Download image outside of main workflow
    # or use a custom Docker image with preinstalled images
```

**Or** use GitHub-hosted runner with preinstalled image:
- Check [actions/runner-images](https://github.com/actions/runner-images) for preinstalled Android images
- May need specific runner (`ubuntu-22.04` vs `ubuntu-24.04`)

### Scenario B: "Start Android Emulator" Step Fails
**Likely causes**:
1. Emulator process crashes immediately → Check emulator log
2. adb never detects device → Check ANDROID_HOME path
3. Device boots slowly → Increase timeout

**Debugging**:
```bash
# Print full emulator log
tail -200 emulator.log

# Check if process is running
ps aux | grep emulator

# Check if adb sees it
adb devices -l

# Check if boot actually happening
adb shell getprop | grep boot
```

**Fixes**:
- Add more memory: `-memory 8192` (if available on runner)
- Disable unnecessary hardware: `-no-sim`, `-no-network`
- Use CPU acceleration: Already in workflow (`-accel on`)
- Use `-snapshot-list` to verify snapshot state

### Scenario C: "Build Android APK" Fails
**Likely cause**: `dx serve` or Gradle issue

**Debugging**:
```bash
# Try building manually
cargo fetch
cargo build --target aarch64-linux-android --release

# Check if amp-android package exists
cargo metadata --format-version 1 | grep -A3 'amp-android'
```

**Fixes**:
- Increase APK build timeout (currently 30 min)
- Cache Cargo artifacts:
```yaml
- name: Cache Cargo
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### Scenario D: "Run tests" Step Fails
**Likely cause**: Instrumentation tests not found or timeout

**Debugging**:
```bash
# List available tests
cargo test --target aarch64-linux-android --no-run -- --list

# Check if emulator is still running
adb devices
adb shell ps

# Check for test failures
adb logcat | grep -i test
```

**Fixes**:
- Verify test target is correct
- Increase test timeout (currently 20 min)
- Run simpler smoke test first:
```bash
# Instead of cargo test, try:
adb shell am instrument -w com.example.tests
```

## Monitoring the Workflow

1. **Check workflow runs**: [amp/actions](https://github.com/resonant-jovian/amp/actions)
2. **View run details**: Click the workflow run → android-tests job
3. **Download artifacts**: Check "Artifacts" section for:
   - `emulator-logs` → `emulator.log` (full emulator output)
   - `test-results` → `build/` (test results if any)

## Iterative Improvement Process

If the current workflow fails:

1. **Identify which step failed** (read the error in workflow log)
2. **Find the matching scenario** above (A, B, C, or D)
3. **Apply the fix** to the workflow file
4. **Commit and push** to trigger workflow again
5. **Check new logs** for progress
6. **Repeat until passing**

## Workflow File Location

`.github/workflows/android-test.yml`

### Key Sections to Modify

| Section | Purpose | Timeout | If Failing... |
|---------|---------|---------|---------------|
| Accept Android SDK licenses | License acceptance | 5 min | Check if licenses cached |
| Setup Android SDK (API 33) | Download/install system image | 20 min | Try API 32/34 or add docker image |
| Setup Android emulator | Create AVD config | 15 min | Verify storage space |
| Start Android emulator | Boot emulator | 25 min | Check process/logs, increase memory |
| Build Android APK | Compile APK | 30 min | Check cargo, increase timeout |
| Run tests | Execute tests | 20 min | Verify test target, check logcat |

## Key Commands for Local Testing

```bash
# Create AVD locally (for testing)
yes | sdkmanager --licenses
yes | sdkmanager "system-images;android-33;google_apis;x86_64"
emulator -avd TestDevice -no-window &
adb wait-for-device
adb shell getprop sys.boot_completed

# Test Rust build
cargo fetch
cargo test --target aarch64-linux-android --no-run

# Check emulator logs
logcat -d > device_log.txt
```

## Success Criteria

Workflow passes when:
- ✅ All steps complete within their timeouts
- ✅ Job completes in < 90 minutes
- ✅ Tests run and produce results
- ✅ Artifacts uploaded (even if test fails)
- ✅ No "operation was canceled" message

## Quick Reference: Common Fixes

```yaml
# If API 33 unavailable, try API 32
system-images;android-32;google_apis;x86_64

# If memory issues
-memory 8192  # Increase from 4096

# If network issues
CARGO_NET_RETRY=10  # More retries

# If timeout issues
timeout-minutes: 120  # Increase from 90
```

## Emergency: Hard Reset

If workflow is stuck in a bad state:

```bash
# Clear all Android cache
rm -rf ~/.android

# Clear all Gradle cache
rm -rf ~/.gradle

# Clear Cargo cache (careful!)
rm -rf ~/.cargo/registry
```

Then push a new commit to trigger fresh workflow run.

---

**Last Updated**: 2026-01-20  
**Commit**: 79dea6ec (Initial comprehensive fixes)  
**Next Review**: After first workflow run completion
