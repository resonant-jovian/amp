# Naming Migration Reference

## Purpose

This document tracks the translation of Swedish and mixed-language identifiers to English throughout the refactoring process. All naming follows Rust conventions:

- **snake_case**: functions, variables, module files
- **PascalCase**: types, structs, enums, traits
- **SCREAMING_SNAKE_CASE**: constants, statics

## Module Files

### android/src/ui/

| Original (Swedish) | English Translation | File Path |
|--------------------|--------------------|-----------|
| `adresser.rs` | `addresses.rs` | `android/src/ui/addresses.rs` |
| `paneler.rs` | `panels.rs` | `android/src/ui/panels.rs` |
| `topbar.rs` | `top_bar.rs` | `android/src/ui/top_bar.rs` |

### iOS/src/ui/ (mirrors android after migration)

Same naming applies to iOS modules.

## Struct Fields

### StoredAddress

Defined in: `android/src/ui/mod.rs` (later `android/src/ui/mod.rs` or extracted)

| Original (Swedish) | English Translation | Type | Description |
|--------------------|--------------------| -----|-------------|
| `gata` | `street` | `String` | Street name (e.g., "Storgatan") |
| `gatunummer` | `street_number` | `String` | Street number (e.g., "10") |
| `postnummer` | `postal_code` | `String` | Postal code (e.g., "22100") |

**Example Before**:
```rust
pub struct StoredAddress {
    pub id: usize,
    pub gata: String,
    pub gatunummer: String,
    pub postnummer: String,
    pub valid: bool,
    pub active: bool,
    pub matched_entry: Option<StaticAddressEntry>,
}
```

**Example After**:
```rust
pub struct StoredAddress {
    pub id: usize,
    pub street: String,
    pub street_number: String,
    pub postal_code: String,
    pub valid: bool,
    pub active: bool,
    pub matched_entry: Option<StaticAddressEntry>,
}
```

### StaticAddressEntry

Defined in: `android/src/static_data.rs`

*Needs inspection to determine field names and translations*

## Function Parameters

### Address Matching Functions

Defined in: `android/src/matching.rs` and `android/src/ui/mod.rs`

| Original (Swedish) | English Translation |
|--------------------|--------------------|
| `gata: &str` | `street: &str` |
| `gatunummer: &str` | `street_number: &str` |
| `postnummer: &str` | `postal_code: &str` |

**Functions Affected**:
- `fuzzy_match_address(gata, gatunummer, postnummer)` → `fuzzy_match_address(street, street_number, postal_code)`
- `match_address(gata, gatunummer, postnummer)` → `match_address(street, street_number, postal_code)`
- `StoredAddress::new(gata, gatunummer, postnummer)` → `StoredAddress::new(street, street_number, postal_code)`

## Component Names

### UI Components

Defined in: `android/src/ui/`

| Original (Swedish) | English Translation | Component Type |
|--------------------|--------------------|-----------------|
| `Adresser` | `Addresses` | Component displaying address list |
| `TopBar` | `TopBar` | ✅ Already English |

### Panel Components

Defined in: `android/src/ui/paneler.rs` → `android/src/ui/panels.rs`

*Needs inspection for component names inside paneler.rs*

## Variable Names

### Common Variable Translations

| Original (Swedish) | English Translation | Context |
|--------------------|--------------------|---------|
| `addrs` | `addrs` | ✅ Acceptable abbreviation (or expand to `addresses`) |
| `stored_addresses` | `stored_addresses` | ✅ Already English |

## Constants & Statics

### Examples Found

| Name | Status | Location |
|------|--------|----------|
| `CSS` | ✅ Already English | `android/src/ui/mod.rs` |
| `ADDRESS_ID_COUNTER` | ✅ Already English | `android/src/ui/mod.rs` |

## Enums & Types

### MatchResult

Defined in: `android/src/matching.rs`

*Needs inspection for variant names*

## Translation Verification Checklist

After renaming, verify all references are updated:

- [ ] Module imports (`use crate::ui::addresses::*`)
- [ ] Component references in RSX
- [ ] Function calls with parameter names
- [ ] Struct instantiation
- [ ] Field access (`address.street` not `address.gata`)
- [ ] Documentation strings
- [ ] Log messages (`info!`, `warn!`, etc.)
- [ ] Test cases (if any)

## Swedish→English Dictionary (Context-Specific)

For future reference when encountering Swedish terms:

| Swedish | English | Context |
|---------|---------|----------|
| adress | address | Postal address |
| adresser | addresses | Plural of address |
| gata | street | Street name |
| gatunummer | street_number | House/building number |
| postnummer | postal_code | ZIP/postal code |
| panel | panel | ✅ Same in English |
| paneler | panels | Plural of panel |
| topbar | top_bar | UI navigation bar |
| matcha | match | To match/compare |
| giltig | valid | Validation status |
| aktiv | active | Active state |

## Implementation Strategy

### Phase 1: Module Renames
1. Rename files (adresser.rs → addresses.rs)
2. Update `mod.rs` declarations
3. Update imports across codebase
4. Run `cargo check`

### Phase 2: Struct Field Renames
1. Update struct definitions
2. Update all field access sites
3. Update struct instantiation
4. Update pattern matching
5. Run `cargo check`

### Phase 3: Function Parameter Renames
1. Update function signatures
2. Update function bodies using parameters
3. Update all call sites
4. Run `cargo check`

### Phase 4: Component & Type Renames
1. Update component names in files
2. Update RSX invocations
3. Update imports
4. Run `cargo check`

### Phase 5: Validation
1. Full `cargo check --all-features`
2. Run `./scripts/fmt_fix_clippy.sh`
3. Fix any remaining references
4. Update documentation
5. Commit changes

## Notes

- Preserve git history by renaming files in separate commits
- Use search & replace carefully with whole-word matching
- Test incrementally after each batch of changes
- Document any ambiguous translations for review

---

*This document will be updated as new Swedish/mixed-language terms are discovered during refactoring.*
