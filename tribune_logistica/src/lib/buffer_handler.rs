use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::{db_handlers, models::*};
use crate::audio_handler;
use tokio::sync::oneshot;

pub enum FillerCommand {
    Ensure {
        book: BookKey,
        start: ChunkCursor,
        respond_to: Option<oneshot::Sender<SeekDecision>>,
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

    loop {
        tokio::select! {
            // Control plane: reconcile intent
            Some(cmd) = rx.recv() => {
                match cmd {
                    FillerCommand::Stop => break,

                    FillerCommand::Ensure { book, start, respond_to } => {
                        let mut buf = buffer.write().await;
                        
                        let decision=if current_book.as_ref() != Some(&book) {
                            buf.clear(book.clone());
                            current_book = Some(book);
                            cursor = Some(start);
                            SeekDecision::Reset
                        }else{
                            let c=classify_seek(&start, &buf);
                            match c {
                                SeekDecision::Reset => {
                                    buf.clear(current_book.clone().unwrap());
                                    cursor = Some(start);
                                }
                                SeekDecision::TrimToStart => {
                                    buf.trim_until(start.chapter, start.chunk);
                                }
                                SeekDecision::NoOp => {}
                            }
                            c
                        };
                        if let Some(tx)=respond_to{
                            let _ =tx.send(decision);
                        }
                    }
                }
            }

            // Work loop: fill opportunistically
            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
                let c = match cursor.clone() {
                    Some(c) => c,
                    None => continue,
                };
                let at_end=
                    c.chapter == c.max_chapter &&
                    c.chunk == c.chapter_to_chunk[&c.chapter];

                let should_fill = {
                    let buf = buffer.read().await;
                    buf.chunks.len() < buf.max_size && !at_end
                };

                if !should_fill {
                    continue;
                }

                let status = BookStatus {
                    name: current_book.as_ref().unwrap().name.clone(),
                    path: current_book.as_ref().unwrap().path.clone(),
                    chapter: c.chapter,
                    chunk: c.chunk,
                    initial_chapter: 0,
                    time: 0.0,
                    json: String::new(),
                    max_chapter: c.max_chapter,
                    chapter_to_chunk: HashMap::new(),
                    duration: 0.0,
                };

                let map = match db_handlers::get_audiomap(&status) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let data = match audio_handler::get_audio_chunk(
                    &status,
                    &map,
                    c.chapter as usize,
                    c.chunk as usize,
                    "test.mp3",
                    false,
                ) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let mut buf = buffer.write().await;
                buf.push(AudioChunkResult {
                    data,
                    place: format!("{},{}", c.chapter, c.chunk),
                    reached_end: at_end,
                });

                cursor = Some(advance_cursor(c));
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
pub fn classify_seek(
    seeker: &ChunkCursor,
    buffer: &AudioBuffer,
) -> SeekDecision {
    let first = match buffer.chunks.front() {
        Some(c) => c,
        None => return SeekDecision::Reset,
    };

    let last = match buffer.chunks.back() {
        Some(c) => c,
        None => return SeekDecision::Reset,
    };

    let (seek_ch, seek_ck) = (seeker.chapter, seeker.chunk);
    let (first_ch, first_ck) = parse_place(&first.place);
    let (last_ch, last_ck) = parse_place(&last.place);

    if seek_ch == first_ch && seek_ck == first_ck {
        return SeekDecision::NoOp; // First chunk → nothing to do
    }

    if is_after(seek_ch, seek_ck, first_ch, first_ck) &&
       is_before_or_equal(seek_ch, seek_ck, last_ch, last_ck) {
        return SeekDecision::TrimToStart; // Inside buffer → drop earlier chunks
    }

    SeekDecision::Reset // Outside buffer → reset
}


// Helpers
fn is_after(a_ch:u32, a_ck:u32, b_ch:u32, b_ck:u32) -> bool {
    a_ch > b_ch || (a_ch == b_ch && a_ck > b_ck)
}
fn is_before_or_equal(a_ch:u32, a_ck:u32, b_ch:u32, b_ck:u32) -> bool {
    a_ch < b_ch || (a_ch == b_ch && a_ck <= b_ck)
}


