#[cfg(test)]
mod epub_tests{
    use crate::lib::verifiers::verify_zip_integrity;
    use crate::tests::test_epub::TestEpub;
    use crate::tests::test_epub::TocItem;

#[test]
fn epub_integrity_happy_path() {
    let dir = tempdir::TempDir::new("happy_path").unwrap();
    let path = dir.path().join("happy.epub");

    // Build a fully valid EPUB
    let epub = TestEpub::new("Happy Book", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c2.xhtml"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml",]);

    epub.write_to(&path).unwrap();
    let result = verify_zip_integrity(&path);
    
    if result.is_err(){
        println!("{:?}",result);
    }

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn epub_integrity_invalid_zip() {
    let dir = tempdir::TempDir::new("invalid_zip").unwrap();
    let path = dir.path().join("invalid.epub");
    std::fs::write(&path, b"not a zip").unwrap();

    let result = verify_zip_integrity(&path);
    assert!(result.is_err());
}
#[test]
fn epub_integrity_missing_container() {
    let dir = tempdir::TempDir::new("happy_path").unwrap();
    let path = dir.path().join("happy.epub");

    // Build a fully valid EPUB
    let epub = TestEpub::new("Happy Book", false)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
            TocItem::chapter("c2.xhtml"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"]);

    epub.write_to(&path).unwrap();
    assert!(verify_zip_integrity(&path).is_err())
}

#[test]
fn epub_integrity_missing_file_from_manifest() {
    let dir = tempdir::TempDir::new("missing_spine_file").unwrap();
    let path = dir.path().join("missing_spine_file.epub");

    let epub = TestEpub::new("Missing Chapter", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .toc(vec![
            TocItem::chapter("c1.xhtml"),
        ])
        .spine(vec!["c1.xhtml","c2.xhtml"])
        .remove_file("c2.xhtml");

    epub.write_to(&path).unwrap();

    let result = verify_zip_integrity(&path);
    assert!(result.is_err())
}


#[test]
fn epub_integrity_missing_toc() {
    let dir = tempdir::TempDir::new("missing_toc").unwrap();
    let path = dir.path().join("missing_toc.epub");

    let epub = TestEpub::new("No TOC", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .spine(vec!["c1.xhtml","c2.xhtml"])
        .no_toc();

    epub.write_to(&path).unwrap();

    let result = verify_zip_integrity(&path).unwrap();

    
    assert_eq!(result,true)
}


#[test]
fn epub_integrity_missing_spine() {
    let dir = tempdir::TempDir::new("missing_toc").unwrap();
    let path = dir.path().join("missing_toc.epub");

    let epub = TestEpub::new("No TOC", true)
        .chapters(vec!["c1.xhtml", "c2.xhtml"])
        .no_toc();

    epub.write_to(&path).unwrap();

    assert!(verify_zip_integrity(&path).is_err());
}

#[test]
fn epub_integrity_wrong_spine() {
    let dir = tempdir::TempDir::new("missing_toc").unwrap();
    let path = dir.path().join("missing_toc.epub");

    let epub = TestEpub::new("No TOC", true)
        .chapters(vec!["c1.xhtml", "wrong"])
        .no_toc();

    epub.write_to(&path).unwrap();

    assert!(verify_zip_integrity(&path).is_err());
}


}