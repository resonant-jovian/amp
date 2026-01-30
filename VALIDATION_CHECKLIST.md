# Validation Checklist

## Pre-Merge Validation Steps

Run these commands locally to validate the refactoring before merging to main.

### 1. Pull Latest Changes

```bash
git fetch origin
git checkout refactor/comprehensive-2026-01
git pull origin refactor/comprehensive-2026-01
```

### 2. Format and Lint Code

```bash
./scripts/fmt_fix_clippy.sh
```

**Expected result:** All code formatted, clippy warnings fixed, no errors.

### 3. Compile Android App

```bash
cd android
cargo build --release
```

**Expected result:** Clean compilation with no errors.

**Common issues:**
- Missing dependencies: Run `cargo fetch` first
- Wrong Rust version: Ensure Rust 1.70+ with `rustc --version`

### 4. Run Tests

```bash
cd android
cargo test --release
```

**Expected result:** All tests pass.

### 5. Check for Unused Dependencies

```bash
cd android
cargo check
```

**Expected result:** No warnings about unused imports or variables.

### 6. Build Android APK (Optional)

Only if you have Android SDK and keystore configured:

```bash
./scripts/build.sh
```

**Expected result:** Signed APK in `target/dx/amp/release/android/app/app/build/outputs/apk/release/`

### 7. Verify File Structure

```bash
# Check that old files are deleted
test ! -f android/src/ui/adresser.rs && echo "✓ adresser.rs deleted"
test ! -f android/src/ui/paneler.rs && echo "✓ paneler.rs deleted"
test ! -f android/src/ui/topbar.rs && echo "✓ topbar.rs deleted"

# Check that new files exist
test -f android/src/ui/addresses.rs && echo "✓ addresses.rs exists"
test -f android/src/ui/panels.rs && echo "✓ panels.rs exists"
test -f android/src/ui/top_bar.rs && echo "✓ top_bar.rs exists"

# Check that scripts are in place
test -f scripts/build.sh && echo "✓ scripts/build.sh exists"
test -f scripts/serve.sh && echo "✓ scripts/serve.sh exists"
test -f scripts/adb-install.sh && echo "✓ scripts/adb-install.sh exists"
test -f scripts/fmt_fix_clippy.sh && echo "✓ scripts/fmt_fix_clippy.sh exists"
```

### 8. Verify Documentation

```bash
# Generate and check rustdoc
cd android
cargo doc --no-deps --open
```

**Expected result:** Documentation generates without warnings, opens in browser.

### 9. Search for Remaining Swedish Terms

```bash
cd android/src

# Search for Swedish variable names (should find none in code)
grep -r "gata[^l]" . --include="*.rs" || echo "✓ No 'gata' found"
grep -r "gatunummer" . --include="*.rs" || echo "✓ No 'gatunummer' found"
grep -r "postnummer" . --include="*.rs" || echo "✓ No 'postnummer' found"
grep -r "\<dag\>" . --include="*.rs" || echo "✓ No 'dag' found"
grep -r "\<tid\>" . --include="*.rs" || echo "✓ No 'tid' found"
grep -r "AdressInfo" . --include="*.rs" || echo "✓ No 'AdressInfo' found"

# These Swedish UI strings SHOULD be found (user-facing text)
grep -r "Adresser" . --include="*.rs" && echo "✓ Swedish UI text preserved"
grep -r "Städas nu" . --include="*.rs" && echo "✓ Swedish UI text preserved"
```

### 10. Verify Imports

```bash
cd android/src/ui

# Check mod.rs has correct imports
grep "pub mod addresses" mod.rs && echo "✓ addresses module declared"
grep "pub mod panels" mod.rs && echo "✓ panels module declared"
grep "pub mod top_bar" mod.rs && echo "✓ top_bar module declared"

grep "use crate::ui::addresses::Addresses" mod.rs && echo "✓ Addresses imported"
grep "use crate::ui::panels::" mod.rs && echo "✓ Panel components imported"
grep "use crate::ui::top_bar::TopBar" mod.rs && echo "✓ TopBar imported"
```

---

## Validation Checklist Summary

- [ ] Code formatted with `./scripts/fmt_fix_clippy.sh`
- [ ] Android app compiles: `cd android && cargo build --release`
- [ ] All tests pass: `cd android && cargo test --release`
- [ ] No clippy warnings: `./scripts/fmt_fix_clippy.sh` completes clean
- [ ] Old files deleted (adresser.rs, paneler.rs, topbar.rs)
- [ ] New files exist (addresses.rs, panels.rs, top_bar.rs)
- [ ] Scripts in scripts/ directory
- [ ] Documentation generates without warnings
- [ ] No Swedish variable names in code
- [ ] Swedish UI text preserved
- [ ] Correct module imports in mod.rs

---

## If All Checks Pass

You're ready to merge!

```bash
# Switch to main
git checkout main

# Merge refactoring branch
git merge refactor/comprehensive-2026-01

# Push to remote
git push origin main

# Optional: Delete refactoring branch
git branch -d refactor/comprehensive-2026-01
git push origin --delete refactor/comprehensive-2026-01
```

---

## If Issues Found

1. Note the specific error messages
2. Fix issues on the refactor branch
3. Commit fixes with semantic messages
4. Re-run validation checklist
5. Repeat until all checks pass

---

## Quick Validation Script

Run all checks at once:

```bash
#!/bin/bash
set -e

echo "🔍 Running comprehensive validation..."
echo ""

echo "1. Format and lint..."
./scripts/fmt_fix_clippy.sh

echo ""
echo "2. Build Android..."
cd android && cargo build --release

echo ""
echo "3. Run tests..."
cargo test --release

echo ""
echo "4. Generate documentation..."
cargo doc --no-deps

cd ..

echo ""
echo "✅ All validation checks passed!"
echo ""
echo "Ready to merge to main."
```

Save this as `validate.sh` and run: `chmod +x validate.sh && ./validate.sh`
