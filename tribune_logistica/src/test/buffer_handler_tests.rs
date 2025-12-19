#[cfg(test)]
mod buffer_tests{
    use crate::test::helpers::test_helpers;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tribune_logistica::{buffer_handler, models::*};
    use serial_test::serial;


    #[tokio::test]
    #[serial]
    async fn fills_to_min_size() {
        //let t=test_helpers::get_real_data("fused", "./data", "books.json");;
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let book:BookKey=BookKey { name: t.name.clone(), path:t.path.clone() };
        let cursor=ChunkCursor { chapter: t.chapter, chunk: t.chunk, chapter_to_chunk: t.chapter_to_chunk, max_chapter:t.max_chapter };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(3, 8)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book, cursor).await;

        let buf = buffer.read().await;
        assert!(
            buf.chunks.len() >= buf.min_size,
            "buffer did not reach min_size"
        );
    }
     
    #[tokio::test]
    #[serial]
    async fn buffer_never_exceeds_max_size() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };

        let cursor = ChunkCursor {
            chapter: t.chapter,
            chunk: t.chunk,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(2, 5)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book, cursor).await;

        let buf = buffer.read().await;
        assert!(
            buf.chunks.len() <= buf.max_size,
            "buffer exceeded max_size"
        );
    }

    #[tokio::test]
    #[serial]
    async fn chunks_are_contiguous() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };

        let mut cursor = ChunkCursor {
            chapter: t.chapter,
            chunk: t.chunk,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(4, 10)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book, cursor.clone()).await;

        let buf = buffer.read().await;

        for chunk in buf.chunks.iter() {
            let (ch, ck) = test_helpers::parse_place(&chunk.place);
            assert_eq!(ch, cursor.chapter, "chapter discontinuity");
            assert_eq!(ck, cursor.chunk, "chunk discontinuity");
            let expected = buffer_handler::advance_cursor(cursor.clone());
            cursor = expected;
        }
    }

    #[tokio::test]
    #[serial]
    async fn buffer_crosses_chapter_boundary() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let mut cursor = ChunkCursor {
            chapter: t.chapter,
            chunk: t.chapter_to_chunk[&t.chapter] - 1,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(4, 8)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book, cursor.clone()).await;

        let buf = buffer.read().await;

        let crossed = buf.chunks.iter().any(|c| {
            let (ch, _) = test_helpers::parse_place(&c.place);
            ch > cursor.chapter
        });

        assert!(crossed, "buffer did not cross chapter boundary");
    }

    #[tokio::test]
    #[serial]
    async fn request_ahead_resets_buffer() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let mut cursor = ChunkCursor {
            chapter: t.chapter,
            chunk: t.chunk,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };
        

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(3, 6)));
        let tx = test_helpers::start_filler(buffer.clone()).await;
        test_helpers::ensure_and_wait(&tx, book.clone(), cursor.clone()).await;

        cursor.chunk += 10; // seek ahead

        test_helpers::ensure_and_wait(&tx, book, cursor.clone()).await;
        let buf = buffer.read().await;
        
        let first = buf.chunks.front().expect("buffer empty after reset");
        let (ch, ck) = test_helpers::parse_place(&first.place);

        assert_eq!(ch, cursor.chapter, "incorrect chapter");
        assert_eq!(ck, cursor.chunk, "incorrect chapter");
    }


    #[tokio::test]
    #[serial]
    async fn request_ahead_within_buffer() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let mut cursor = ChunkCursor {
            chapter: t.chapter,
            chunk: t.chunk,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };
        

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(3, 6)));
        let tx = test_helpers::start_filler(buffer.clone()).await;
        test_helpers::ensure_and_wait(&tx, book.clone(), cursor.clone()).await;

        cursor.chunk += 4; // seek ahead

        test_helpers::ensure_and_wait(&tx, book, cursor.clone()).await;
        let buf = buffer.read().await;
        
        let first = buf.chunks.front().expect("buffer empty after reset");
        let (ch, ck) = test_helpers::parse_place(&first.place);

        assert_eq!(ch, cursor.chapter, "incorrect chapter");
        assert_eq!(ck, cursor.chunk, "incorrect chapter");
    }



    #[tokio::test]
    #[serial]
    async fn request_too_far_behind_resets_buffer() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");

        let cursor = ChunkCursor {
            chapter: t.chapter+1,
            chunk: t.chunk,
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };
        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(5, 10)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book.clone(), cursor.clone()).await;

        let behind = ChunkCursor {
            chapter: t.chapter,
            chunk: 1,
            chapter_to_chunk: cursor.chapter_to_chunk.clone(),
            max_chapter: cursor.max_chapter,
        };

        test_helpers::ensure_and_wait(&tx, book, behind.clone()).await;

        let buf = buffer.read().await;
        let first = buf.chunks.front().expect("buffer empty after reset");
        let (ch, ck) = test_helpers::parse_place(&first.place);

        assert_eq!(ch, behind.chapter);
        assert_eq!(ck, behind.chunk);
    }


    #[tokio::test]
    #[serial]
    async fn end_of_book_stops_filling() {
        let t=test_helpers::get_real_data("fused", "./data", "books.json");
        let cursor = ChunkCursor {
            chapter: t.max_chapter,
            chunk: t.chapter_to_chunk[&(t.max_chapter)],
            chapter_to_chunk: t.chapter_to_chunk.clone(),
            max_chapter: t.max_chapter,
        };

        let book = BookKey {
            name: t.name.clone(),
            path: t.path.clone(),
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(2, 10)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book, cursor).await;

        let buf = buffer.read().await;

        assert!(
            buf.chunks.len() <= 1,
            "buffer kept filling past end of book"
        );
    }

    #[tokio::test]
    #[serial]
    async fn book_change_clears_buffer() {
        let t1=test_helpers::get_real_data("fused", "./data", "books.json");
        let t2=test_helpers::get_real_data("fusing", "./data", "books.json");

        let book1 = BookKey {
            name: t1.name.clone(),
            path: t1.path.clone(),
        };

        let book2 = BookKey {
            name: t2.name.clone(),
            path: t2.path.clone(),
        };

        let cursor1 = ChunkCursor {
            chapter: t1.chapter,
            chunk: t1.chunk,
            chapter_to_chunk: t1.chapter_to_chunk.clone(),
            max_chapter: t1.max_chapter,
        };

        let cursor2 = ChunkCursor {
            chapter: t2.chapter,
            chunk: t2.chunk,
            chapter_to_chunk: t2.chapter_to_chunk.clone(),
            max_chapter: t2.max_chapter,
        };

        let buffer = Arc::new(RwLock::new(AudioBuffer::new(3, 6)));
        let tx = test_helpers::start_filler(buffer.clone()).await;

        test_helpers::ensure_and_wait(&tx, book1, cursor1).await;
        test_helpers::ensure_and_wait(&tx, book2, cursor2.clone()).await;

        let buf = buffer.read().await;
        let first = buf.chunks.front().expect("buffer empty after book change");
        let (ch, ck) = test_helpers::parse_place(&first.place);

        assert_eq!(ch, cursor2.chapter);
        assert_eq!(ck, cursor2.chunk);
    }



}

#[cfg(test)]
mod seek_tests {
    use tribune_logistica::models::ChunkCursor;
    use tribune_logistica::buffer_handler::SeekDecision;
    use tribune_logistica::buffer_handler::classify_seek;



    fn cursor(
        chapter: u32,
        chunk: u32,
        chapter_to_chunk: &[(u32, u32)],
        max_chapter: u32,
    ) -> ChunkCursor {
        ChunkCursor {
            chapter,
            chunk,
            chapter_to_chunk: chapter_to_chunk.iter().cloned().collect(),
            max_chapter,
        }
    }

    fn chapters() -> Vec<(u32, u32)> {
        vec![
            (1, 10),
            (2, 10),
            (3, 10),
        ]
    }

    /*
     * Cursor is at: chapter 2, chunk 5
     * Buffered = 6
     *
     * Oldest buffered position:
     *   chapter 2, chunk 5
     *   rewind 6 → chapter 1, chunk 9
     */

    #[test]
    fn seek_ahead_of_cursor_resets() {
        let c = cursor(2, 5, &chapters(), 3);
        let start = cursor(2, 6, &chapters(), 3);

        let decision = classify_seek(&start, &c, 6);
        assert_eq!(decision, SeekDecision::Reset);
    }

    #[test]
    fn seek_far_behind_resets() {
        let c = cursor(2, 5, &chapters(), 3);
        let start = cursor(1, 1, &chapters(), 3);

        let decision = classify_seek(&start, &c, 6);
        assert_eq!(decision, SeekDecision::Reset);
    }

    #[test]
    fn seek_exactly_at_oldest_is_noop() {
        let c = cursor(2, 5, &chapters(), 3);
        let start = cursor(1, 9, &chapters(), 3);

        let decision = classify_seek(&start, &c, 6);
        assert_eq!(decision, SeekDecision::NoOp);
    }

    #[test]
    fn seek_inside_buffer_trims() {
        let c = cursor(2, 5, &chapters(), 3);
        let start = cursor(2, 3, &chapters(), 3);

        let decision = classify_seek(&start, &c, 6);
        assert_eq!(decision, SeekDecision::TrimToStart);
    }

    #[test]
    fn seek_same_position_is_noop() {
        let c = cursor(2, 5, &chapters(), 3);
        let start = cursor(1, 9, &chapters(), 3);

        let decision = classify_seek(&start, &c, 6);
        assert_eq!(decision, SeekDecision::NoOp);
    }

    #[test]
    fn seek_cross_chapter_inside_buffer_trims() {
        let c = cursor(2, 3, &chapters(), 3);
        let start = cursor(1, 10, &chapters(), 3);

        let decision = classify_seek(&start, &c, 5);
        assert_eq!(decision, SeekDecision::TrimToStart);
    }

    #[test]
    fn seek_cross_chapter_too_far_resets() {
        let c = cursor(2, 3, &chapters(), 3);
        let start = cursor(1, 4, &chapters(), 3);

        let decision = classify_seek(&start, &c, 5);
        assert_eq!(decision, SeekDecision::Reset);
    }
}
