# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AMP (Address-to-Miljözone Parking) is a Rust monorepo that correlates street addresses with parking restriction zones in Malmö, Sweden. It uses geospatial algorithms to match addresses to environmental parking zones and paid parking areas, with offline-first mobile apps built using Dioxus.

## Build & Development Commands

```bash
# Build everything
cargo build --release

# Run all tests
cargo test --release

# Run tests for a specific crate
cargo test -p amp_core
cargo test -p amp_server

# Run a single test by name
cargo test -p amp_core test_db_from_dag_tid

# Format, fix, and lint
./scripts/fmt_fix_clippy.sh                               # format

# CLI tool commands
./scripts/check_update.sh
./scripts/benchmark.sh
./scripts/correlate.sh
./scripts/test.sh

# Run emulator
emulator -avd Pixel_7_API_36

# Android build
dx build --platform amp-android --release --android
./scripts/build.sh              # setup notifications + build APK
adb uninstall se.malmo.skaggbyran.amp
./scripts/adb-install.sh

# Android tests
cd android && cargo test --lib notifications
cd android && cargo test --lib transitions
cd android && cargo test --test notification_integration_tests
```

## Architecture

### Workspace Crates

- **`core/`** (`amp_core`) — Core library: data structures, GeoJSON API client, Parquet I/O, correlation algorithms
- **`server/`** (`amp_server`) — CLI tool for testing, benchmarking, and running correlations
- **`android/`** — Dioxus-based Android app with JNI bridge to Kotlin
- **`ios/`** — Dioxus-based iOS app (shares UI patterns with Android)
- **`scripts/`** — Build automation (bash scripts)

### Data Flow

GeoJSON from Malmö Open Data API → `amp_core` algorithms → pre-computed Parquet files → embedded in mobile apps for O(1) lookups.

Three data sources: addresses (`adresser.json`), environmental parking zones (`miljoparkeringar.json`), and paid parking zones (`parkeringsavgifter.json`).

### Core Data Types (`core/src/structs.rs`)

- `AdressClean` — Street address with coordinates
- `MiljoeDataClean` / `ParkeringsDataClean` — Parking restriction zones from GeoJSON
- `DB` — Time-aware parking restriction entry (the main correlated result)
- `LocalData` — User's saved address with matched restrictions
- `SettingsData` — App preferences (notifications, theme, language)

### Correlation Algorithms (`core/src/correlation_algorithms/`)

Six implementations of the `CorrelationAlgo` trait, selectable at runtime:
- **KD-Tree** (default/recommended) — O(log n) lookups
- **R-Tree** — Alternative spatial index
- **Grid-based** — Spatial hashing, O(1) average
- **Overlapping Chunks** — Parallelizable chunked approach
- **Distance-based** — Brute-force baseline
- **Raycasting** — Polygon containment tests

### Android App Layers (`android/src/`)

- `ui/` — Dioxus RSX components (top bar, panels, address list, dialogs)
- `components/` — Business logic (storage, matching, notifications, countdown timers, transitions, settings)
- `android_bridge.rs` — JNI bridge to Kotlin helpers (`android/kotlin/`)
- `components/storage.rs` — Parquet-based persistence for user data
- `components/static_data.rs` — Embedded parking database from assets

### Key Patterns

- **Time handling**: All times stored as UTC, displayed in Swedish timezone (`Europe/Stockholm`) via `chrono-tz`. Year validation range: 2020–2100.
- **Parquet storage**: Used for both pre-computed correlation data and user data persistence. ~90% smaller than GeoJSON.
- **Notification system**: Three-tier channels (Active Now, 6 Hours, 1 Day) with transition-based triggering via JNI to Android NotificationManager.
- **Platform conditionals**: `#[cfg(target_os = "android")]` gates Android-specific code.
- **Fuzzy matching**: Levenshtein distance via `strsim` for address search.

## CI

GitHub Actions runs on push to `main`, `feature/*`, `refactor/*` branches and PRs to `main`:
- `cargo fmt --check` per crate (core, server, android)
- `cargo clippy -- -D warnings` per crate
- `cargo test --release` for core and server
- Separate workflows for correlation algorithm tests, server benchmarks, and Android APK builds
