// tests/validator_tests.rs
//
// Run `cargo test generate_broken_epubs` first, so target/broken_epubs/ exists.
// Then:  cargo test --test validator_tests
//
// Adjust the `use` lines below to wherever your crate exports these.
use crate::validator::{
    verify_ebook, ChapterError, CoverError, ManifestError, NavError, PackageError, SpineError,
    ValidationError as V, ZipError,
};

const OPF: &str = "OEBPS/content.opf";

fn check(book: &str) -> Result<(), V> {
    let path = format!("./target/broken_epubs/{book}.epub");
    assert!(
        std::path::Path::new(&path).exists(),
        "{path} missing: run `cargo test generate_broken_epubs` first"
    );
    verify_ebook(&path, OPF).map(|_| ())
}

type Expect = fn(&Result<(), V>) -> bool;

/// Runs every case and reports all mismatches at once.
fn run(cases: &[(&str, Expect)]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(book, expect)| {
            let got = check(book);
            (!expect(&got)).then(|| format!("{book}: got {got:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

// ------------------------------------------------------------------ the happy path

#[test]
fn valid_book_passes() {
    let r = check("valid");
    assert!(r.is_ok(), "{r:?}");
}

// ------------------------------------------------------------------ package

#[test]
fn package_errors() {
    run(&[
        ("o1_opf_malformed",     |r| matches!(r, Err(V::Package(PackageError::MalformedFile(_))))),
        ("o1_root_not_package",  |r| matches!(r, Err(V::Package(PackageError::MalformedFile(_))))),
    ]);
}

#[test]
fn missing_opf_path() {
    let r = verify_ebook("./target/broken_epubs/valid.epub", "OEBPS/does_not_exist.opf");
    assert!(matches!(r, Err(V::Package(PackageError::MissingOpf(_)))));
}

#[test]
fn missing_zip_file() {
    let r = verify_ebook("./target/broken_epubs/no_such_file.epub", OPF);
    assert!(matches!(r, Err(V::Zip(ZipError::ZipFileMissing))));
}

// ------------------------------------------------------------------ manifest

#[test]
fn manifest_errors() {
    run(&[
        ("o3_no_manifest",   |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::EmptyManifest))))),
        ("m1_duplicate_id",  |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::DuplicateId(_)))))),
    ]);
}

// ------------------------------------------------------------------ spine

#[test]
fn spine_errors() {
    run(&[
        ("o4_no_spine",        |r| matches!(r, Err(V::Package(PackageError::Spine(SpineError::EmptySpine))))),
        ("s1_empty_spine",     |r| matches!(r, Err(V::Package(PackageError::Spine(SpineError::EmptySpine))))),
        ("s4_duplicate_idref", |r| matches!(r, Err(V::Package(PackageError::Spine(SpineError::DuplicateIdref(_)))))),
        ("s2_dangling_idref",  |r| matches!(r, Err(V::Chapter(ChapterError::SpineReferenceNotFound(_))))),
    ]);
}

// ------------------------------------------------------------------ cover

#[test]
fn cover_errors() {
    run(&[
        ("v1_cover_id_missing",  |r| matches!(r, Err(V::Cover(CoverError::MissingId)))),
        ("v4_no_cover",          |r| matches!(r, Err(V::Cover(CoverError::MissingId)))),
        ("v1_cover_wrong_type",  |r| matches!(r, Err(V::Cover(CoverError::UnsupportedMediaType(_))))),
        ("v5_cover_undecodable", |r| matches!(r, Err(V::Cover(CoverError::InvalidImage(_))))),
        ("v5_cover_too_small",   |r| matches!(r, Err(V::Cover(CoverError::InvalidImage(_))))),
    ]);
}

// ------------------------------------------------------------------ nav

#[test]
fn nav_errors() {
    run(&[
        ("n1_nav_id_missing",    |r| matches!(r, Err(V::Nav(NavError::MissingId)))),
        ("n1_nav_wrong_media_type", |r| matches!(r, Err(V::Nav(NavError::UnsupportedMediaType(_))))),
        ("n4_span_heading",      |r| matches!(r, Err(V::Nav(NavError::MissingHref)))),
        ("n6_empty_label",       |r| matches!(r, Err(V::Nav(NavError::EmptyLabel)))),
        ("n7_no_entries",        |r| matches!(r, Err(V::Nav(NavError::EmptyToc)))),
    ]);
}

// ------------------------------------------------------------------ chapters

#[test]
fn chapter_errors() {
    run(&[
        ("x1_malformed_xml",   |r| matches!(r, Err(V::Chapter(ChapterError::MalformedFile(_))))),
        ("x1_latin1_not_utf8", |r| matches!(r, Err(V::Chapter(ChapterError::MalformedFile(_))))),
    ]);
}

// ------------------------------------------------------------------ everything at once

#[test]
fn all_issues_book_fails() {
    // Which error comes first depends on the validator's check order; we only require failure.
    assert!(check("all_issues").is_err());
}

// ------------------------------------------------------------------ out of scope

/// These violate the profile, but belong to the fixer or to the zip-safety step,
/// not to verify_ebook. If the validator starts checking one, move it to a test above.
#[test]
fn out_of_scope_books_are_not_the_validators_business() {
    run(&[
        ("c2_two_rootfiles",     |r| r.is_ok()),
        ("c2_wrong_media_type",  |r| r.is_ok()),
        ("m6_file_not_in_manifest", |r| r.is_ok()),
        ("x4_script",            |r| r.is_ok()),
        ("x5_event_handler",     |r| r.is_ok()),
        ("y2_import",            |r| r.is_ok()),
    ]);
}