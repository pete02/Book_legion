use thiserror::Error;

/// Everything `verify_ebook` can fail with.
///
/// Each variant wraps the layer that found the problem, so the fixable/fatal split
/// is not per variant here: [`ValidationError::is_fixable`] delegates to the wrapped
/// error. Variants stay in check order.
#[derive(Debug, Error)]
pub enum ValidationError{
    #[error("Zip error: {0}")]
    Zip(#[from]ZipError),

    #[error("Package fetching failed: {0}")]
    Package(#[from] PackageError),

    #[error("Cover validation failed: {0}")]
    Cover(#[from] CoverError),

    #[error("Nav validation failed: {0}")]
    Nav(#[from] NavError),

    #[error("Chapter validation failed: {0}")]
    Chapter(#[from] ChapterError)
}

impl ValidationError {
    /// `true` when a fixer pass can repair the book, `false` when the book must be
    /// rejected outright.
    ///
    /// Mirrors the FIX/STRIP and REJECT outcomes in `docs/validator-requiements.md`.
    pub fn is_fixable(&self) -> bool {
        match self {
            ValidationError::Zip(e) => e.is_fixable(),
            ValidationError::Package(e) => e.is_fixable(),
            ValidationError::Cover(e) => e.is_fixable(),
            ValidationError::Nav(e) => e.is_fixable(),
            ValidationError::Chapter(e) => e.is_fixable(),
        }
    }
}
#[derive(Debug, Error)]
pub enum ZipError{
    // fatal: Z1, Z3 and Z4 are REJECT. A broken, hostile or oversized archive is
    // repacked from the original upload, never repaired in place, and a missing
    // file cannot be invented.
    #[error("No zip file found")]
    ZipFileMissing,

    #[error("Internal error: {0}")]
    Internal(Box<dyn std::error::Error>),

    #[error("Zip file exceeds size limits")]
    ZipFileSizeError,

    #[error("too many files")]
    ZipFileAmountError,

    #[error("File too large: {0}")]
    ZipFileTooLarge(String)
}

impl ZipError {
    pub fn is_fixable(&self) -> bool {
        match self {
            ZipError::ZipFileMissing
            | ZipError::Internal(_)
            | ZipError::ZipFileSizeError
            | ZipError::ZipFileAmountError
            | ZipError::ZipFileTooLarge(_) => false,
        }
    }
}

#[derive(Debug, Error)]
pub enum PackageError{
    // decided by the layer below
    #[error("Manifest validation failed: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Spine validation failed: {0}")]
    Spine(#[from] SpineError),

    // fatal: O1/C3 REJECT. Without a readable package document there is no
    // manifest or spine left to repair.
    #[error("Missing opf file in path {0}")]
    MissingOpf(String),

    #[error ("Malformed file at {0}")]
    MalformedFile(String),
}

impl PackageError {
    pub fn is_fixable(&self) -> bool {
        match self {
            PackageError::Manifest(e) => e.is_fixable(),
            PackageError::Spine(e) => e.is_fixable(),
            PackageError::MissingOpf(_) | PackageError::MalformedFile(_) => false,
        }
    }
}


#[derive(Debug, Error)]
pub enum ManifestError {
    // fixable: M1 generates missing ids and drops items with no href, O3/M1 merge
    // duplicate ids into a single manifest.
    #[error ("Manifest is missing item {0}")]
    MissingItem(String),

    #[error ("Manifest contains duplicate ids: {0}")]
    DuplicateId(String),

    // fatal: O3 refuses any epub without a manifest.
    #[error ("Manifest did not contain any items")]
    EmptyManifest,
}

impl ManifestError {
    pub fn is_fixable(&self) -> bool {
        match self {
            ManifestError::MissingItem(_) | ManifestError::DuplicateId(_) => true,
            ManifestError::EmptyManifest => false,
        }
    }
}
#[derive(Debug, Error)]
pub enum SpineError{
    // fixable: S2 drops an itemref that points at nothing, S4 keeps the first of a
    // duplicate. An itemref with no idref attribute at all is dropped the same way.
    #[error ("Missing idref")]
    MissingIdref,

    #[error ("Spine contains duplicate ids: {0}")]
    DuplicateIdref(String),

    // fatal: S1 REJECT. An empty spine means the book has no chapters at all.
    #[error ("Spine did not contain any items")]
    EmptySpine,
}

impl SpineError {
    pub fn is_fixable(&self) -> bool {
        match self {
            SpineError::MissingIdref | SpineError::DuplicateIdref(_) => true,
            SpineError::EmptySpine => false,
        }
    }
}

#[derive(Debug, Error)]
pub enum CoverError {
    // fixable: V1/V4 rename or generate a placeholder cover, M2/Z2 normalize the
    // path, M4/V1 re-sniff the media type, M3 drop the item, V5 re-encode or replace.
    #[error("Manifest contains no item with id 'cover'")]
    MissingId,

    #[error("Path escape detected: {0}")]
    PathEscape(String),

    #[error("Cover file has unsupported media type '{0}'")]
    UnsupportedMediaType(String),

    #[error("Cover file not found in '{0}'")]
    MissingFile(String),

    #[error("Cover file is not a valid supported image: {0}")]
    InvalidImage(String),

    // fatal: the entry is there but its bytes cannot be read, so the fixer can
    // sniff nothing and re-encode nothing. Never constructed at the moment.
    #[error("Could not read cover file in '{0}'")]
    ReadError(String),
}

impl CoverError {
    pub fn is_fixable(&self) -> bool {
        match self {
            CoverError::MissingId
            | CoverError::PathEscape(_)
            | CoverError::UnsupportedMediaType(_)
            | CoverError::MissingFile(_)
            | CoverError::InvalidImage(_) => true,
            CoverError::ReadError(_) => false,
        }
    }
}



#[derive(Debug, Error)]
pub enum NavError {
    // fixable: N3-N8 rebuild the toc, N5 drops entries that point nowhere, N6
    // derives labels, N7 synthesizes a toc from the spine.
    #[error("Path escape detected: {0}")]
    PathEscape(String),

    #[error("Nav file not found in '{0}'")]
    MissingFile(String),

    #[error("Nav file is malformed XML: '{0}'")]
    MalformedFile(String),

    #[error("Nav file contains no TOC navigation element")]
    MissingToc,

    #[error("TOC entry has no link with an href")]
    MissingHref,

    #[error("TOC entry has an empty label")]
    EmptyLabel,

    #[error("Invalid navigation href: '{0}'")]
    InvalidHref(String),

    #[error("Navigation target is not in the manifest: '{0}'")]
    TargetNotInManifest(String),

    #[error("TOC contains no entries")]
    EmptyToc,

    // fatal: N1 REJECT. The nav item is absent from the manifest or is not
    // xhtml, so there is no nav document to rewrite.
    #[error("Manifest contains no item with id 'nav'")]
    MissingId,

    #[error("Nav file has unsupported media type '{0}'")]
    UnsupportedMediaType(String),
}

impl NavError {
    pub fn is_fixable(&self) -> bool {
        match self {
            NavError::PathEscape(_)
            | NavError::MissingFile(_)
            | NavError::MalformedFile(_)
            | NavError::MissingToc
            | NavError::MissingHref
            | NavError::EmptyLabel
            | NavError::InvalidHref(_)
            | NavError::TargetNotInManifest(_)
            | NavError::EmptyToc => true,
            NavError::MissingId | NavError::UnsupportedMediaType(_) => false,
        }
    }
}

/// Internal helper for "this zip entry could not be read". Every caller maps it
/// onto a layer error, so it never escapes as a `ValidationError` and is not
/// classified as fixable or fatal.
#[derive(Debug, Error)]
pub enum ReadFail{
    #[error("File could not be read")]
    Unreadable,
    #[error("File not utf8")]
    NotUtf8,
    #[error("File not found")]
    NotFound

}

#[derive(Debug, Error)]
pub enum ChapterError {
    // fixable: every chapter failure has a fixer move. S2 drops the dangling ref,
    // M4 sniffs or strips the item, M2/Z2 normalizes the path, M3 drops the item
    // and its spine refs, X1 transcodes or repairs the document.
    #[error ("Spine item not found in manifest: {0}")]
    SpineReferenceNotFound(String),

    #[error ("Spine item has unsupported type: {0}")]
    UnsupportedMediaType(String),

    #[error("Path escape detected: {0}")]
    PathEscape(String),

    #[error ("Missing file at {0}")]
    MissingFile(String),

    #[error ("Malformed file at {0}")]
    MalformedFile(String)
}

impl ChapterError {
    pub fn is_fixable(&self) -> bool {
        match self {
            ChapterError::SpineReferenceNotFound(_)
            | ChapterError::UnsupportedMediaType(_)
            | ChapterError::PathEscape(_)
            | ChapterError::MissingFile(_)
            | ChapterError::MalformedFile(_) => true,
        }
    }
}
