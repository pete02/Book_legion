use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::{db_handlers, models::*};
use crate::audio_handler;

pub enum FillerCommand {
    Ensure {
        book: BookKey,
        start: ChunkCursor,
    },
    Stop,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SeekDecision {
    Reset,          // outside window or backwards
    TrimToStart,    // ahead but inside buffer
    NoOp,           // same or behind cursor but still valid
}

pub async fn run_filler(
    mut rx: mpsc::Receiver<FillerCommand>,
    buffer: Arc<RwLock<AudioBuffer>>,
) {
    let mut current_book: Option<BookKey> = None;
    let mut cursor: Option<ChunkCursor> = None;
    let mut just_reset=false;

    while let Some(cmd) = rx.recv().await {
        match cmd {
            FillerCommand::Stop => break,

            FillerCommand::Ensure { book, start } => {
                // Reset on book change
                if current_book.as_ref() != Some(&book) {
                    let mut buf = buffer.write().await;
                    buf.clear(book.clone());
                    current_book = Some(book.clone());
                    cursor = Some(start);
                }else{
                    let mut buf = buffer.write().await;
                    let c= cursor.clone().unwrap();
                    match classify_seek(&start, &c, buf.chunks.len()) {
                        SeekDecision::Reset => {
                            buf.clear(book.clone());
                            cursor = Some(start);
                            just_reset = true;
                        }

                        SeekDecision::TrimToStart => {
                            buf.trim_until(start.chapter, start.chunk);
                        }

                        SeekDecision::NoOp => {}
                    }

                }

                let should_not_start={
                    let buf = buffer.read().await;
                    buf.chunks.len() > buf.min_size
                };


                if !should_not_start{
                    loop {
                        let c = cursor.clone().unwrap();

                        let should_fill = {
                            let buf = buffer.read().await;
                            let at_end = c.chapter == c.max_chapter && c.chunk == c.chapter_to_chunk[&c.chapter];

                            buf.chunks.len() < buf.max_size && !at_end
                        };

                        if !should_fill {
                            break;
                        }


                        let status = BookStatus {
                            name: book.name.clone(),
                            path: book.path.clone(),
                            chapter: c.chapter,
                            chunk: c.chunk,
                            // remaining fields omitted for brevity
                            initial_chapter: 0,
                            time: 0.0,
                            json: String::new(),
                            max_chapter: c.max_chapter,
                            chapter_to_chunk: HashMap::new(),
                            duration: 0.0,
                        };

                        let map = match db_handlers::get_audiomap(&status) {
                            Ok(m) => m,
                            Err(_) => {
                                println!("error in getting audiomap");
                                break
                            },
                        };

                        let result = audio_handler::get_audio_chunk(&status, &map, c.chapter as usize, c.chunk as usize, "test.mp3", false);
                        let chunk = match result {
                            Ok(d) => AudioChunkResult{ data: d, place: format!("{},{}",c.chapter,c.chunk), reached_end: false},
                            Err(a) => {
                                println!("error in getting audio_chunks: {}",a);
                                break
                            },
                        };

                        let mut buf = buffer.write().await;{
                            buf.push(chunk);
                            cursor = Some(advance_cursor(cursor.unwrap()));
                        }
                    }
                }
            }
        }
    }
}


pub fn advance_cursor(mut c: ChunkCursor) -> ChunkCursor {
    if c.chunk == c.chapter_to_chunk[&c.chapter]{
        c.chapter+=1;
        c.chunk=1;
    }else{
        c.chunk += 1;
    }
    c
}


fn is_outside_window(
    start: &ChunkCursor,
    cursor: &ChunkCursor,
    buffered: usize,
) -> bool {
    let start_pos = Position::from(start);
    let cursor_pos = Position::from(cursor);

    // Requested ahead of cursor → reset
    if start_pos > cursor_pos {
        return true;
    }

    // Compute oldest buffered position (may cross chapters)
    let oldest = rewind_cursor(cursor.clone(), buffered);

    let oldest_pos = Position::from(&oldest);
    // Requested too far behind → reset
    if start_pos < oldest_pos {
        return true;
    }

    false
}


pub fn classify_seek(
    start: &ChunkCursor,
    cursor: &ChunkCursor,
    buffered: usize,
) -> SeekDecision {
    let start_pos = Position::from(start);
    let cursor_pos = Position::from(cursor);
    // Ahead of cursor → always reset
    if start_pos > cursor_pos {
        return SeekDecision::Reset;
    }

    let oldest = rewind_cursor(cursor.clone(), buffered);
    let oldest_pos = Position::from(&oldest);

    // Too far behind → reset
    if start_pos < oldest_pos {
        return SeekDecision::Reset;
    }

    // Ahead of oldest but <= cursor → trim
    if start_pos > oldest_pos {
        return SeekDecision::TrimToStart;
    }

    SeekDecision::NoOp
}


fn rewind_cursor(
    mut c: ChunkCursor,
    mut steps: usize,
) -> ChunkCursor {
    while steps > 0 {
        if c.chunk > 1 {
            c.chunk -= 1;
        } else {
            // Move to previous chapter
            if c.chapter == 1 {
                break;
            }
            c.chapter -= 1;
            c.chunk = c.chapter_to_chunk[&c.chapter];
        }
        steps -= 1;
    }
    c
}