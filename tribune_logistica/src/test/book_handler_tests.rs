#[cfg(test)]
mod book_handler_tests {
    use tribune_logistica::book_handler::*;
    use crate::test::helpers::test_helpers;
    use std::path::Path;

    /*
     * init_book
     */

    #[test]
    fn init_book_audio_ok() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();

        let res = init_book(&status.name, "audio", &status.json);
        assert!(res.is_ok());

        let new_status = res.unwrap();
        assert_eq!(new_status.name, status.name);
        assert_eq!(new_status.chapter, status.chapter);
        assert_eq!(new_status.chunk, status.chunk);
    }

    #[test]
    fn init_book_text_ok() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();

        let res = init_book(&status.name, "text", &status.json);
        assert!(res.is_ok());
    }

    #[test]
    fn init_book_rejects_invalid_type() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();

        let err = init_book(&status.name, "pdf", &status.json).unwrap_err();
        assert_eq!(err["status"], "incorrect format");
    }

    #[test]
    fn init_book_errors_on_missing_book() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();

        let err = init_book("nonexistent", "audio", &status.json).unwrap_err();
        assert_eq!(err["status"], "not in library");
    }

    /*
     * get_chapter
     */

    #[test]
    fn get_chapter_returns_text() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();

        let txt = get_chapter(&status).unwrap();
        assert!(!txt.is_empty());
    }

    #[test]
    fn get_chapter_errors_on_overflow() {
        let (_dir, mut status, _data,_) = test_helpers::setup_test_book();
        status.chapter = status.max_chapter + 1;

        let err = get_chapter(&status).unwrap_err();
        assert_eq!(err, "chapter too large");
    }

    /*
     * extract_files
     */

    #[test]
    fn extract_files_finds_css() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);

        let files = extract_files(&epub_path, vec![".css"]).unwrap();
        assert!(!files.is_empty());
    }

    #[test]
    fn extract_files_returns_empty_for_missing_type() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);

        let files = extract_files(&epub_path, vec![".woff"]).unwrap();
        assert!(files.is_empty());
    }

    /*
     * extract_css
     */

    #[test]
    fn extract_css_concatenates_css() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();
        let epub_path = format!("{}/{}.epub", status.path, status.name);

        let css = extract_css(&epub_path).unwrap();
        assert!(!css.is_empty());
    }

    #[test]
    fn extract_css_errors_on_missing_epub() {
        let err = extract_css("/no/such/book.epub");
        assert!(err.is_err());
    }
}
