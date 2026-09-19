pub mod types;
pub mod result;

pub use types::{
    ArchiveInfo,
    ErrorCode,
    ManifestInfo,
    ManifestItem,
    NavInfo,
    OpfInfo,
    OpfPackage,
    Severity,
    SpineInfo,
    SpineItem,
    TocEntry,
    ValidationCategory,
    ValidationError,
    ValidationLocation,
};

pub use result::{ValidationResult, ValidationResultBuilder};
