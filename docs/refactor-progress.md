# Comprehensive Refactoring Progress

## Branch: refactor/comprehensive-2026-01
Created: 2026-01-30
Base: feature/android (SHA: 3ad83fe21c96a8dd94a3ff6321a49f95f4377623)

## Objectives

1. **Function Refactoring**: Break down large functions into smaller, focused units (<50 lines, SRP)
2. **Code Migration**: Sync android/ to iOS/ with platform-specific modules (GPS, storage, notifications)
3. **Deprecation Management**: Move unused code to deprecated.rs with proper annotations
4. **Dependency Cleanup**: Comment out unused imports in all Cargo.toml files
5. **Documentation Standardization**: Move all docs to docs/ folder or README files (minimal)
6. **Code Documentation**: Add /// doc comments to all public functions
7. **Naming Convention**: Translate Swedish/mixed names to English (snake_case, PascalCase, SCREAMING_SNAKE_CASE)
8. **Script Organization**: Move all .sh files to scripts/ folder with dynamic path resolution
9. **Dioxus 0.7.3 Migration**: Use #[derive(Routable)] and use_signal exclusively
10. **Testing & Validation**: Run cargo check for all features after each major change

## Repository Structure Analysis

### Current Structure (feature/android)
```
amp/
├── android/               # Android-specific mobile app
│   ├── src/
│   │   ├── components/    # Platform components (GPS, file, notification)
│   │   ├── ui/           # UI modules (Swedish naming)
│   │   ├── countdown.rs
│   │   ├── matching.rs
│   │   ├── static_data.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── README.md
├── ios/                  # iOS-specific mobile app (minimal/empty)
├── core/                 # Shared business logic
├── server/               # Server components
├── docs/                 # Documentation
├── scripts/              # Build and utility scripts (already exists)
├── *.sh                  # Shell scripts in root (need to move)
└── Cargo.toml           # Workspace root
```

### Target Structure (refactor/comprehensive-2026-01)
```
amp/
├── android/
│   ├── src/
│   │   ├── components/
│   │   │   ├── gps.rs       # Android GPS implementation
│   │   │   ├── storage.rs   # Android storage
│   │   │   ├── notification.rs # Android notifications
│   │   │   └── mod.rs
│   │   ├── ui/              # Refactored UI (English naming, routing)
│   │   │   ├── addresses.rs # Renamed from adresser.rs
│   │   │   ├── panels.rs    # Renamed from paneler.rs
│   │   │   ├── top_bar.rs   # Renamed from topbar.rs
│   │   │   └── mod.rs
│   │   ├── deprecated.rs    # Deprecated code
│   │   ├── matching.rs      # Refactored with smaller functions
│   │   ├── countdown.rs     # Refactored
│   │   ├── static_data.rs   # Refactored
│   │   └── main.rs          # Refactored entry point
│   └── Cargo.toml           # Cleaned dependencies
├── ios/
│   ├── src/
│   │   ├── components/
│   │   │   ├── gps.rs       # iOS GPS implementation
│   │   │   ├── storage.rs   # iOS storage
│   │   │   ├── notification.rs # iOS notifications
│   │   │   └── mod.rs
│   │   ├── ui/              # Copied from android (English naming)
│   │   ├── deprecated.rs
│   │   ├── matching.rs
│   │   ├── countdown.rs
│   │   ├── static_data.rs
│   │   └── main.rs
│   └── Cargo.toml
├── core/                    # Enhanced shared logic
│   ├── src/
│   │   ├── models/          # Shared data models
│   │   ├── utils/           # Shared utilities
│   │   └── lib.rs
├── server/                  # Server refactored
├── docs/
│   ├── refactor-progress.md # This file
│   ├── naming-migration.md  # Translation reference
│   ├── architecture.md      # Overall architecture
│   └── api/                 # API documentation
├── scripts/
│   ├── build.sh            # Moved from root
│   ├── serve.sh            # Moved from root
│   ├── adb-install.sh      # Moved from root
│   ├── fmt_fix_clippy.sh   # Already in scripts/ root
│   └── README.md           # Script documentation
└── Cargo.toml              # Workspace dependencies cleaned
```

## Key Issues Identified

### 1. Swedish/Mixed Naming
- `adresser` → `addresses`
- `paneler` → `panels`
- `gata` → `street`
- `gatunummer` → `street_number`
- `postnummer` → `postal_code`

### 2. Missing Dioxus 0.7.3 Routing
- No #[derive(Routable)] enum defined
- Using deprecated patterns
- Need to implement proper Router

### 3. Signal Usage
- Currently uses `use_signal` (CORRECT)
- Need to verify no deprecated `use_state` or `cx` usage

### 4. Empty Component Files
- `android/src/components/file.rs` (0 bytes)
- `android/src/components/geo.rs` (0 bytes)
- `android/src/components/notification.rs` (0 bytes)
- These should either be implemented or moved to deprecated.rs

### 5. Large Functions Detected
- `android/src/ui/mod.rs::App()` component (needs breakdown)
- Various UI rendering functions likely exceed 50 lines

## Refactoring Passes

### Pass 1: Function Refactoring & Code Organization ⏳
**Status**: Not Started  
**Target**: Break all functions into <50 line units with SRP

#### Sub-tasks:
- [ ] Analyze all .rs files for function sizes
- [ ] Identify functions exceeding 50 lines
- [ ] Create helper functions for complex logic
- [ ] Extract impl blocks for related functionality
- [ ] Document each function with /// comments

**Files to Process**:
- [ ] `android/src/ui/mod.rs`
- [ ] `android/src/ui/adresser.rs`
- [ ] `android/src/ui/paneler.rs`
- [ ] `android/src/ui/topbar.rs`
- [ ] `android/src/matching.rs`
- [ ] `android/src/countdown.rs`
- [ ] `android/src/static_data.rs`
- [ ] `core/src/**`
- [ ] `server/src/**`

### Pass 2: Unused Code & Deprecation ⏳
**Status**: Not Started  
**Target**: Move all unused code to deprecated.rs files

#### Sub-tasks:
- [ ] Scan for unused imports across all files
- [ ] Identify unused functions/structs
- [ ] Create deprecated.rs in android/src/
- [ ] Create deprecated.rs in ios/src/
- [ ] Move unused code with #[deprecated] attributes
- [ ] Ensure deprecated.rs is compilable
- [ ] Update module declarations

### Pass 3: Documentation Standardization ⏳
**Status**: Not Started  
**Target**: Consolidate all documentation per Space rules

#### Sub-tasks:
- [ ] Move android/ARCHITECTURE.md → docs/architecture/android.md
- [ ] Move android/CHANGELOG.md → docs/changelogs/android.md
- [ ] Move android/TODO_IMPLEMENTATION.md → docs/todo/android-implementation.md
- [ ] Move android/README.md content → docs/platforms/android.md (keep minimal README)
- [ ] Create docs/naming-migration.md for translation reference
- [ ] Add /// doc comments to all public functions
- [ ] Remove obvious inline comments, keep only obscure logic comments
- [ ] Ensure docs/ folder is primary documentation location

### Pass 4: Naming Convention Migration ⏳
**Status**: Not Started  
**Target**: Translate all Swedish/mixed naming to English

#### Translation Map:
**Modules**:
- `adresser.rs` → `addresses.rs`
- `paneler.rs` → `panels.rs`
- `topbar.rs` → `top_bar.rs`

**Structs/Types**:
- `StoredAddress.gata` → `street`
- `StoredAddress.gatunummer` → `street_number`
- `StoredAddress.postnummer` → `postal_code`

**Functions**:
- All function names already in snake_case (verify English words)

**Constants**:
- Verify SCREAMING_SNAKE_CASE

#### Sub-tasks:
- [ ] Create complete translation reference in docs/naming-migration.md
- [ ] Rename modules (adresser → addresses)
- [ ] Update struct field names
- [ ] Update function names if needed
- [ ] Update all references and imports
- [ ] Run cargo check after each batch

### Pass 5: Android → iOS Migration ⏳
**Status**: Not Started  
**Target**: Sync android/ code to iOS/ with platform-specific modules

#### Sub-tasks:
- [ ] Create iOS directory structure (if missing)
- [ ] Copy android/src/ui/ → ios/src/ui/ (post-refactor)
- [ ] Copy android/src/matching.rs → ios/src/
- [ ] Copy android/src/countdown.rs → ios/src/
- [ ] Copy android/src/static_data.rs → ios/src/
- [ ] Copy android/src/main.rs → ios/src/ (adapt as needed)
- [ ] Create ios/src/components/ with iOS-specific implementations:
  - [ ] ios/src/components/gps.rs (iOS GPS API)
  - [ ] ios/src/components/storage.rs (iOS storage API)
  - [ ] ios/src/components/notification.rs (iOS notification API)
- [ ] Copy & adapt android/Cargo.toml → ios/Cargo.toml
- [ ] Update workspace Cargo.toml to include ios member
- [ ] Ensure both platforms share maximum code through core/

### Pass 6: Dioxus 0.7.3 Migration ⏳
**Status**: Not Started  
**Target**: Implement proper routing and signal patterns

#### Sub-tasks:
- [ ] Define Route enum with #[derive(Routable)]
- [ ] Implement Router in App component
- [ ] Replace any deprecated patterns (verify none exist)
- [ ] Ensure all state uses use_signal (NOT use_state)
- [ ] Add navigation between routes
- [ ] Test routing in both android/ and iOS/

### Pass 7: Script Organization ⏳
**Status**: Not Started  
**Target**: Move all shell scripts to scripts/ folder

#### Sub-tasks:
- [ ] Move build.sh → scripts/build.sh
- [ ] Move serve.sh → scripts/serve.sh
- [ ] Move adb-install.sh → scripts/adb-install.sh
- [ ] Update all scripts to use dynamic path resolution:
  ```bash
  REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  cd "$REPO_ROOT"
  ```
- [ ] Create scripts/README.md documenting each script
- [ ] Test each script after moving
- [ ] Update any documentation referencing old paths

### Pass 8: Dependency Cleanup ⏳
**Status**: Not Started  
**Target**: Clean all Cargo.toml files

#### Sub-tasks:
- [ ] Audit workspace Cargo.toml dependencies
- [ ] Comment out unused workspace dependencies
- [ ] Audit android/Cargo.toml
- [ ] Comment out unused android dependencies
- [ ] Audit ios/Cargo.toml
- [ ] Comment out unused ios dependencies
- [ ] Audit core/Cargo.toml
- [ ] Comment out unused core dependencies
- [ ] Audit server/Cargo.toml
- [ ] Comment out unused server dependencies
- [ ] Run cargo check --all-features to verify

### Pass 9: Testing & Validation ✅
**Status**: Not Started  
**Target**: Verify all changes compile and follow best practices

#### Sub-tasks:
- [ ] Run `cargo check --features android`
- [ ] Run `cargo check --features ios`
- [ ] Run `./scripts/fmt_fix_clippy.sh`
- [ ] Fix all clippy warnings
- [ ] Verify no compilation errors
- [ ] Verify proper error handling
- [ ] Verify accessibility (where applicable)
- [ ] Final documentation review

## Statistics

### Initial Analysis (2026-01-30)
- **Total Rust files**: TBD (scan in progress)
- **Empty files identified**: 3 (file.rs, geo.rs, notification.rs)
- **Swedish-named modules**: 3 (adresser, paneler, topbar)
- **Shell scripts in root**: 3 (build.sh, serve.sh, adb-install.sh)

### Post-Refactoring (TBD)
- **Files changed**: TBD
- **Functions refactored**: TBD
- **Deprecated items moved**: TBD
- **Documentation files created**: TBD
- **Average function length**: TBD
- **Lines of code**: TBD

## Next Steps

1. Complete comprehensive codebase scan
2. Begin Pass 1: Function refactoring
3. Create naming-migration.md reference
4. Execute passes sequentially with validation
5. Document findings and decisions

## Notes & Decisions

### 2026-01-30: Initial Setup
- Created refactor branch: `refactor/comprehensive-2026-01`
- Base SHA: `3ad83fe21c96a8dd94a3ff6321a49f95f4377623`
- Identified scripts/ folder already exists
- fmt_fix_clippy.sh already in root (not scripts/), needs to move
- Both android/ and iOS/ folders exist but iOS needs population

### Space Instructions Compliance
- ✅ Documentation going to docs/ folder (this file)
- ✅ Following Rust naming conventions
- ✅ Using Dioxus 0.7.3 patterns
- ✅ Breaking functions into small, testable units

---

*This document will be updated after each pass with detailed statistics and findings.*
