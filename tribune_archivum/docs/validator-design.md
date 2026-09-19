# Tribune Logistica EPUB Validator Design Document

## Overview

This document specifies the design for an EPUB validator that determines whether an EPUB file satisfies the Tribune Logistica contract. The validator must be deterministic, produce structured diagnostics, and distinguish between errors (which cause rejection) and warnings (which do not).

## Architecture

### Component Structure

```
validator/
├── core/
│   ├── types.rs          # Error/warning types, validation result
│   ├── checker.rs        # Validation orchestration
│   └── result.rs         # Validation outcome aggregation
├── archive/
│   ├── reader.rs         # ZIP archive access
│   └── integrity.rs      # Archive structure validation
├── epub/
│   ├── container.rs      # container.xml parsing & validation
│   ├── opf.rs            # OPF package document parsing & validation
│   ├── spine.rs          # Spine validation
│   ├── manifest.rs       # Manifest validation
│   ├── nav.rs            # Navigation document validation
│   └── ncx.rs            # NCX document validation
├── content/
│   ├── chapter.rs        # Chapter document validation
│   ├── cover.rs          # Cover image validation
│   └── css.rs            # CSS file validation
├── consistency/
│   ├── spine_manifest.rs # Spine → Manifest consistency
│   ├── manifest_archive.rs # Manifest → Archive consistency
│   └── nav_spine.rs      # Navigation → Spine consistency
└── main.rs               # CLI entry point
```

## Data Types

### Validation Result

```rust
#[derive(Debug, Clone)]
pub enum Severity {
    Error,   // Causes rejection
    Warning, // Does not cause rejection
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub category: ValidationCategory,
    pub code: ErrorCode,
    pub location: ValidationLocation,
    pub reference: Option<String>,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum ValidationCategory {
    Archive,
    Container,
    Opf,
    Spine,
    Manifest,
    Navigation,
    Content,
    Cover,
    Css,
    Consistency,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    // Archive errors
    InvalidZip,
    UnreadableEntry,
    DuplicatePath,
    
    // Container errors
    MissingContainerXml,
    InvalidContainerXml,
    MissingOpfReference,
    InvalidOpfReference,
    
    // OPF errors
    MissingOpf,
    InvalidOpfXml,
    MissingSpine,
    EmptySpine,
    MissingManifest,
    
    // Spine errors
    MissingManifestItem,
    MissingSpineFile,
    InvalidLinearValue,
    
    // Manifest errors
    DuplicateManifestId,
    MissingManifestMediaType,
    InvalidManifestId,
    
    // Navigation errors
    MissingNavDocument,
    MissingNcxDocument,
    InvalidNavXml,
    InvalidNcxXml,
    MissingTocNav,
    EmptyToc,
    InvalidTocEntry,
    
    // Content errors
    MissingChapterFile,
    UnparseableHtml,
    InvalidChapterPath,
    
    // Cover errors
    MissingCover,
    InvalidCoverFormat,
    UnreadableCover,
    
    // CSS errors
    MissingCss,
    UnreadableCss,
    
    // Consistency errors
    SpineManifestMismatch,
    ManifestArchiveMismatch,
    NavSpineMismatch,
}

#[derive(Debug, Clone)]
pub enum ValidationLocation {
    Root,
    Path(String),           // e.g., "META-INF/container.xml"
    OpfElement { file: String, element: String }, // e.g., {"content.opf", "spine/itemref"}
    NavElement { file: String, element: String },
    NcxElement { file: String, element: String },
    ManifestItem { id: String },
    SpineItem { index: usize },
    TocEntry { index: usize },
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub passed: bool,
}
```

## Validation Phases

The validator executes in six sequential phases. Each phase must complete before the next begins.

### Phase 1: Archive Integrity

**Goal**: Verify the EPUB is a valid, readable ZIP archive.

**Checks**:

1. **ZIP validity**
   - File is a valid ZIP archive
   - Central directory is present and valid
   - No corrupted entries

2. **Entry accessibility**
   - All entries can be read
   - No truncated entries

3. **Path uniqueness**
   - No duplicate archive paths (case-sensitive)
   - Report duplicates as errors

**Implementation**:

```rust
pub fn validate_archive(path: &str) -> Result<ArchiveInfo, ValidationError> {
    let mut entries: HashMap<String, usize> = HashMap::new();
    let mut errors: Vec<ValidationError> = Vec::new();
    
    let mut archive = match ZipArchive::open(path) {
        Ok(archive) => archive,
        Err(e) => {
            return Err(ValidationError {
                category: ValidationCategory::Archive,
                code: ErrorCode::InvalidZip,
                location: ValidationLocation::Root,
                reference: None,
                message: format!("Failed to open ZIP archive: {}", e),
                severity: Severity::Error,
            });
        }
    };
    
    // Check for duplicates
    for i in 0..archive.len() {
        let entry = match archive.by_index(i) {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(ValidationError {
                    category: ValidationCategory::Archive,
                    code: ErrorCode::UnreadableEntry,
                    location: ValidationLocation::Root,
                    reference: None,
                    message: format!("Failed to read entry {}: {}", i, e),
                    severity: Severity::Error,
                });
                continue;
            }
        };
        
        let name = entry.name().to_string();
        *entries.entry(name).or_insert(0) += 1;
    }
    
    // Report duplicates
    for (name, count) in entries {
        if count > 1 {
            errors.push(ValidationError {
                category: ValidationCategory::Archive,
                code: ErrorCode::DuplicatePath,
                location: ValidationLocation::Path(name.clone()),
                reference: Some(format!("{} occurrences", count)),
                message: format!("Duplicate archive path: {}", name),
                severity: Severity::Error,
            });
        }
    }
    
    Ok(ArchiveInfo {
        entries,
        errors,
    })
}
```

### Phase 2: Container and OPF Discovery

**Goal**: Locate and validate the container.xml and OPF document.

**Checks**:

1. **container.xml existence**
   - `META-INF/container.xml` must exist
   - Must be valid XML

2. **OPF reference validation**
   - Must contain at least one `<rootfile>` element
   - Must have `media-type="application/oebps-package+xml"` OR path ending in `.opf`
   - Referenced OPF path must exist in archive

**Implementation**:

```rust
pub fn validate_container(archive: &ZipArchive, archive_info: &ArchiveInfo) -> Result<OpfInfo, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    
    // Check container.xml exists
    let container_data = match archive.by_name("META-INF/container.xml") {
        Ok(data) => data,
        Err(_) => {
            errors.push(ValidationError {
                category: ValidationCategory::Container,
                code: ErrorCode::MissingContainerXml,
                location: ValidationLocation::Path("META-INF/container.xml".to_string()),
                reference: None,
                message: "Required file META-INF/container.xml not found".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Parse container.xml
    let container: Container = match xml::parse(&container_data) {
        Ok(container) => container,
        Err(e) => {
            errors.push(ValidationError {
                category: ValidationCategory::Container,
                code: ErrorCode::InvalidContainerXml,
                location: ValidationLocation::Path("META-INF/container.xml".to_string()),
                reference: None,
                message: format!("Failed to parse container.xml: {}", e),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Find OPF path
    let opf_path = container.rootfiles
        .iter()
        .find(|rf| {
            rf.media_type == "application/oebps-package+xml" || 
            rf.full_path.ends_with(".opf")
        })
        .map(|rf| rf.full_path.clone());
    
    let opf_path = match opf_path {
        Some(path) => path,
        None => {
            errors.push(ValidationError {
                category: ValidationCategory::Container,
                code: ErrorCode::MissingOpfReference,
                location: ValidationLocation::Path("META-INF/container.xml".to_string()),
                reference: None,
                message: "No valid OPF reference found in container.xml".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Check OPF exists
    if !archive_info.entries.contains_key(&opf_path) {
        errors.push(ValidationError {
            category: ValidationCategory::Container,
            code: ErrorCode::InvalidOpfReference,
            location: ValidationLocation::Path("META-INF/container.xml".to_string()),
            reference: Some(opf_path.clone()),
            message: format!("OPF file referenced in container.xml not found: {}", opf_path),
            severity: Severity::Error,
        });
        return Err(errors);
    }
    
    Ok(OpfInfo {
        path: opf_path,
        errors,
    })
}
```

### Phase 3: OPF and Spine Validation

**Goal**: Validate the OPF document structure and spine.

**Checks**:

1. **OPF structure**
   - Valid XML
   - Contains `<spine>` element
   - Contains `<manifest>` element

2. **Spine validation**
   - Must have at least one `<itemref>`
   - Must have at least one linear item (linear != "no")
   - Each `<itemref>` must reference a manifest item

**Implementation**:

```rust
pub fn validate_spine(opf_data: &[u8], opf_path: &str, archive: &ZipArchive) -> Result<SpineInfo, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    
    // Parse OPF
    let opf: OpfPackage = match xml::parse(opf_data) {
        Ok(opf) => opf,
        Err(e) => {
            errors.push(ValidationError {
                category: ValidationCategory::Opf,
                code: ErrorCode::InvalidOpfXml,
                location: ValidationLocation::OpfElement {
                    file: opf_path.to_string(),
                    element: "root".to_string(),
                },
                reference: None,
                message: format!("Failed to parse OPF: {}", e),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Check spine exists
    let spine = match &opf.spine {
        Some(spine) => spine,
        None => {
            errors.push(ValidationError {
                category: ValidationCategory::Opf,
                code: ErrorCode::MissingSpine,
                location: ValidationLocation::OpfElement {
                    file: opf_path.to_string(),
                    element: "spine".to_string(),
                },
                reference: None,
                message: "OPF document missing <spine> element".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Check spine has items
    if spine.itemrefs.is_empty() {
        errors.push(ValidationError {
            category: ValidationCategory::Spine,
            code: ErrorCode::EmptySpine,
            location: ValidationLocation::OpfElement {
                file: opf_path.to_string(),
                element: "spine".to_string(),
            },
            reference: None,
            message: "Spine contains no itemref elements".to_string(),
            severity: Severity::Error,
        });
        return Err(errors);
    }
    
    // Validate each itemref
    let mut linear_count = 0;
    let mut spine_items = Vec::new();
    
    for (index, itemref) in spine.itemrefs.iter().enumerate() {
        // Check linear value
        let is_linear = itemref.linear != Some("no".to_string());
        if is_linear {
            linear_count += 1;
        }
        
        spine_items.push(SpineItemInfo {
            index,
            idref: itemref.idref.clone(),
            is_linear,
        });
    }
    
    // Check at least one linear item
    if linear_count == 0 {
        errors.push(ValidationError {
            category: ValidationCategory::Spine,
            code: ErrorCode::EmptySpine,
            location: ValidationLocation::OpfElement {
                file: opf_path.to_string(),
                element: "spine".to_string(),
            },
            reference: None,
            message: "Spine contains no linear items".to_string(),
            severity: Severity::Error,
        });
        return Err(errors);
    }
    
    Ok(SpineInfo {
        items: spine_items,
        linear_count,
        errors,
    })
}
```

### Phase 4: Manifest Validation

**Goal**: Validate the manifest structure and uniqueness.

**Checks**:

1. **Manifest structure**
   - Must exist in OPF
   - Must have at least one item

2. **Manifest ID uniqueness**
   - No duplicate IDs (case-sensitive)
   - IDs must be valid (non-empty, no spaces)

3. **Media type presence**
   - All items should have media-type (though not strictly required)

**Implementation**:

```rust
pub fn validate_manifest(opf: &OpfPackage, opf_path: &str) -> Result<ManifestInfo, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    
    let manifest = match &opf.manifest {
        Some(manifest) => manifest,
        None => {
            errors.push(ValidationError {
                category: ValidationCategory::Opf,
                code: ErrorCode::MissingManifest,
                location: ValidationLocation::OpfElement {
                    file: opf_path.to_string(),
                    element: "manifest".to_string(),
                },
                reference: None,
                message: "OPF document missing <manifest> element".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Check for duplicate IDs
    let mut id_counts: HashMap<String, usize> = HashMap::new();
    for item in &manifest.items {
        *id_counts.entry(item.id.clone()).or_insert(0) += 1;
    }
    
    for (id, count) in &id_counts {
        if *count > 1 {
            errors.push(ValidationError {
                category: ValidationCategory::Manifest,
                code: ErrorCode::DuplicateManifestId,
                location: ValidationLocation::ManifestItem { id: id.clone() },
                reference: Some(format!("{} occurrences", count)),
                message: format!("Duplicate manifest ID: {}", id),
                severity: Severity::Error,
            });
        }
    }
    
    // Check for empty IDs
    for item in &manifest.items {
        if item.id.is_empty() {
            errors.push(ValidationError {
                category: ValidationCategory::Manifest,
                code: ErrorCode::InvalidManifestId,
                location: ValidationLocation::ManifestItem { id: "<empty>".to_string() },
                reference: None,
                message: "Manifest item has empty ID".to_string(),
                severity: Severity::Error,
            });
        }
    }
    
    Ok(ManifestInfo {
        items: manifest.items.clone(),
        id_counts,
        errors,
    })
}
```

### Phase 5: Navigation Validation

**Goal**: Validate navigation document existence and structure.

**Checks**:

1. **Navigation document discovery**
   - Try EPUB3 nav first (manifest item with `properties="nav"`)
   - Fall back to EPUB2 NCX (manifest item with `media-type="application/x-dtbncx+xml"` or spine toc reference)

2. **EPUB3 nav structure**
   - Must contain `<nav epub:type="toc">`
   - Must contain `<ol>` within nav
   - Must contain `<li>` elements with `<a>` tags
   - Each `<a>` must have valid `href`

3. **EPUB2 NCX structure**
   - Must contain `<navMap>` elements
   - Must contain `<navPoint>` elements
   - Each `<navPoint>` must have `<navLabel><text>` and `<content>`

4. **TOC entry validity**
   - Each entry must have non-empty href (fragment-only links are skipped)
   - At least one valid TOC entry required

**Implementation**:

```rust
pub fn validate_navigation(
    opf: &OpfPackage, 
    opf_path: &str, 
    archive: &ZipArchive,
    manifest_info: &ManifestInfo
) -> Result<NavInfo, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let opf_dir = path::Path::new(opf_path).parent().unwrap();
    
    // Try EPUB3 nav first
    let nav_path = manifest_info.items.iter()
        .find(|item| {
            item.properties.as_ref()
                .map(|props| props.contains("nav"))
                .unwrap_or(false)
        })
        .map(|item| opf_dir.join(&item.href));
    
    let nav_info = if let Some(path) = nav_path {
        let path_str = path.to_string_lossy().to_string();
        validate_epub3_nav(&path_str, archive, &manifest_info.id_counts)?
    } else {
        // Try EPUB2 NCX
        let ncx_path = find_ncx_path(opf, opf_dir, &manifest_info.id_counts);
        match ncx_path {
            Some(path) => {
                let path_str = path.to_string_lossy().to_string();
                validate_epub2_ncx(&path_str, archive)?
            }
            None => {
                errors.push(ValidationError {
                    category: ValidationCategory::Navigation,
                    code: ErrorCode::MissingNavDocument,
                    location: ValidationLocation::Root,
                    reference: None,
                    message: "No navigation document found (no EPUB3 nav, no EPUB2 NCX)".to_string(),
                    severity: Severity::Error,
                });
                return Err(errors);
            }
        }
    };
    
    Ok(nav_info)
}

fn validate_epub3_nav(
    nav_path: &str,
    archive: &ZipArchive,
    id_counts: &HashMap<String, usize>
) -> Result<NavInfo, Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    
    // Read nav document
    let nav_data = match archive.by_name(nav_path) {
        Ok(data) => data,
        Err(_) => {
            errors.push(ValidationError {
                category: ValidationCategory::Navigation,
                code: ErrorCode::MissingNavDocument,
                location: ValidationLocation::Path(nav_path.to_string()),
                reference: None,
                message: format!("Navigation document not found: {}", nav_path),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Parse HTML
    let doc = match html::parse(&nav_data) {
        Ok(doc) => doc,
        Err(e) => {
            errors.push(ValidationError {
                category: ValidationCategory::Navigation,
                code: ErrorCode::InvalidNavXml,
                location: ValidationLocation::NavElement {
                    file: nav_path.to_string(),
                    element: "root".to_string(),
                },
                reference: None,
                message: format!("Failed to parse nav document: {}", e),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Find nav with epub:type="toc"
    let toc_nav = find_toc_nav(&doc);
    let toc_nav = match toc_nav {
        Some(nav) => nav,
        None => {
            errors.push(ValidationError {
                category: ValidationCategory::Navigation,
                code: ErrorCode::MissingTocNav,
                location: ValidationLocation::NavElement {
                    file: nav_path.to_string(),
                    element: "nav".to_string(),
                },
                reference: None,
                message: "No <nav> element with epub:type=\"toc\" found".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Find <ol> within nav
    let toc_ol = find_descendant(toc_nav, "ol");
    let toc_ol = match toc_ol {
        Some(ol) => ol,
        None => {
            errors.push(ValidationError {
                category: ValidationCategory::Navigation,
                code: ErrorCode::MissingTocNav,
                location: ValidationLocation::NavElement {
                    file: nav_path.to_string(),
                    element: "nav/ol".to_string(),
                },
                reference: None,
                message: "<nav> element contains no <ol>".to_string(),
                severity: Severity::Error,
            });
            return Err(errors);
        }
    };
    
    // Extract TOC entries
    let mut toc_entries = Vec::new();
    extract_toc_entries(toc_ol, &mut toc_entries, nav_path);
    
    // Check for empty TOC
    if toc_entries.is_empty() {
        errors.push(ValidationError {
            category: ValidationCategory::Navigation,
            code: ErrorCode::EmptyToc,
            location: ValidationLocation::NavElement {
                file: nav_path.to_string(),
                element: "nav/ol/li/a".to_string(),
            },
            reference: None,
            message: "TOC contains no usable entries".to_string(),
            severity: Severity::Error,
        });
        return Err(errors);
    }
    
    Ok(NavInfo {
        path: nav_path.to_string(),
        entries: toc_entries,
        errors,
    })
}
```

### Phase 6: Cross-Structure Consistency

**Goal**: Verify consistency between spine, manifest, archive, and navigation.

**Checks**:

1. **Spine → Manifest**
   - Every linear spine `idref` must have exactly one matching manifest item
   - Report duplicate manifest IDs as errors

2. **Manifest → Archive**
   - Every manifest item's `href` must exist in the archive
   - Path resolution must be valid (no path traversal)

3. **Navigation → Spine**
   - Every navigation target must resolve to a linear spine document
   - Navigation entries pointing to non-linear documents are allowed but warned

**Implementation**:

```rust
pub fn validate_consistency(
    spine_info: &SpineInfo,
    manifest_info: &ManifestInfo,
    archive: &ZipArchive,
    nav_info: &NavInfo,
    opf_path: &str
) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let opf_dir = path::Path::new(opf_path).parent().unwrap();
    
    // Build manifest lookup
    let manifest_lookup: HashMap<&str, &ManifestItem> = manifest_info.items
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    
    // Check spine → manifest consistency
    for spine_item in &spine_info.items {
        if !spine_item.is_linear {
            continue;
        }
        
        let manifest_item = match manifest_lookup.get(spine_item.idref.as_str()) {
            Some(item) => item,
            None => {
                errors.push(ValidationError {
                    category: ValidationCategory::Consistency,
                    code: ErrorCode::SpineManifestMismatch,
                    location: ValidationLocation::SpineItem { index: spine_item.index },
                    reference: Some(spine_item.idref.clone()),
                    message: format!(
                        "Linear spine itemref references manifest ID '{}' which does not exist",
                        spine_item.idref
                    ),
                    severity: Severity::Error,
                });
                continue;
            }
        };
        
        // Check manifest → archive consistency
        let file_path = opf_dir.join(&manifest_item.href);
        let file_path_str = file_path.to_string_lossy().to_string();
        
        if !archive.by_name(&file_path_str).is_ok() {
            errors.push(ValidationError {
                category: ValidationCategory::Consistency,
                code: ErrorCode::ManifestArchiveMismatch,
                location: ValidationLocation::ManifestItem { id: manifest_item.id.clone() },
                reference: Some(file_path_str.clone()),
                message: format!(
                    "Manifest item '{}' references file not found in archive: {}",
                    manifest_item.id, file_path_str
                ),
                severity: Severity::Error,
            });
        }
    }
    
    // Check navigation → spine consistency
    let linear_spine_paths: HashSet<String> = spine_info.items
        .iter()
        .filter(|item| item.is_linear)
        .map(|item| {
            let manifest_item = manifest_lookup.get(item.idref.as_str()).unwrap();
            opf_dir.join(&manifest_item.href).to_string_lossy().to_string()
        })
        .collect();
    
    for (index, entry) in nav_info.entries.iter().enumerate() {
        let nav_path = path::Path::new(&nav_info.path).parent().unwrap()
            .join(&entry.href);
        let nav_path_str = nav_path.to_string_lossy().to_string();
        
        if !linear_spine_paths.contains(&nav_path_str) {
            errors.push(ValidationError {
                category: ValidationCategory::Consistency,
                code: ErrorCode::NavSpineMismatch,
                location: ValidationLocation::TocEntry { index },
                reference: Some(entry.href.clone()),
                message: format!(
                    "Navigation target '{}' is not part of the linear spine",
                    entry.href
                ),
                severity: Severity::Warning,
            });
        }
    }
    
    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
}
```

## Cover Validation (Optional)

**Goal**: Verify cover image availability if required.

**Checks**:

1. **Cover discovery** (in priority order):
   - EPUB3: manifest item with `properties="cover-image"`
   - EPUB2: `<meta name="cover" content="manifest-id">`
   - Guide: `<guide><reference type="cover" href="...">`

2. **Cover file validation**:
   - File must exist in archive
   - File must be readable
   - File must be a supported format (PNG, JPEG, GIF, SVG, WebP)

**Classification**: APPLICATION POLICY (currently required by Book Legion)

## CSS Validation (Optional)

**Goal**: Verify CSS file availability.

**Checks**:

1. **CSS file discovery**:
   - Find all files with `.css` extension in archive

2. **CSS file validation**:
   - All CSS files must be readable

**Classification**: NOT REQUIRED (no CSS EPUBs are valid)

## Validation Execution

### Main Validation Flow

```rust
pub fn validate_epub(path: &str) -> ValidationResult {
    let mut all_errors: Vec<ValidationError> = Vec::new();
    
    // Phase 1: Archive integrity
    let archive_info = match validate_archive(path) {
        Ok(info) => info,
        Err(e) => {
            return ValidationResult {
                errors: vec![e],
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Open archive for subsequent phases
    let archive = match ZipArchive::open(path) {
        Ok(archive) => archive,
        Err(_) => {
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Phase 2: Container and OPF discovery
    let opf_info = match validate_container(&archive, &archive_info) {
        Ok(info) => info,
        Err(errors) => {
            all_errors.extend(errors);
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Read OPF data
    let opf_data = match archive.by_name(&opf_info.path) {
        Ok(data) => data,
        Err(_) => {
            all_errors.push(ValidationError {
                category: ValidationCategory::Opf,
                code: ErrorCode::MissingOpf,
                location: ValidationLocation::Path(opf_info.path.clone()),
                reference: None,
                message: format!("OPF file not found: {}", opf_info.path),
                severity: Severity::Error,
            });
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Phase 3: OPF and spine validation
    let spine_info = match validate_spine(&opf_data, &opf_info.path, &archive) {
        Ok(info) => info,
        Err(errors) => {
            all_errors.extend(errors);
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Parse OPF for manifest and navigation
    let opf: OpfPackage = match xml::parse(&opf_data) {
        Ok(opf) => opf,
        Err(_) => {
            // Errors already reported in validate_spine
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Phase 4: Manifest validation
    let manifest_info = match validate_manifest(&opf, &opf_info.path) {
        Ok(info) => info,
        Err(errors) => {
            all_errors.extend(errors);
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Phase 5: Navigation validation
    let nav_info = match validate_navigation(&opf, &opf_info.path, &archive, &manifest_info) {
        Ok(info) => info,
        Err(errors) => {
            all_errors.extend(errors);
            return ValidationResult {
                errors: all_errors,
                warnings: Vec::new(),
                passed: false,
            };
        }
    };
    
    // Phase 6: Consistency validation
    if let Err(errors) = validate_consistency(
        &spine_info,
        &manifest_info,
        &archive,
        &nav_info,
        &opf_info.path,
    ) {
        all_errors.extend(errors);
    }
    
    // Optional: Cover validation (APPLICATION POLICY)
    if let Err(errors) = validate_cover(&opf, &opf_info.path, &archive) {
        all_errors.extend(errors);
    }
    
    // Separate errors and warnings
    let (errors, warnings): (Vec<_>, Vec<_>) = all_errors
        .into_iter()
        .partition(|e| e.severity == Severity::Error);
    
    ValidationResult {
        errors,
        warnings,
        passed: errors.is_empty(),
    }
}
```

## CLI Interface

```rust
#[derive(Parser)]
#[command(name = "epub-validator")]
#[command(about = "Validates EPUB files against the Tribune Logistica contract")]
struct Args {
    /// Path to EPUB file
    #[arg(short, long)]
    input: String,
    
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
    
    /// Include warnings in output
    #[arg(short, long, default_value = "false")]
    include_warnings: bool,
    
    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    let result = validate_epub(&args.input);
    
    match args.format {
        OutputFormat::Text => {
            print_text_output(&result, args.include_warnings);
        }
        OutputFormat::Json => {
            print_json_output(&result, args.include_warnings);
        }
    }
}

fn print_text_output(result: &ValidationResult, include_warnings: bool) {
    if result.passed {
        println!("✓ Validation PASSED");
        if !result.warnings.is_empty() && include_warnings {
            println!("\nWarnings:");
            for warning in &result.warnings {
                print_warning(warning);
            }
        }
    } else {
        println!("✗ Validation FAILED");
        println!("\nErrors:");
        for error in &result.errors {
            print_error(error);
        }
        if include_warnings && !result.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &result.warnings {
                print_warning(warning);
            }
        }
    }
}

fn print_error(error: &ValidationError) {
    let location = match &error.location {
        ValidationLocation::Root => "root".to_string(),
        ValidationLocation::Path(p) => p.clone(),
        ValidationLocation::OpfElement { file, element } => {
            format!("{} ({})", file, element)
        }
        ValidationLocation::NavElement { file, element } => {
            format!("{} ({})", file, element)
        }
        ValidationLocation::NcxElement { file, element } => {
            format!("{} ({})", file, element)
        }
        ValidationLocation::ManifestItem { id } => {
            format!("manifest ID: {}", id)
        }
        ValidationLocation::SpineItem { index } => {
            format!("spine item #{}", index)
        }
        ValidationLocation::TocEntry { index } => {
            format!("TOC entry #{}", index)
        }
    };
    
    println!("  [{}] {} at {}", error.code, error.message, location);
}
```

## Error Codes Reference

| Code | Category | Severity | Description |
|------|----------|----------|-------------|
| INVALID_ZIP | Archive | Error | Not a valid ZIP archive |
| UNREADABLE_ENTRY | Archive | Error | Cannot read archive entry |
| DUPLICATE_PATH | Archive | Error | Duplicate archive path |
| MISSING_CONTAINER_XML | Container | Error | META-INF/container.xml not found |
| INVALID_CONTAINER_XML | Container | Error | container.xml is not valid XML |
| MISSING_OPF_REFERENCE | Container | Error | No valid OPF reference in container.xml |
| INVALID_OPF_REFERENCE | Container | Error | OPF file not found |
| MISSING_OPF | Opf | Error | OPF file cannot be read |
| INVALID_OPF_XML | Opf | Error | OPF is not valid XML |
| MISSING_SPINE | Opf | Error | OPF missing <spine> element |
| EMPTY_SPINE | Spine | Error | Spine has no linear items |
| MISSING_MANIFEST | Opf | Error | OPF missing <manifest> element |
| DUPLICATE_MANIFEST_ID | Manifest | Error | Duplicate manifest ID |
| INVALID_MANIFEST_ID | Manifest | Error | Empty manifest ID |
| MISSING_NAV_DOCUMENT | Navigation | Error | No EPUB3 nav or EPUB2 NCX found |
| INVALID_NAV_XML | Navigation | Error | Nav document is not valid HTML |
| INVALID_NCX_XML | Navigation | Error | NCX document is not valid XML |
| MISSING_TOC_NAV | Navigation | Error | No <nav epub:type="toc"> found |
| EMPTY_TOC | Navigation | Error | TOC has no usable entries |
| INVALID_TOC_ENTRY | Navigation | Error | TOC entry has empty href |
| MISSING_CHAPTER_FILE | Content | Error | Chapter file not found in archive |
| UNPARSABLE_HTML | Content | Error | Chapter HTML cannot be parsed |
| MISSING_COVER | Cover | Error | No cover image found |
| INVALID_COVER_FORMAT | Cover | Error | Cover is not a supported format |
| MISSING_CSS | Css | Error | No CSS files found |
| SPINE_MANIFEST_MISMATCH | Consistency | Error | Spine references missing manifest ID |
| MANIFEST_ARCHIVE_MISMATCH | Consistency | Error | Manifest references missing file |
| NAV_SPINE_MISMATCH | Consistency | Warning | Navigation points to non-linear document |

## Test Corpus

The validator must be tested against a comprehensive test corpus:

```
fixtures/
├── valid/
│   ├── epub-001.epub          # Minimal valid EPUB3
│   ├── epub-002.epub          # Minimal valid EPUB2
│   ├── epub-003.epub          # EPUB with cover
│   ├── epub-004.epub          # EPUB with CSS
│   └── ...
├── invalid/
│   ├── not-a-zip.epub         # Not a ZIP archive
│   ├── missing-container.epub # No container.xml
│   ├── missing-opf.epub       # No OPF reference
│   ├── empty-spine.epub       # Spine with no linear items
│   ├── missing-manifest-id.epub # Spine references missing manifest ID
│   ├── missing-nav.epub       # No navigation document
│   ├── empty-toc.epub         # TOC with no entries
│   ├── duplicate-path.epub    # Duplicate archive paths
│   ├── duplicate-id.epub      # Duplicate manifest IDs
│   └── ...
└── edge-cases/
    ├── fragment-only-toc.epub # TOC with only fragment links
    ├── non-linear-nav.epub    # Navigation to non-linear docs
    ├── malformed-html.epub    # Malformed but parseable HTML
    └── ...
```

## Implementation Priority

1. **Phase 1-2** (Archive + Container): Critical path, must work first
2. **Phase 3-4** (OPF + Manifest): Core structure validation
3. **Phase 5** (Navigation): Essential for reading
4. **Phase 6** (Consistency): Ensures all parts work together
5. **Cover + CSS**: Optional, can be added later

## Success Criteria

The validator is complete when:

- ✓ All six validation phases are implemented
- ✓ All error codes are defined and used appropriately
- ✓ CLI interface works with text and JSON output
- ✓ Test corpus passes/fails as expected
- ✓ No false positives (valid EPUBs pass)
- ✓ No false negatives (invalid EPUBs fail)
- ✓ Warnings are correctly distinguished from errors
- ✓ Validation is deterministic (same input → same output)
