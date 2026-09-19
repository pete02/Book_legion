# Unit Testing Plan for Tribune Archivum EPUB Validator

## Overview

This document outlines a comprehensive unit testing strategy for the EPUB validator implementation in `tribune_archivum`. The testing plan follows the six-phase validation architecture and ensures thorough coverage of all validation modules.

## Testing Strategy

### Test Organization

Tests will be organized by module, mirroring the source code structure:

```
src/
├── core/
│   ├── types_test.rs
│   └── result_test.rs
├── archive/
│   ├── integrity_test.rs
│   └── reader_test.rs
├── epub/
│   ├── container_test.rs
│   ├── opf_test.rs
│   ├── spine_test.rs
│   ├── manifest_test.rs
│   └── navigation_test.rs
├── consistency/
│   └── mod_test.rs
└── validator_test.rs
```

### Test Categories

1. **Unit Tests**: Test individual functions in isolation
2. **Integration Tests**: Test module interactions
3. **Edge Case Tests**: Test boundary conditions and error scenarios
4. **Positive Tests**: Test valid EPUB files
5. **Negative Tests**: Test invalid EPUB files

## Test Modules

### 1. Core Module Tests (`src/core/*_test.rs`)

#### `types_test.rs`
- **ErrorCode**: Test all error codes map correctly to `ValidationCategory`
- **Severity**: Test `Error` and `Warning` variants
- **ValidationLocation**: Test all location variants (Root, ArchiveEntry, Opf, Spine, Manifest, Nav)
- **ValidationError**: Test construction, code/message/location/severity accessors

#### `result_test.rs`
- **ValidationResult**: Test empty result creation
- **add_error()**: Test error addition, count increment, errors vector population
- **add_warning()**: Test warning addition, count increment, warnings vector population
- **Builder Pattern**: Test `ValidationResultBuilder` functionality
- **Aggregation**: Test error/warning separation and counting

### 2. Archive Module Tests (`src/archive/*_test.rs`)

#### `integrity_test.rs`
- **Valid ZIP**: Test archive opening with valid EPUB files
- **Invalid ZIP**: Test error handling for non-ZIP files
- **Corrupted ZIP**: Test handling of partially corrupted archives
- **Duplicate Paths**: Test detection of duplicate entry paths
- **Unreadable Entries**: Test handling of unreadable archive entries
- **Empty Archive**: Test handling of empty ZIP files
- **ArchiveInfo**: Test entry collection and duplicate path tracking

#### `reader_test.rs`
- **read_entry()**: Test successful entry reading
- **read_entry()**: Test error handling for missing entries
- **entry_exists()**: Test true case for existing entries
- **entry_exists()**: Test false case for missing entries
- **find_entries()**: Test finding entries by prefix
- **find_entries()**: Test empty results for no matches

### 3. EPUB Module Tests (`src/epub/*_test.rs`)

#### `container_test.rs`
- **Valid container.xml**: Test parsing of standard container.xml
- **OPF Discovery**: Test correct OPF path extraction
- **Multiple OPFs**: Test handling of multiple rootfiles
- **Invalid container.xml**: Test XML parsing error handling
- **Missing container.xml**: Test error when container.xml is absent
- **OpfInfo**: Test OPF data structure population

#### `opf_test.rs`
- **Valid OPF**: Test parsing of EPUB2 and EPUB3 package documents
- **Metadata Extraction**: Test title, creator, publisher, date extraction
- **Manifest Parsing**: Test manifest item collection
- **Spine Parsing**: Test spine item collection
- **Toc Reference**: Test NCX and nav document reference extraction
- **Invalid OPF**: Test XML parsing error handling
- **Missing Required Fields**: Test handling of missing mandatory metadata
- **OpfPackage**: Test package document structure population

#### `spine_test.rs`
- **Linear Items**: Test identification of linear spine items
- **Non-Linear Items**: Test identification of non-linear spine items
- **ItemRef Validation**: Test idref validation against manifest
- **Empty Spine**: Test handling of spine with no items
- **Duplicate Idrefs**: Test duplicate idref detection
- **Missing Idrefs**: Test missing manifest item detection
- **SpineInfo**: Test linear/non-linear separation

#### `manifest_test.rs`
- **ID Uniqueness**: Test detection of duplicate manifest IDs
- **MIME Type Validation**: Test media type format validation
- **Full Path Validation**: Test full-path format validation
- **Empty Manifest**: Test handling of manifest with no items
- **ManifestInfo**: Test manifest item collection and ID tracking

#### `navigation_test.rs`
- **EPUB3 Nav**: Test parsing of standard nav document
- **EPUB2 NCX**: Test parsing of NCX document
- **Nav Document Missing**: Test error when nav/NCX absent
- **Invalid Nav XML**: Test XML parsing error handling
- **Invalid NCX XML**: Test XML parsing error handling
- **Nav Points**: Test navigation point extraction
- **NcxDocument**: Test NCX structure population
- **NavDocument**: Test nav structure population

### 4. Consistency Module Tests (`src/consistency/mod_test.rs`)

#### `mod_test.rs`
- **Spine-Manifest Consistency**: Test all spine itemrefs exist in manifest
- **Manifest-Archive Consistency**: Test all manifest items exist in archive
- **Nav-Spine Consistency**: Test nav points reference spine items
- **Cross-Reference Errors**: Test proper error categorization
- **Missing References**: Test detection of missing cross-references
- **ConsistencyInfo**: Test consistency information aggregation

### 5. Main Validator Tests (`src/validator_test.rs`)

#### `validator_test.rs`
- **Complete Valid EPUB**: Test full validation pipeline on valid EPUB
- **Complete Invalid EPUB**: Test full validation pipeline on invalid EPUB
- **Phase Orchestration**: Test all six phases execute in order
- **Error Aggregation**: Test errors from all phases are collected
- **Result Construction**: Test final ValidationResult construction
- **Partial Validation**: Test validation continues after errors
- **Performance**: Test validation completes in reasonable time

## Test Data Strategy

### Test Fixtures

Create test EPUB files in `test_assets/` directory:

```
test_assets/
├── valid/
│   ├── basic.epub
│   ├── epub2.epub
│   ├── epub3.epub
│   └── complex.epub
├── invalid/
│   ├── not_zip.epub
│   ├── corrupted.epub
│   ├── empty.epub
│   ├── no_container.epub
│   ├── invalid_container.epub
│   ├── no_opf.epub
│   ├── invalid_opf.epub
│   ├── missing_nav.epub
│   ├── invalid_nav.epub
│   ├── no_ncx.epub
│   ├── invalid_ncx.epub
│   ├── duplicate_ids.epub
│   ├── missing_spine_item.epub
│   ├── missing_manifest_item.epub
│   └── duplicate_paths.epub
└── edge_cases/
    ├── empty_manifest.epub
    ├── empty_spine.epub
    ├── single_item.epub
    └── large_epub.epub
```

### Test Data Generation

Use existing EPUB files from workspace:
- `/home/pete/code/rust/book_legion/data/books.json` - List of available books
- `/home/pete/code/rust/book_legion/data/binding/binding.epub`
- `/home/pete/code/rust/book_legion/data/bound/bound.epub`
- `/home/pete/code/rust/book_legion/data/citybound/citybound.epub`
- And other EPUB files in the data directory

## Test Implementation Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_case() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_output);
    }
    
    #[test]
    fn test_invalid_case() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.error_count(), 1);
    }
}
```

### Test Helpers

Create helper functions in `tests/helpers.rs`:

```rust
/// Creates a temporary EPUB file from bytes
pub fn create_temp_epub(bytes: &[u8]) -> tempfile::TempPath {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().join("test.epub");
    std::fs::write(&temp_path, bytes).unwrap();
    temp_path
}

/// Loads a test fixture from test_assets
pub fn load_fixture(fixture_name: &str) -> Vec<u8> {
    let path = format!("test_assets/{}", fixture_name);
    std::fs::read(&path).unwrap()
}

/// Creates a minimal valid EPUB for testing
pub fn create_minimal_epub() -> Vec<u8> {
    // Implementation to create minimal valid EPUB
}
```

### Test Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.0"
```

## Test Coverage Goals

### Module Coverage Targets

| Module | Target Coverage | Priority |
|--------|----------------|----------|
| `core/types.rs` | 100% | High |
| `core/result.rs` | 100% | High |
| `archive/integrity.rs` | 95% | High |
| `archive/reader.rs` | 95% | High |
| `epub/container.rs` | 95% | High |
| `epub/opf.rs` | 95% | High |
| `epub/spine.rs` | 90% | Medium |
| `epub/manifest.rs` | 90% | Medium |
| `epub/navigation.rs` | 90% | Medium |
| `consistency/mod.rs` | 90% | Medium |
| `validator.rs` | 85% | Medium |

### Critical Path Coverage

Ensure 100% coverage of:
- All error codes and their triggers
- All validation phases
- All error handling paths
- All public API functions

## Execution Plan

### Phase 1: Core Module Tests (Week 1)
- [ ] `types_test.rs` - Test all core types
- [ ] `result_test.rs` - Test ValidationResult and builder

### Phase 2: Archive Module Tests (Week 2)
- [ ] `integrity_test.rs` - Test ZIP validation
- [ ] `reader_test.rs` - Test archive entry reading

### Phase 3: EPUB Module Tests (Week 3-4)
- [ ] `container_test.rs` - Test container.xml parsing
- [ ] `opf_test.rs` - Test OPF parsing
- [ ] `spine_test.rs` - Test spine validation
- [ ] `manifest_test.rs` - Test manifest validation
- [ ] `navigation_test.rs` - Test nav/NCX validation

### Phase 4: Consistency Tests (Week 5)
- [ ] `mod_test.rs` - Test cross-structure validation

### Phase 5: Integration Tests (Week 6)
- [ ] `validator_test.rs` - Test full validation pipeline
- [ ] End-to-end tests with real EPUB files

### Phase 6: Test Data Preparation (Ongoing)
- [ ] Create test fixture EPUB files
- [ ] Document test data structure
- [ ] Add test data to version control

## Running Tests

### Basic Test Execution

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test --package tribune_archivum --lib archive::integrity::tests

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_valid_zip_archive
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html

# View coverage report
open target/cargo_tarpaulin/report.html
```

## Maintenance

### Test Updates

When adding new validation rules:
1. Update test fixtures if needed
2. Add new test cases for the rule
3. Update test coverage metrics
4. Document new test cases

### Test Data Updates

When EPUB specifications change:
1. Review existing test fixtures
2. Update fixtures to match new spec
3. Add new fixtures for new features
4. Remove obsolete fixtures

## Success Criteria

- [ ] All tests pass consistently
- [ ] Code coverage meets targets (>90% overall)
- [ ] All error codes have corresponding tests
- [ ] All validation phases have integration tests
- [ ] Test suite runs in <30 seconds
- [ ] No flaky tests (tests that fail intermittently)
- [ ] Test data is version controlled
- [ ] Documentation is up to date

## Notes

- Tests should be deterministic and reproducible
- Test data should be minimal but representative
- Error messages in tests should be clear and descriptive
- Tests should fail fast on errors
- Integration tests should use real EPUB files where possible
