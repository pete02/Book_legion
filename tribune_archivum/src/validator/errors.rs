use thiserror::Error;


#[derive(Debug, Error)]
pub enum ValidationError{
    #[error("Zip error:")]
    Zip(#[from]ZipError),

    #[error("Package fetchinfg failed: {0}")]
    Package(#[from] PackageError),

    #[error("Cover validation failed: {0}")]
    Cover(#[from] CoverError),


    #[error("Nav validation failed: {0}")]
    Nav(#[from] NavError),

    #[error("Chapter validation failed: {0}")]
    Chapter(#[from] ChapterError)
}
#[derive(Debug, Error)]
pub enum ZipError{
    #[error("No zip file found")]
    ZipFileMissing,

    #[error("Internal error: {0}")]
    Internal(Box<dyn std::error::Error>)
}

#[derive(Debug, Error)]
pub enum PackageError{
    #[error("Missing opf file in path {0}")]
    MissingOpf(String),
    #[error ("Malformed file at {0}")]
    MalformedFile(String),

    #[error("Manifest validation failed: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Spine validation failed: {0}")]
    Spine(#[from] SpineError),
}


#[derive(Debug, Error)]
pub enum ManifestError {
    #[error ("Manifest is missing item {0}")]
    MissingItem(String),

    #[error ("Manifest did not contain any items")]
    EmptyManifest,

    #[error ("Manifest contains duplicate ids: {0}")]
    DuplicateId(String)
}
#[derive(Debug, Error)]
pub enum SpineError{
    #[error ("Missing idref")]
    MissingIdref,
    #[error ("Spine did not contain any items")]
    EmptySpine,


    #[error ("Spine contains duplicate ids: {0}")]
    DuplicateIdref(String)
}

#[derive(Debug, Error)]
pub enum CoverError {
    #[error("Manifest contains no item with id 'cover'")]
    MissingId,

    #[error("Path escape detected: {0}")]
    PathEscape(String),

    #[error("Cover file has unsupported media type '{0}'")]
    UnsupportedMediaType(String),

    #[error("Cover file not found in '{0}'")]
    MissingFile(String),
     
    #[error("Could not read cover file in '{0}'")]
    ReadError(String),

    #[error("Cover file is not a valid supported image: {0}")]
    InvalidImage(String),
}



#[derive(Debug, Error)]
pub enum NavError {
    #[error("Manifest contains no item with id 'nav'")]
    MissingId,

    #[error("Path escape detected: {0}")]
    PathEscape(String),

    #[error("Nav file has unsupported media type '{0}'")]
    UnsupportedMediaType(String),

    #[error("Nav file not found in '{0}'")]
    MissingFile(String),

    #[error("Could not read nav file in '{0}'")]
    ReadError(String),

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
}


#[derive(Debug, Error)]
pub enum ChapterError {
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
