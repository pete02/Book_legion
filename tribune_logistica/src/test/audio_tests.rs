#[cfg(test)]
mod audio_tests {
    use tribune_logistica::audio_handler;
    use crate::test::helpers::test_helpers;
    #[test]
    fn test_create_temp_mp3() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;
        println!("status_path: {}",status.path);
         assert!(audio_handler::get_audio_chunk(
            &status.name,
            1,
            1,
            "test.mp3",
            true,
            base
        ).is_ok());
    }

    #[test]
    fn get_audio_chunks_errors_without_book() {
        let base = ".";
        let err = audio_handler::get_audio_chunks(None, 3, base).unwrap_err();
        assert_eq!(err.to_string(), "No book");
    }

    #[test]
    fn get_audio_chunks_respects_advance_limit() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;

        let chunks = audio_handler::get_audio_chunks(Some(&status), 3, base).unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn get_audio_chunks_stops_at_max() {
        let (_dir, mut status) = test_helpers::setup_test_book();
        let base = &status.path;
        status.chunk = 4;

        let chunks = audio_handler::get_audio_chunks(Some(&status), 10, base).unwrap();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks.last().unwrap().place, "1,5");
        assert!(chunks.last().unwrap().reached_end);
    }

    #[test]
    fn only_last_chunk_sets_reached_end() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;

        let chunks = audio_handler::get_audio_chunks(Some(&status), 10, base).unwrap();

        for (idx, chunk) in chunks.iter().enumerate() {
            if idx == chunks.len() - 1 {
                assert!(chunk.reached_end);
            } else {
                assert!(!chunk.reached_end);
            }
        }
    }

    #[test]
    fn places_are_sequential_and_correct() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;

        let chunks = audio_handler::get_audio_chunks(Some(&status), 3, base).unwrap();
        let places: Vec<_> = chunks.iter().map(|c| c.place.clone()).collect();
        assert_eq!(places, vec!["1,1", "1,2", "1,3"]);
    }

    #[test]
    fn get_audio_chunk_errors_on_missing_chunk() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;

        let err = audio_handler::get_audio_chunk(
            &status.name,
            status.chapter as usize,
            99,
            "out.mp3",
            false,
            base
        ).unwrap_err();

        assert!(err.to_string().contains("no such starting point"));
    }

    #[test]
    fn get_audio_chunk_deletes_file_when_not_kept() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;
        let output = "tmp_out.mp3";

        let _ = audio_handler::get_audio_chunk(
            &status.name,
            status.chapter as usize,
            1,
            output,
            false,
            base
        ).unwrap();

        assert!(!std::path::Path::new(output).exists());
    }

    #[test]
    fn get_audio_chunk_keeps_file_when_requested() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;
        let output = "tmp_out.mp3";

        let _ = audio_handler::get_audio_chunk(
            &status.name,
            status.chapter as usize,
            1,
            output,
            true,
            base
        ).unwrap();

        assert!(std::path::Path::new(output).exists());

        std::fs::remove_file(output).unwrap();
    }

    #[test]
    fn get_audio_chunk_returns_audio_data() {
        let (_dir, status) = test_helpers::setup_test_book();
        let base = &status.path;

        let data = audio_handler::get_audio_chunk(
            &status.name,
            status.chapter as usize,
            1,
            "out.mp3",
            false,
            base
        ).unwrap();

        assert!(!data.is_empty());
    }
}
