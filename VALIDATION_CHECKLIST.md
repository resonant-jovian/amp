# Validation Checklist

> **Note:** This document has been superseded by the comprehensive validation documentation.
> See **[docs/COMPLETION_VALIDATION.md](docs/COMPLETION_VALIDATION.md)** for the complete validation checklist.

---

## Quick Validation

For immediate validation, run:

```bash
# Format, lint, and check code
./scripts/fmt_fix_clippy.sh

# Run all tests
cargo test --all-targets --all-features --release

# Build all packages
cargo build --release --workspace
```

---

## Comprehensive Documentation

For detailed validation procedures, see:

- **[docs/COMPLETION_VALIDATION.md](docs/COMPLETION_VALIDATION.md)** - Complete validation checklist
- **[docs/testing.md](docs/testing.md)** - Testing strategies and CI/CD pipeline
- **[scripts/README.md](scripts/README.md)** - Build script documentation

---

## CI/CD Status

[![CI](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml/badge.svg)](https://github.com/resonant-jovian/amp/actions/workflows/ci.yml)
[![Tests](https://github.com/resonant-jovian/amp/actions/workflows/test.yml/badge.svg)](https://github.com/resonant-jovian/amp/actions/workflows/test.yml)

All commits are automatically validated via GitHub Actions.
