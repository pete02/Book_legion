#[cfg(test)]
#[cfg(test)]
mod db_handler_tests {
    use tribune_logistica::db_handlers::*;
    use crate::test::helpers::test_helpers;

    #[test]
    fn load_books_returns_all_books() {
        let (_dir, book_status, expected_bookdata,_) = test_helpers::setup_test_book();
        let books = load_books(&book_status.json ).unwrap();

        assert!(books.contains_key(&book_status.name));
        assert_eq!(books.get(&book_status.name).unwrap(), &expected_bookdata, "Books should hold bookdata");
    }

    #[test]
    fn load_book_returns_single_book() {
        let (_dir, book_status, expected_bookdata, _) = test_helpers::setup_test_book();
        let book = load_book(&book_status).unwrap();

        assert_eq!(book, expected_bookdata, "Book should equal bookdata");
    }

    #[test]
    fn load_book_errors_for_missing_book() {
        let (_dir, mut book_status, _expected_bookdata, _) = test_helpers::setup_test_book();
        book_status.name = "nonexistent".to_string();

        let err = load_book(&book_status).unwrap_err();
        assert!(err.to_string().contains("No such book in library"));
    }

    
    #[test]
    fn get_audiomap_returns_correct_map() {
        // Setup test book; capture the expected audiomap from setup
        let (_dir, book_status, _expected_bookdata, expected_map) = test_helpers::setup_test_book();

        // Fetch the audiomap using the function under test
        let fetched_map = get_audiomap(&book_status).unwrap();

        // Check that the number of entries matches
        assert_eq!(fetched_map.map.len(), expected_map.map.len(), "Audiomap length mismatch");

        // Validate every entry in the fetched map against the expected map
        for (key, expected_entry) in &expected_map.map {
            let fetched_entry = fetched_map.map.get(&key.clone())
                .expect(&format!("Missing audiomap entry for key {}", key));

            assert_eq!(fetched_entry.chapter_number, expected_entry.chapter_number, "Chapter mismatch for key {}", key);
            assert_eq!(fetched_entry.chunk_number, expected_entry.chunk_number, "Chunk mismatch for key {}", key);
            assert!((fetched_entry.start_time - expected_entry.start_time).abs() < f32::EPSILON, "Start time mismatch for key {}", key);
            assert!((fetched_entry.duration - expected_entry.duration).abs() < f32::EPSILON, "Duration mismatch for key {}", key);
        }
    }


    #[test]
    fn get_library_manifest_errors_on_missing_file() {
        let err = get_library_manifest("/nonexistent/path.json").unwrap_err();
        assert!(err.to_string().contains("No such file"));
    }
}
