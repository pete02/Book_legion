use serde::{Deserialize, Serialize};

/// Severity level of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Error causes rejection of the EPUB
    Error,
    /// Warning does not cause rejection
    Warning,
}

/// Category of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCategory {
    Archive,
    Container,
    Opf,
    Spine,
    Manifest,
    Navigation,
    Content,
    Cover,
    Consistency,
}

/// Error codes for validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    MissingManifest,
    
    // Spine errors
    EmptySpine,
    MissingItemRef,
    
    // Manifest errors
    DuplicateManifestId,
    InvalidManifestId,
    MissingMediaType,
    
    // Navigation errors
    MissingNavDocument,
    InvalidNavXml,
    InvalidNcxXml,
    MissingTocNav,
    EmptyToc,
    InvalidTocEntry,
    
    // Content errors
    MissingChapterFile,
    UnparsableHtml,
    
    // Cover errors
    MissingCover,
    
    // Consistency errors
    SpineManifestMismatch,
    ManifestArchiveMismatch,
    NavSpineMismatch,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl ErrorCode {
    pub fn category(&self) -> ValidationCategory {
        match self {
            ErrorCode::InvalidZip
            | ErrorCode::UnreadableEntry
            | ErrorCode::DuplicatePath => ValidationCategory::Archive,
            
            ErrorCode::MissingContainerXml
            | ErrorCode::InvalidContainerXml
            | ErrorCode::MissingOpfReference
            | ErrorCode::InvalidOpfReference => ValidationCategory::Container,
            
            ErrorCode::MissingOpf
            | ErrorCode::InvalidOpfXml
            | ErrorCode::MissingSpine
            | ErrorCode::MissingManifest => ValidationCategory::Opf,
            
            ErrorCode::EmptySpine | ErrorCode::MissingItemRef => ValidationCategory::Spine,
            
            ErrorCode::DuplicateManifestId
            | ErrorCode::InvalidManifestId
            | ErrorCode::MissingMediaType => ValidationCategory::Manifest,
            
            ErrorCode::MissingNavDocument
            | ErrorCode::InvalidNavXml
            | ErrorCode::InvalidNcxXml
            | ErrorCode::MissingTocNav
            | ErrorCode::EmptyToc
            | ErrorCode::InvalidTocEntry => ValidationCategory::Navigation,
            
            ErrorCode::MissingChapterFile | ErrorCode::UnparsableHtml => ValidationCategory::Content,
            
            ErrorCode::MissingCover => ValidationCategory::Cover,
            
            ErrorCode::SpineManifestMismatch
            | ErrorCode::ManifestArchiveMismatch
            | ErrorCode::NavSpineMismatch => ValidationCategory::Consistency,
        }
    }
    
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::InvalidZip => "Not a valid ZIP archive",
            ErrorCode::UnreadableEntry => "Cannot read archive entry",
            ErrorCode::DuplicatePath => "Duplicate archive path",
            ErrorCode::MissingContainerXml => "META-INF/container.xml not found",
            ErrorCode::InvalidContainerXml => "container.xml is not valid XML",
            ErrorCode::MissingOpfReference => "No valid OPF reference in container.xml",
            ErrorCode::InvalidOpfReference => "OPF file not found",
            ErrorCode::MissingOpf => "OPF file cannot be read",
            ErrorCode::InvalidOpfXml => "OPF is not valid XML",
            ErrorCode::MissingSpine => "OPF missing <spine> element",
            ErrorCode::MissingManifest => "OPF missing <manifest> element",
            ErrorCode::EmptySpine => "Spine has no linear items",
            ErrorCode::MissingItemRef => "Item reference missing from manifest",
            ErrorCode::DuplicateManifestId => "Duplicate manifest ID",
            ErrorCode::InvalidManifestId => "Invalid manifest ID",
            ErrorCode::MissingMediaType => "Missing media-type attribute",
            ErrorCode::MissingNavDocument => "No EPUB3 nav or EPUB2 NCX found",
            ErrorCode::InvalidNavXml => "Nav document is not valid HTML",
            ErrorCode::InvalidNcxXml => "NCX document is not valid XML",
            ErrorCode::MissingTocNav => "No <nav epub:type=\"toc\"> found",
            ErrorCode::EmptyToc => "TOC has no usable entries",
            ErrorCode::InvalidTocEntry => "TOC entry has empty href",
            ErrorCode::MissingChapterFile => "Chapter file not found in archive",
            ErrorCode::UnparsableHtml => "Chapter HTML cannot be parsed",
            ErrorCode::MissingCover => "No cover image found",
            ErrorCode::SpineManifestMismatch => "Spine itemref not found in manifest",
            ErrorCode::ManifestArchiveMismatch => "Manifest item not found in archive",
            ErrorCode::NavSpineMismatch => "Navigation target not in spine",
        }
    }
}

/// Location of a validation issue within the EPUB
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLocation {
    Root,
    ArchiveEntry { path: String },
    Opf { path: String },
    Spine,
    Manifest,
    Navigation { path: String },
    Ncx { path: String },
    Chapter { path: String },
    Cover { path: String },
    TocEntry { index: usize },
}

impl std::fmt::Display for ValidationLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLocation::Root => write!(f, "root"),
            ValidationLocation::ArchiveEntry { path } => write!(f, "{}", path),
            ValidationLocation::Opf { path } => write!(f, "OPF ({})", path),
            ValidationLocation::Spine => write!(f, "spine"),
            ValidationLocation::Manifest => write!(f, "manifest"),
            ValidationLocation::Navigation { path } => write!(f, "navigation ({})", path),
            ValidationLocation::Ncx { path } => write!(f, "NCX ({})", path),
            ValidationLocation::Chapter { path } => write!(f, "chapter ({})", path),
            ValidationLocation::Cover { path } => write!(f, "cover ({})", path),
            ValidationLocation::TocEntry { index } => write!(f, "TOC entry #{}", index),
        }
    }
}

/// A single validation error or warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub location: ValidationLocation,
    pub severity: Severity,
}

impl ValidationError {
    pub fn new(code: ErrorCode, location: ValidationLocation) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            location,
            severity: Severity::Error,
        }
    }
    
    pub fn warning(code: ErrorCode, location: ValidationLocation) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            location,
            severity: Severity::Warning,
        }
    }
}

/// Information about the EPUB archive structure
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub entries: Vec<String>,
    pub duplicate_paths: Vec<String>,
}

/// Information about the container.xml and OPF document
#[derive(Debug, Clone)]
pub struct OpfInfo {
    pub opf_path: String,
    pub opf_data: Vec<u8>,
}

/// Information about the OPF package document
#[derive(Debug, Clone)]
pub struct OpfPackage {
    pub path: String,
    pub root_file_path: String,
    pub metadata: serde_json::Value,
    pub manifest_items: Vec<ManifestItem>,
    pub spine_items: Vec<SpineItem>,
}

/// A manifest item from the OPF
#[derive(Debug, Clone)]
pub struct ManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub properties: Vec<String>,
}

/// A spine item (itemref) from the OPF
#[derive(Debug, Clone)]
pub struct SpineItem {
    pub idref: String,
    pub linear: String,
    pub properties: Vec<String>,
}

/// Information about the spine
#[derive(Debug, Clone)]
pub struct SpineInfo {
    pub items: Vec<SpineItem>,
    pub linear_items: Vec<SpineItem>,
}

/// Information about the manifest
#[derive(Debug, Clone)]
pub struct ManifestInfo {
    pub items: Vec<ManifestItem>,
    pub id_to_item: std::collections::HashMap<String, ManifestItem>,
}

/// Information about navigation documents
#[derive(Debug, Clone)]
pub struct NavInfo {
    pub nav_path: Option<String>,
    pub ncx_path: Option<String>,
    pub toc_entries: Vec<TocEntry>,
}

/// A table of contents entry
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub href: String,
    pub children: Vec<TocEntry>,
}
