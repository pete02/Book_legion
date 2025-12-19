use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::sync::Mutex;



use crate::{audio_gen_handler, db_handlers, models::*};
use crate::audio_handler;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration, timeout};

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

    // Lock to ensure only one fetch at a time
    let fetch_lock = Arc::new(Mutex::new(false));

    // Channel for completed chunks
    let (tx_chunk, mut rx_chunk) = mpsc::channel::<(AudioChunkResult, Option<BookKey>)>(100);
    loop {
        tokio::select! {
            // --- Handle commands ---
            Some(cmd) = rx.recv() => {
                match cmd {
                    FillerCommand::Stop => break,

                    FillerCommand::Ensure { book, start, respond_to } => {
                        let mut buf = buffer.write().await;

                        let decision = if current_book.as_ref() != Some(&book) {
                            buf.clear(book.clone());
                            current_book = Some(book);
                            cursor = Some(start);
                            SeekDecision::Reset
                        } else {
                            let c = classify_seek(&start, &buf);
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

                        if let Some(tx) = respond_to {
                            let _ = tx.send(decision);
                        }
                    }
                }
            }

            // --- Opportunistic filler ---
            _ = sleep(Duration::from_millis(10)) => {
                let c = match cursor.clone() {
                    Some(c) => c,
                    None => continue,
                };

                let at_end = c.chapter == c.max_chapter && c.chunk == c.chapter_to_chunk[&c.chapter];

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

                // --- Try to acquire fetch lock for 10ms ---
                let mut start_fetch = false;
                match timeout(Duration::from_millis(10), fetch_lock.lock()).await {
                    Ok(mut guard) => {
                        if !*guard {
                            *guard = true;
                            start_fetch = true;
                        }
                    }
                    Err(_) => {
                        // Lock unavailable, skip this iteration
                        continue;
                    }
                }

                if !start_fetch {
                    continue;
                }

                // Clone variables for the spawned task
                let fetch_lock_clone = Arc::clone(&fetch_lock);
                let tx_clone = tx_chunk.clone();
                let status_clone = status.clone();
                let c_clone = c.clone();
                let at_end_clone = at_end;
                let current_book=current_book.clone();
                tokio::spawn(async move {
                    // Fetch audio chunk
                    let data = match audio_gen_handler::get_audio_chunk(&status_clone).await {
                        Ok(d) => d,
                        Err(err) => {
                            eprintln!("Failed to get audio chunk: {}", err);
                            let mut guard = fetch_lock_clone.lock().await;
                            *guard = false;
                            return;
                        }
                    };

                    // Send completed chunk to channel
                    let chunk = AudioChunkResult {
                        data,
                        place: format!("{},{}", c_clone.chapter, c_clone.chunk),
                        reached_end: at_end_clone,
                    };

                    if tx_clone.send((chunk,current_book)).await.is_err() {
                        eprintln!("Failed to send audio chunk to channel");
                    }

                    // Release fetch lock
                    let mut guard = fetch_lock_clone.lock().await;
                    *guard = false;
                });

                cursor = Some(advance_cursor(c));
            }
        }

        // --- Drain completed chunks from channel ---
        while let Ok((chunk,key)) = rx_chunk.try_recv() {
            if key==current_book{
                let mut buf = buffer.write().await;
                buf.push(chunk);
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


