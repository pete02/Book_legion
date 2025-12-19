#[cfg(test)]
mod book_handler_tests {
    use tribune_logistica::book_handler::*;
    use crate::test::helpers::test_helpers;

    #[test]
    fn get_chapter_returns_text() {
        let (_dir, status, _bookdata, _amap) = test_helpers::setup_test_book();
        let chapter_text = get_chapter(&status).unwrap();
        assert!(chapter_text.contains("Hello chapter one"));
    }

    #[test]
    fn get_chapter_errors_on_large_chapter() {
        let (_dir, mut status, _bookdata, _amap) = test_helpers::setup_test_book();
        status.chapter = 99; // bigger than actual chapters
        let res = get_chapter(&status);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), format!("chapter too large: 1 max"));
    }

    #[test]
    fn get_chunk_returns_correct_line() {
        let (_dir, mut status, _bookdata, _amap) = test_helpers::setup_test_book();
        status.chunk = 0;
        let chunk = get_chunk(&status).unwrap();
        assert!(chunk.contains("Hello chapter one"));
    }

    #[test]
    fn get_chunk_errors_when_chunk_too_far() {
        let (_dir, mut status, _bookdata, _amap) = test_helpers::setup_test_book();
        status.chunk = 129; // only one line in chapter
        let res = get_chunk(&status);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "requested chunk too far");
    }

    #[test]
    fn extract_files_returns_expected_types() {
        let (_dir, status, _bookdata, _amap) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);
        let files = extract_files(&epub_path, vec![".xhtml", ".opf"]).unwrap();
        assert!(files.keys().any(|k| k.contains("chapter1.xhtml")));
        assert!(files.keys().any(|k| k.contains("content.opf")));
    }

    #[test]
    fn extract_css_returns_empty_when_no_css() {
        let (_dir, status, _bookdata, _amap) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);
        let css = extract_css(&epub_path).unwrap();
        assert_eq!(css, ""); // our minimal EPUB has no CSS
    }

    #[test]
    fn extract_cover_errors_when_no_cover() {
        let (_dir, status, _bookdata, _amap) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);
        let res = extract_cover(&epub_path);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Cover not unabiguous");
    }
}
