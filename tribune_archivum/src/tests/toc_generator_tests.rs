#[cfg(test)]
mod toc_generator_tests{

use crate::lib::verifiers::verify_toc_integrity;
use crate::lib::generator::generate_toc; // assume this is your TOC generator
use crate::tests::test_epub::{TestEpub, TocItem};
use tempdir::TempDir;

#[test]
fn toc_generator_from_nav() {
    let dir = TempDir::new("toc_from_nav").unwrap();
    let path = dir.path().join("toc_from_nav.epub");

    // Create EPUB with a nav, but no toc.ncx
    let epub = TestEpub::new("Happy Book", true)
        .with_nav()
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c2.xhtml"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml",]);

    epub.write_to(&path).unwrap();

    // TOC verifier should fail (no toc.ncx)
    let result_before = verify_toc_integrity(&path);
    assert!(result_before.is_err());

    // Run the TOC generator (should create toc.ncx from nav.xhtml)
    generate_toc(&path).unwrap();

    // TOC verifier should now pass
    let result_after = verify_toc_integrity(&path);
    if result_after.is_err(){
        println!("res: {:?}", result_after);
    }
    assert!(result_after.is_ok());
}

#[test]
fn toc_generator_from_spine() {
    let dir = TempDir::new("toc_from_nav").unwrap();
    let path = dir.path().join("toc_from_nav.epub");

    // Create EPUB with a nav, but no toc.ncx
    let epub = TestEpub::new("Happy Book", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .no_toc()
        .spine(vec!["c1.xhtml","c2.xhtml",]);

    epub.write_to(&path).unwrap();

    // TOC verifier should fail (no toc.ncx)
    let result_before = verify_toc_integrity(&path);
    assert!(result_before.is_err());

    // Run the TOC generator (should create toc.ncx from nav.xhtml)
    generate_toc(&path).unwrap();

    // TOC verifier should now pass
    let result_after = verify_toc_integrity(&path);
    assert!(result_after.is_ok());
}
}