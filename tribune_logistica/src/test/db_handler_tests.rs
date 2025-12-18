#[cfg(test)]
#[cfg(test)]
mod db_handler_tests {
    use tribune_logistica::db_handlers::*;
    use crate::test::helpers::test_helpers;

    #[test]
    fn load_books_returns_all_books() {
        let (_dir, book_status, expected_bookdata) = test_helpers::setup_test_book();
        let books = load_books(&book_status.json).unwrap();

        assert!(books.contains_key(&book_status.name));
        assert_eq!(books.get(&book_status.name).unwrap(), &expected_bookdata, "Books should hold bookdata");
    }

    #[test]
    fn load_book_returns_single_book() {
        let (_dir, book_status, expected_bookdata) = test_helpers::setup_test_book();
        let book = load_book(&book_status).unwrap();

        assert_eq!(book, expected_bookdata, "Book should equal bookdata");
    }

    #[test]
    fn load_book_errors_for_missing_book() {
        let (_dir, mut book_status, _expected_bookdata) = test_helpers::setup_test_book();
        book_status.name = "nonexistent".to_string();

        let err = load_book(&book_status).unwrap_err();
        assert!(err.to_string().contains("No such book in library"));
    }

    #[test]
    fn get_audiomap_returns_correct_map() {
        let (_dir, book_status, _expected_bookdata) = test_helpers::setup_test_book();

        let map = get_audiomap(&book_status).unwrap();

        let key = format!("{},{}", book_status.chapter, 1);
        assert!(map.map.contains_key(&key));
        let entry = map.map.get(&key).unwrap();
        assert_eq!(entry.chapter_number as u32, book_status.chapter);
        assert_eq!(entry.chunk_number, 1);
    }

    #[test]
    fn get_library_manifest_errors_on_missing_file() {
        let err = get_library_manifest("/nonexistent/path.json").unwrap_err();
        assert!(err.to_string().contains("No such file"));
    }
}
