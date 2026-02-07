use tempfile::tempdir;
use crate::analysis::*;
use crate::tests::test_epub_generator::*;

#[test]
fn detects_toc_presence() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("toc_present.epub");

    let epub = TestEpub::new("With ToC")
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c2.xhtml"),
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(result.toc_present);
}

#[test]
fn detects_missing_toc() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("no_toc.epub");

    let epub = TestEpub::new("No ToC")
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .no_toc();

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_present);
}

#[test]
fn validates_perfect_toc() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("perfect_match.epub");

    let epub = TestEpub::new("Perfect ToC")
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c2.xhtml"),
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(result.toc_matches_chapters);
    assert!(result.missing_chapters.is_empty());
    assert!(result.orphan_toc_entries.is_empty());
}

#[test]
fn detects_missing_chapter() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing_chapter.epub");

    let epub = TestEpub::new("Missing Chapter")
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            // missing c2.xhtml in ToC
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_matches_chapters);
    assert_eq!(result.missing_chapters.len(), 1);
}

#[test]
fn detects_orphan_toc_entry() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("orphan_entry.epub");

    let epub = TestEpub::new("Orphan ToC Entry")
        .chapters(vec!["c1.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("ghost.xhtml"),
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_matches_chapters);
    assert_eq!(result.orphan_toc_entries.len(), 1);
}

#[test]
fn detects_order_mismatch() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("order_mismatch.epub");

    let epub = TestEpub::new("Order Mismatch")
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c2.xhtml"),
            TocItem::chapter("c1.xhtml"),
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_matches_chapters);
}

#[test]
fn detects_broken_anchor() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("broken_anchor.epub");

    let epub = TestEpub::new("Broken Anchor")
        .chapters(vec!["c1.xhtml"])
        .toc(vec![
            TocItem::anchor("c1.xhtml", "missing_anchor"),
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_matches_chapters);
}

#[test]
fn detects_duplicate_toc_entries() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("duplicate_entries.epub");

    let epub = TestEpub::new("Duplicate Entries")
        .chapters(vec!["c1.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c1.xhtml"), // duplicate
        ]);

    epub.write_to(&path).unwrap();

    let result = analyze_epub(&path);
    assert!(!result.toc_matches_chapters);
}