pub mod types;
pub mod result;

#[cfg(test)]
mod types_test;

#[cfg(test)]
mod result_test;

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
