#[cfg(test)]
mod toc_integrity_tests{
use crate::lib::verifiers::verify_toc_integrity;
use crate::tests::test_epub::{TestEpub, TocItem};
use tempdir::TempDir;

#[test]
fn toc_happy_path() {
    let dir = TempDir::new("toc_happy").unwrap();
    let path = dir.path().join("toc_happy.epub");

    let epub = TestEpub::new("Happy TOC", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter_with_playorder("c1.xhtml", "chap1", "1"),
            TocItem::chapter_with_playorder("c2.xhtml", "chap2", "2"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    let integrity = crate::lib::verifiers::verify_zip_integrity(&path).unwrap();
    assert_eq!(integrity, false); // EPUB structurally valid

    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_ok());
}

#[test]
fn toc_invalid_playorder() {
    let dir = TempDir::new("toc_invalid_playorder").unwrap();
    let path = dir.path().join("toc_invalid_playorder.epub");

    let epub = TestEpub::new("Invalid PlayOrder", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter_with_playorder("c1.xhtml"," chap1", "one"), // invalid
            TocItem::chapter_with_playorder("c2.xhtml","chap2", "two"), // invalid
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    let integrity = crate::lib::verifiers::verify_zip_integrity(&path).unwrap();
    assert_eq!(integrity, false);

    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_err()); // Should fail due to non-numeric playOrder
}

#[test]
fn toc_missing_file() {
    let dir = TempDir::new("toc_missing_file").unwrap();
    let path = dir.path().join("toc_missing_file.epub");

    let epub = TestEpub::new("Missing TOC File", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter_with_playorder("c1.xhtml", "chap1","1"),
            TocItem::chapter_with_playorder("c3.xhtml", "chap2","2"), // file doesn't exist
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    let integrity = crate::lib::verifiers::verify_zip_integrity(&path).unwrap();
    assert_eq!(integrity, false);

    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_err()); // Should fail: c3.xhtml missing
}

#[test]
fn toc_navlabel_length_limit() {
    let dir = TempDir::new("toc_navlabel_length").unwrap();
    let path = dir.path().join("toc_navlabel_length.epub");

    // Create a label that is too long
    let long_label = "A".repeat(500); // 500 chars, definitely too long

    let epub = TestEpub::new("TOC Label Length", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter_with_playorder("c1.xhtml", &long_label, "1"),
            TocItem::chapter_with_playorder("c2.xhtml", "Chapter 2", "2"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    // Verify ZIP is structurally valid
    let integrity = crate::lib::verifiers::verify_zip_integrity(&path).unwrap();
    assert_eq!(integrity, false);

    // Verify TOC fails due to excessive navLabel text length
    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_err());
}


#[test]
fn toc_navlabel_empty() {
    let dir = TempDir::new("toc_navlabel_length").unwrap();
    let path = dir.path().join("toc_navlabel_length.epub");

    let epub = TestEpub::new("TOC Label Length", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter_with_playorder("c1.xhtml", "", "1"),
            TocItem::chapter_with_playorder("c2.xhtml", "Chapter 2", "2"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    // Verify ZIP is structurally valid
    let integrity = crate::lib::verifiers::verify_zip_integrity(&path).unwrap();
    assert_eq!(integrity, false);

    // Verify TOC fails due to excessive navLabel text length
    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_err());
}

#[test]
fn toc_no_navpoints() {
    let dir = TempDir::new("toc_no_navpoints").unwrap();
    let path = dir.path().join("toc_no_navpoints.epub");

    let epub = TestEpub::new("No TOC", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![]) // no navPoints
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();

    let toc_result = verify_toc_integrity(&path);
    assert!(toc_result.is_err()); // Should fail: no navPoints
}
}