# Testing Documentation

## CI/CD Status

[![CI](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml)

### Individual Check Status

| Check | Status | Description |
|-------|--------|-------------|
| **Format** | [![Format](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg?job=format)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml) | Code formatting with `cargo fmt` |
| **Clippy** | [![Clippy](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg?job=clippy)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml) | Linting with `cargo clippy` |
| **Build** | [![Build](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg?job=build)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml) | Release build compilation |
| **Test** | [![Test](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg?job=test)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml) | Unit and integration tests |
| **Doc** | [![Doc](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg?job=doc)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml) | Documentation generation |

---

## Automated Testing (CI/CD)

### GitHub Actions Workflow

The project uses GitHub Actions for continuous integration. All checks run automatically on:
- Every push to `main`, `feature/*`, and `refactor/*` branches
- Every pull request to `main`

**Workflow file:** [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

### CI Jobs

#### 1. Format Check
- **Tool:** `cargo fmt`
- **Scope:** Core and Android crates
- **Failure condition:** Code not formatted according to Rust style guidelines

#### 2. Clippy Lint
- **Tool:** `cargo clippy`
- **Scope:** All targets and features
- **Failure condition:** Any warnings (treated as errors with `-D warnings`)
- **Cache:** Uses cargo cache for faster builds

#### 3. Build
- **Tool:** `cargo build --release`
- **Scope:** Core and Android crates (parallel matrix)
- **Failure condition:** Compilation errors
- **Cache:** Uses cargo cache for faster builds

#### 4. Test
- **Tool:** `cargo test --release`
- **Scope:** Core and Android crates (parallel matrix)
- **Failure condition:** Any test failures
- **Cache:** Uses cargo cache for faster builds

#### 5. Documentation
- **Tool:** `cargo doc`
- **Scope:** Core and Android crates
- **Failure condition:** Documentation warnings or errors
- **Environment:** `RUSTDOCFLAGS: -D warnings`

---

## Local Testing

### Quick Validation

Run all checks locally before pushing:

```bash
# From repository root
./validate.sh
```

This runs:
1. Format and lint (`./scripts/fmt_fix_clippy.sh`)
2. Build (release mode)
3. Tests (release mode)
4. Documentation generation
5. File structure verification
6. Swedish variable name check

### Individual Commands

#### Format Code

```bash
# Format all code
./scripts/fmt_fix_clippy.sh

# Or manually:
cd core && cargo fmt
cd android && cargo fmt
```

#### Run Clippy

```bash
# Lint all code
cd core && cargo clippy --all-targets --all-features -- -D warnings
cd android && cargo clippy --all-targets --all-features -- -D warnings
```

#### Build

```bash
# Build release
cd android && cargo build --release

# Build debug (faster)
cd android && cargo build
```

#### Run Tests

```bash
# Run all tests
cd core && cargo test --release
cd android && cargo test --release

# Run specific test
cd android && cargo test test_name

# Run with output
cd android && cargo test -- --nocapture
```

#### Generate Documentation

```bash
# Generate and open docs
cd android && cargo doc --no-deps --open

# Generate without opening
cd core && cargo doc --no-deps
cd android && cargo doc --no-deps
```

---

## Test Coverage

### Current Test Suite

**Core Crate:**
- Model serialization/deserialization tests
- Data validation tests

**Android Crate:**
- Address matching algorithm tests
- Countdown calculation tests
- Fuzzy matching tests
- UI component tests

### Running with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cd android
cargo tarpaulin --out Html --output-dir coverage

# Open coverage report
open coverage/index.html
```

---

## Pre-Commit Hooks (Optional)

Set up automatic formatting and linting before commits:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

**`.pre-commit-config.yaml`** (create in repo root):

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt
        language: system
        types: [rust]
        pass_filenames: false
      
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
```

---

## Continuous Deployment

### Android APK Build

To build and sign the Android APK:

```bash
./scripts/build.sh
```

**Requirements:**
- Android SDK installed
- NDK installed
- Keystore configured
- `dx` tool installed

**Output:** `target/dx/amp/release/android/app/app/build/outputs/apk/release/app-release.apk`

### Installation

```bash
# Install to connected device
./scripts/adb-install.sh
```

---

## Troubleshooting

### Common Issues

#### Format Check Fails

```bash
# Fix automatically
cd core && cargo fmt
cd android && cargo fmt
```

#### Clippy Warnings

```bash
# See all warnings
cd android && cargo clippy --all-targets --all-features

# Fix common issues
./scripts/fmt_fix_clippy.sh
```

#### Build Fails

```bash
# Clean and rebuild
cd android && cargo clean && cargo build --release

# Update dependencies
cd android && cargo update
```

#### Tests Fail

```bash
# Run with verbose output
cd android && cargo test -- --nocapture

# Run specific failing test
cd android && cargo test test_name -- --nocapture
```

#### Documentation Warnings

```bash
# Check warnings
cd android && cargo doc --no-deps 2>&1 | grep warning

# Fix missing docs
# Add /// comments to public items
```

---

## Best Practices

### Before Committing

1. ✅ Run `./validate.sh`
2. ✅ All checks pass
3. ✅ Commit with semantic message
4. ✅ Push to remote

### Before Creating PR

1. ✅ All local tests pass
2. ✅ Code formatted
3. ✅ No clippy warnings
4. ✅ Documentation updated
5. ✅ CI checks passing on branch

### Code Quality Standards

- **Format:** Use `cargo fmt` default style
- **Linting:** Zero clippy warnings
- **Documentation:** All public items documented
- **Tests:** Maintain or improve coverage
- **Commits:** Use semantic commit messages

---

## CI/CD Architecture

```
Push/PR
   ↓
GitHub Actions
   ↓
   ├─→ Format Check (cargo fmt)
   ├─→ Clippy Lint (cargo clippy)
   ├─→ Build (cargo build --release)
   │   ├─→ Core crate
   │   └─→ Android crate
   ├─→ Test (cargo test --release)
   │   ├─→ Core crate
   │   └─→ Android crate
   └─→ Documentation (cargo doc)
       ├─→ Core crate
       └─→ Android crate
   ↓
All Pass ✅ → Merge allowed
Any Fail ❌ → Fix required
```

---

## Performance Benchmarks

Add benchmarks for critical paths:

```bash
# Run benchmarks
cd android && cargo bench

# Run specific benchmark
cd android && cargo bench matching
```

---

## Related Documentation

- **[Validation Checklist](../VALIDATION_CHECKLIST.md)** - Pre-merge validation
- **[Scripts README](../scripts/README.md)** - Build and dev scripts
- **[Contributing Guide](../CONTRIBUTING.md)** - Contribution guidelines
- **[CI Workflow](../.github/workflows/ci.yml)** - GitHub Actions config

---

## Status History

| Date | Format | Clippy | Build | Test | Doc |
|------|--------|--------|-------|------|-----|
| 2026-01-30 | ✅ | ✅ | ✅ | ✅ | ✅ |

---

**Last Updated:** January 30, 2026
