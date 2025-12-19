#[cfg(test)]
mod update_progress_tests {
    use crate::test::helpers::test_helpers;
    use tribune_logistica::models::AudioMap;
    use tribune_logistica::update_handler::update_progress;
    use tribune_logistica::db_handlers::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn updates_progress_using_audiomap_time() {
        let (_dir, mut status, _data,map) = test_helpers::setup_test_book();

        status.chunk = 1;
        status.chapter = status.initial_chapter;
        status.time = 999.0; // should be overridden

        update_progress(&status.clone(),&map).unwrap();

        let books = load_books(&status.json).unwrap();
        let book = books.get(&status.name).unwrap();

        assert_eq!(book.current_chapter, status.chapter);
        assert_eq!(book.current_chunk, status.chunk);
        assert_eq!(book.current_time, 0.0); // audiomap start_time
    }

    #[test]
    fn updates_progress_when_audiomap_entry_missing() {
        let (_dir, status, _data,_) = test_helpers::setup_test_book();
        let map=AudioMap{name: status.name.clone(), map:HashMap::new()};

        update_progress(&status.clone(),&map).unwrap();

        let books = load_books(&status.json).unwrap();
        let book = books.get(&status.name).unwrap();

        assert_eq!(book.current_chunk, status.chunk);
        assert_eq!(book.current_chapter, status.chapter);
        assert_eq!(book.current_time, status.time);
    }

    #[test]
    fn errors_on_missing_manifest() {
        let (_dir, status, _data,map) = test_helpers::setup_test_book();

        fs::remove_file(&status.json).unwrap();

        let res = update_progress(&status,&map);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "missing manifest");
    }

    #[test]
    fn errors_when_book_not_in_library() {
        let (_dir, mut status, _data,map) = test_helpers::setup_test_book();

        status.name = "nonexistent".into();

        let res = update_progress(&status,&map);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "not in library");
    }

    #[test]
    fn errors_on_chapter_overflow() {
        let (_dir, mut status, _data,map) = test_helpers::setup_test_book();

        status.chapter = 999;

        let res = update_progress(&status,&map);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "chapter overflow");
    }

    #[test]
    fn errors_on_invalid_chapter_number() {
        let (_dir, mut status, _data,map) = test_helpers::setup_test_book();

        status.chapter = 333333; // greater than chapter_to_chunk map

        let res = update_progress(&status,&map);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "chapter overflow");
    }

    #[test]
    fn errors_on_chunk_overflow() {
        let (_dir, mut status, data,map) = test_helpers::setup_test_book();

        let max = data.chapter_to_chunk[&status.chapter];
        status.chunk = max + 1;

        let res = update_progress(&status,&map);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "chunk overflow");
    }
}
