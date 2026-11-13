// Tests for the fixer as it exists today: only the behaviour it actually implements.
// Requirements referenced by number from docs/validator-requiements.md §2.3.
//
// HANDLED TODAY
//   M1 (partly)  drop items that have no href
//                mint an id when the id *attribute* is absent
//   M4 (partly)  guess a missing media-type from the href extension
//
// STILL MISSING
//   M1   duplicate ids whose href differs are kept, so the id stays duplicate
//        id="" is never regenerated (only a missing attribute counts)
//   M2   nothing: no percent-decode, no fragment strip, no scheme or `//` rejection,
//        no case correction against the real entry name, no `..`-escape drop
//   M3   nothing: an item whose file is absent from the zip is kept
//   M4   a wrong or disallowed media-type is neither corrected nor stripped
//   M5   nothing: a remote href (http:, https:, //) is kept
//   M6   nothing: files absent from the manifest are not deleted, and
//        write_to_zip rewrites entries but cannot remove any
//   S2   spine pruning: dropping an item leaves its itemref behind, which the
//        validator reports as SpineReferenceNotFound
//
// Run `cargo test generate_broken_epubs` first, so target/broken_epubs/ exists.

use super::*;
use crate::validator::{
    errors::{PackageError, ValidationError as V},
    verify_ebook,
};
use std::fs::{self, File};
use std::path::Path;
use uuid::Uuid;

const OPF: &str = "OEBPS/content.opf";

// ---------------------------------------------------------------- fixtures

fn fixture(book: &str) -> String {
    let path = format!("./target/broken_epubs/{book}.epub");
    assert!(
        Path::new(&path).exists(),
        "{path} missing: run `cargo test generate_broken_epubs` first"
    );
    path
}

/// `fix_manifest` rewrites the zip in place, so every test works on its own copy.
fn copy(book: &str) -> String {
    fs::create_dir_all("./target/fixer_fixed").expect("scratch dir");
    let dst = format!("./target/fixer_fixed/{book}.epub");
    fs::copy(fixture(book), &dst).expect("copy fixture");
    dst
}

/// The errors `main` hands the fixer: whatever the validator reports for the original.
fn reported(book: &str) -> Vec<ManifestError> {
    match verify_ebook(&fixture(book), OPF) {
        Err(V::Package(PackageError::Manifest(e))) => vec![e],
        other => panic!("{book}: expected a manifest error, got {other:?}"),
    }
}

fn check(path: &str) -> Result<(), V> {
    verify_ebook(path, OPF).map(|_| ())
}

fn opf_text(path: &str) -> String {
    let mut archive = ZipArchive::new(File::open(path).expect("open copy")).expect("read zip");
    read_zip_text(&mut archive, OPF).expect("read opf")
}

// ---------------------------------------------------------------- end to end: fix -> validate

/// The two fixtures the fixer can actually repair, repaired.
#[test]
fn fixable_fixtures_validate_after_fixing() {
    for book in ["m1_missing_href", "m4_media_type_missing"] {
        let path = copy(book);
        let errors = reported(book);
        fix_manifest(&path, OPF, errors).unwrap_or_else(|e| panic!("{book}: fix failed: {e}"));
        check(&path).unwrap_or_else(|e| panic!("{book}: still invalid after fixing: {e:?}"));
    }
}

/// Requirement 6: `fix(fix(x)) == fix(x)`. A book that already validates must come
/// out of a second pass byte for byte.
#[test]
fn fixing_a_valid_book_changes_nothing() {
    for book in ["m1_missing_href", "m4_media_type_missing", "valid"] {
        let path = copy(book);
        if let Ok(errors) = std::panic::catch_unwind(|| reported(book)) {
            fix_manifest(&path, OPF, errors).expect("first fix");
        }
        let before = opf_text(&path);
        fix_manifest(&path, OPF, Vec::new()).expect("second fix");
        assert_eq!(before, opf_text(&path), "{book}: a fix pass changed a valid book");
    }
}

// ---------------------------------------------------------------- M1: duplicate id

/// The duplicate handler only removes later items that carry the *same* href as the
/// first one, because that is the only case where it can tell a copy from a chapter.
#[test]
fn duplicate_id_with_the_same_href_is_removed() {
    let opf = opf_with(&[
        "<item id=\"ch1\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/>",
        "<item id=\"css\" href=\"style.css\" media-type=\"text/css\"/>",
        "<item id=\"ch1\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/>",
    ]);
    let once = remove_duplicate_id_item("ch1", &opf, OPF, &mut archive()).expect("fix");
    let twice = remove_duplicate_id_item("ch1", &once, OPF, &mut archive()).expect("refix");

    assert_eq!(
        items(&twice),
        vec![item("ch1", "ch1.xhtml", "application/xhtml+xml"), item("css", "style.css", "text/css")]
    );
    assert_eq!(once, twice, "a second pass changed the manifest");
}

/// KNOWN GAP, see the M1 line at the top of this file. The two items below are two
/// different files sharing an id, and the handler keeps both. Flip this test into the
/// one above when M1 is finished.
#[test]
fn duplicate_id_with_a_different_href_is_still_kept() {
    let opf = opf_with(&[
        "<item id=\"ch1\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/>",
        "<item id=\"ch1\" href=\"style.css\" media-type=\"text/css\"/>",
    ]);
    let out = remove_duplicate_id_item("ch1", &opf, OPF, &mut archive()).expect("fix");
    assert_eq!(
        items(&out),
        vec![item("ch1", "ch1.xhtml", "application/xhtml+xml"), item("ch1", "style.css", "text/css")]
    );
}

// ---------------------------------------------------------------- M1: missing href, missing id

#[test]
fn item_without_href_is_dropped() {
    let opf = opf_with(&[
        "<item id=\"a\" href=\"a.css\" media-type=\"text/css\"/>",
        "<item id=\"b\" media-type=\"text/css\"/>",
    ]);
    let out = fix_missing_id_items("href", &opf, OPF, &mut archive()).expect("fix");
    assert_eq!(items(&out), vec![item("a", "a.css", "text/css")]);
}

#[test]
fn item_without_id_gets_a_generated_one() {
    let opf = opf_with(&["<item href=\"a.css\" media-type=\"text/css\"/>"]);
    let once = fix_missing_id_items("id", &opf, OPF, &mut archive()).expect("fix");

    let got = items(&once);
    assert_eq!(got.len(), 1);
    let id = Uuid::parse_str(&got[0].0).expect("a uuid id should have been minted");
    assert_eq!((got[0].1.as_str(), got[0].2.as_str()), ("a.css", "text/css"));

    // A second pass must keep the id it already gave the item.
    let twice = fix_missing_id_items("id", &once, OPF, &mut archive()).expect("refix");
    assert_eq!(items(&twice), vec![item(&id.to_string(), "a.css", "text/css")]);
}

// ---------------------------------------------------------------- M4: missing media-type

#[test]
fn missing_media_type_is_guessed_from_the_href() {
    let opf = opf_with(&[
        "<item id=\"a\" href=\"a.css\"/>",
        "<item id=\"p\" href=\"page.xhtml\"/>",
    ]);
    let once = fix_missing_id_items("media-type", &opf, OPF, &mut archive()).expect("fix");
    let twice = fix_missing_id_items("media-type", &once, OPF, &mut archive()).expect("refix");

    let want = vec![item("a", "a.css", "text/css"), item("p", "page.xhtml", "application/xhtml+xml")];
    assert_eq!(items(&once), want);
    assert_eq!(items(&twice), want, "a second pass must not re-guess the type");
}

// ---------------------------------------------------------------- helpers

/// The handlers take an archive they do not consult yet; give them the valid book.
fn archive() -> ZipArchive<File> {
    ZipArchive::new(File::open(fixture("valid")).expect("valid book")).expect("read zip")
}

fn opf_with(items: &[&str]) -> String {
    format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
<manifest>
{items}
</manifest>
<spine>
<itemref idref="ch1"/>
</spine>
</package>"#,
        items = items.join("\n")
    )
}

/// The manifest items of an OPF string, as (id, href, media-type); "" where absent.
fn items(opf: &str) -> Vec<(String, String, String)> {
    let root = Element::parse(opf.as_bytes()).expect("test opf must parse");
    let manifest = root.get_child("manifest").expect("manifest");
    manifest
        .children
        .iter()
        .filter_map(|n| n.as_element())
        .filter(|e| e.name == "item")
        .map(|e| {
            let get = |k: &str| e.attributes.get(k).cloned().unwrap_or_default();
            (get("id"), get("href"), get("media-type"))
        })
        .collect()
}

fn item(id: &str, href: &str, media_type: &str) -> (String, String, String) {
    (id.to_string(), href.to_string(), media_type.to_string())
}
