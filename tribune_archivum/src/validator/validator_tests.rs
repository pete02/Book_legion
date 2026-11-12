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

#[test]
fn unreadable_zip() {
    run(&[
        ("z1_not_a_zip",     |r| matches!(r, Err(V::Zip(ZipError::Internal(_))))),
        ("z1_truncated_zip", |r| matches!(r, Err(V::Zip(ZipError::Internal(_))))),
    ]);
}

#[test]
fn zip_size_limits() {
    run(&[
        ("z4_compression_ratio", |r| matches!(r, Err(V::Zip(ZipError::ZipFileSizeError)))),
        ("z4_total_too_large",   |r| matches!(r, Err(V::Zip(ZipError::ZipFileSizeError)))),
        ("z4_entry_too_large",   |r| matches!(r, Err(V::Zip(ZipError::ZipFileTooLarge(_))))),
        ("z4_too_many_entries",  |r| matches!(r, Err(V::Zip(ZipError::ZipFileAmountError)))),
    ]);
}

// ------------------------------------------------------------------ manifest

#[test]
fn manifest_errors() {
    run(&[
        ("o3_no_manifest",   |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::EmptyManifest))))),
        ("m1_duplicate_id",  |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::DuplicateId(_)))))),
        ("m1_missing_href",  |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::MissingItem(_)))))),
        ("m4_media_type_missing", |r| matches!(r, Err(V::Package(PackageError::Manifest(ManifestError::MissingItem(_)))))),
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
        ("s1_missing_idref",   |r| matches!(r, Err(V::Package(PackageError::Spine(SpineError::MissingIdref))))),
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
        ("v1_cover_file_missing", |r| matches!(r, Err(V::Cover(CoverError::MissingFile(_))))),
        ("v1_cover_path_escape",  |r| matches!(r, Err(V::Cover(CoverError::PathEscape(_))))),
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
        ("n1_nav_file_missing",  |r| matches!(r, Err(V::Nav(NavError::MissingFile(_))))),
        ("n1_nav_malformed",     |r| matches!(r, Err(V::Nav(NavError::MalformedFile(_))))),
        ("n3_no_toc_nav",        |r| matches!(r, Err(V::Nav(NavError::MissingToc)))),
        ("n9_fragment_href",     |r| matches!(r, Err(V::Nav(NavError::InvalidHref(_))))),
        ("n9_escape_href",       |r| matches!(r, Err(V::Nav(NavError::PathEscape(_))))),
        ("n9_target_not_in_manifest", |r| matches!(r, Err(V::Nav(NavError::TargetNotInManifest(_))))),
    ]);
}

// ------------------------------------------------------------------ chapters

#[test]
fn chapter_errors() {
    run(&[
        ("x1_malformed_xml",   |r| matches!(r, Err(V::Chapter(ChapterError::MalformedFile(_))))),
        ("x1_latin1_not_utf8", |r| matches!(r, Err(V::Chapter(ChapterError::MalformedFile(_))))),
        ("x3_chapter_file_missing", |r| matches!(r, Err(V::Chapter(ChapterError::MissingFile(_))))),
        ("x3_chapter_wrong_type",   |r| matches!(r, Err(V::Chapter(ChapterError::UnsupportedMediaType(_))))),
        ("x3_chapter_escape",       |r| matches!(r, Err(V::Chapter(ChapterError::PathEscape(_))))),
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

// ------------------------------------------------------------------ classification

fn class(book: &str) -> Option<bool> {
    check(book).as_ref().err().map(|e| e.is_fixable())
}

/// Every error must answer whether a fixer pass could repair the book or the
/// upload must be rejected outright. See docs/validator-requiements.md.
#[test]
fn errors_classify_as_fixable_or_fatal() {
    let cases: &[(&str, Option<bool>)] = &[
        // Z1/Z3/Z4: broken, hostile or oversized archives get repacked, never repaired.
        ("z1_not_a_zip", Some(false)),
        ("z1_truncated_zip", Some(false)),
        ("z4_compression_ratio", Some(false)),
        ("z4_total_too_large", Some(false)),
        ("z4_entry_too_large", Some(false)),
        ("z4_too_many_entries", Some(false)),
        // O1/C3: without a readable package document nothing is left to repair.
        ("o1_opf_malformed", Some(false)),
        ("o1_root_not_package", Some(false)),
        // O3: an epub with no manifest is refused.
        ("o3_no_manifest", Some(false)),
        // S1: an empty spine means the book has no chapters at all.
        ("s1_empty_spine", Some(false)),
        // N1: no nav document to rewrite.
        ("n1_nav_id_missing", Some(false)),
        ("n1_nav_wrong_media_type", Some(false)),
        // M1/M4: ids get generated, items with no href get dropped, duplicates merge.
        ("m1_duplicate_id", Some(true)),
        ("m1_missing_href", Some(true)),
        ("m4_media_type_missing", Some(true)),
        // S2/S4: dangling and duplicate itemrefs get dropped.
        ("s4_duplicate_idref", Some(true)),
        ("s1_missing_idref", Some(true)),
        ("s2_dangling_idref", Some(true)),
        // V1/V4/V5/M3/M4: covers get named, re-typed, replaced or dropped.
        ("v1_cover_id_missing", Some(true)),
        ("v4_no_cover", Some(true)),
        ("v1_cover_wrong_type", Some(true)),
        ("v5_cover_undecodable", Some(true)),
        ("v5_cover_too_small", Some(true)),
        ("v1_cover_file_missing", Some(true)),
        ("v1_cover_path_escape", Some(true)),
        // N3-N8: the toc gets rebuilt, bad entries dropped, labels derived.
        ("n4_span_heading", Some(true)),
        ("n6_empty_label", Some(true)),
        ("n7_no_entries", Some(true)),
        ("n1_nav_file_missing", Some(true)),
        ("n1_nav_malformed", Some(true)),
        ("n3_no_toc_nav", Some(true)),
        ("n9_fragment_href", Some(true)),
        ("n9_escape_href", Some(true)),
        ("n9_target_not_in_manifest", Some(true)),
        // X1/M2/M3: chapters get transcoded, escaped items get their path normalized.
        ("x1_malformed_xml", Some(true)),
        ("x1_latin1_not_utf8", Some(true)),
        ("x3_chapter_file_missing", Some(true)),
        ("x3_chapter_wrong_type", Some(true)),
        ("x3_chapter_escape", Some(true)),
        // a clean book has no error to classify.
        ("valid", None),
    ];

    let mismatches: Vec<String> = cases
        .iter()
        .filter(|(book, want)| class(book) != *want)
        .map(|(book, _)| format!("{book}: got {:?}", class(book)))
        .collect();
    assert!(mismatches.is_empty(), "\n{}", mismatches.join("\n"));

    // the two failure kinds that need no fixture are fatal too.
    let missing = verify_ebook("./target/broken_epubs/no_such_file.epub", OPF);
    assert!(matches!(missing, Err(V::Zip(ZipError::ZipFileMissing))));
    assert!(!missing.unwrap_err().is_fixable());

    let wrong_opf = verify_ebook("./target/broken_epubs/valid.epub", "OEBPS/does_not_exist.opf");
    assert!(matches!(wrong_opf, Err(V::Package(PackageError::MissingOpf(_)))));
    assert!(!wrong_opf.unwrap_err().is_fixable());
}