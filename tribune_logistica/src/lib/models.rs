use std::collections::{HashMap, VecDeque};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AudioMapEntry {
    pub chapter_number: usize,
    pub chunk_number:usize,
    pub start_time: f32, // seconds
    pub duration: f32,
}

#[derive(Serialize, Deserialize)]
pub struct AudioMap {
    pub name:String,
    pub map: HashMap<String, AudioMapEntry>,
}
impl AudioMap {
    pub fn get(&self, key: (usize, usize)) -> Option<&AudioMapEntry> {
        let key_str = format!("{},{}", key.0, key.1);
        self.map.get(&key_str)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BookData {
    pub path: String,
    pub initial_chapter: u32,
    pub duration: f32,
    pub current_chunk: u32,
    pub current_chapter: u32,
    pub current_time: f32,
    pub chapter_to_chunk: HashMap<u32, u32>,
    pub max_chapter: u32,
}

impl BookData {
    /// Creates a new BookData with minimal parameters and defaults
    pub fn new(path: &str, initial_chapter: u32, max_chapter: u32, duration: f32) -> Self {
        let mut chapter_to_chunk = HashMap::new();
        for chapter in 1..=max_chapter {
            // default 10 chunks per chapter
            chapter_to_chunk.insert(chapter, 10);
        }

        Self {
            path: path.to_string(),
            initial_chapter,
            duration,
            current_chunk: 1,
            current_chapter: initial_chapter,
            current_time: 0.0,
            chapter_to_chunk,
            max_chapter,
        }
    }

    /// Resets the book to its initial state
    pub fn reset(&mut self) {
        self.current_chapter = self.initial_chapter;
        self.current_chunk = 1;
        self.current_time = 0.0;
    }

    /// Returns the number of chunks in the current chapter
    pub fn current_chapter_chunks(&self) -> u32 {
        *self.chapter_to_chunk.get(&self.current_chapter).unwrap_or(&0)
    }

    /// Advances the book by one chunk (updates current_chunk and current_time)
    pub fn advance_chunk(&mut self, chunk_duration: f32) {
        self.current_chunk += 1;
        self.current_time += chunk_duration;

        if self.current_chunk > self.current_chapter_chunks() {
            self.current_chapter += 1;
            self.current_chunk = 1;
        }

        if self.current_chapter > self.max_chapter {
            self.current_chapter = self.max_chapter;
            self.current_chunk = self.current_chapter_chunks();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct BookStatus {
    pub name:String,
    pub path:String,
    pub chapter: u32,
    pub initial_chapter: u32,
    pub time: f32,
    pub chunk: u32,
    pub json: String,
    pub max_chapter: u32,
    pub chapter_to_chunk: HashMap<u32,u32>,
    pub duration: f32
}
impl BookStatus{
    pub fn new(name:&str, base_path:&str ,book:BookData, json_file:&str)->BookStatus{
        BookStatus{
            name: name.to_owned(),
            path: format!("{}/{}",base_path,book.path),
            chapter: book.current_chapter,
            chunk: book.current_chunk,
            chapter_to_chunk: book.chapter_to_chunk.clone(),
            time: book.current_time,
            initial_chapter: book.initial_chapter,
            json: json_file.to_owned(),
            max_chapter: book.max_chapter,
            duration: book.duration
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioChunkResult {
    pub data: Vec<u8>,
    pub place:String,
    pub reached_end: bool,
}


#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub username: String,
    pub password_hash: String,
    pub refresh_token: String,  // store valid refresh tokens
}


#[derive(Serialize, Deserialize)]
pub struct LoginRecord {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshRecord{
    pub username: String,
    pub refresh_token: String
}


#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // username or user id
    pub iat: usize,       // issued at (seconds since epoch)
    pub exp: usize,       // expiration
}

#[derive(Deserialize)]
pub struct InitQuery {
    pub name: String,
    #[serde(rename = "type")]
    pub book_type: String,
}


#[derive(Deserialize)]
pub struct AudioQuery {
    pub chunk: u32,
}



#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookKey {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct ChunkCursor {
    pub chapter: u32,
    pub chunk: u32,
    pub chapter_to_chunk:HashMap<u32,u32>,
    pub max_chapter: u32
}

pub struct AudioBuffer {
    pub book: Option<BookKey>,
    pub chunks: VecDeque<AudioChunkResult>,
    pub min_size: usize,
    pub max_size: usize,
}

impl AudioBuffer {
    pub fn new(min_size:usize, max_size: usize) -> Self {
        Self {
            book: None,
            chunks: VecDeque::new(),
            max_size,
            min_size: min_size
        }
    }

    pub fn clear(&mut self, book: BookKey) {
        self.book = Some(book);
        self.chunks.clear();
    }

    pub fn push(&mut self, chunk: AudioChunkResult) {
        if self.chunks.len() >= self.max_size {
            self.chunks.pop_front();
        }
        self.chunks.push_back(chunk);
    }

    pub fn pop(&mut self) -> Option<AudioChunkResult> {
        self.chunks.pop_front()
    }
    pub fn trim_until(&mut self, chapter: u32, chunk: u32) {
        while let Some(front) = self.chunks.front() {
            let (ch, ck) = parse_place(&front.place);
            if ch == chapter && ck == chunk {
                break;
            }
            self.chunks.pop_front();
        }
    }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub chapter: u32,
    pub chunk: u32,
}

impl Position {
    pub fn from(cursor:&ChunkCursor)->Position{
        Position {
            chapter: cursor.chapter,
            chunk: cursor.chunk,
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.chapter.cmp(&other.chapter) {
            std::cmp::Ordering::Equal => self.chunk.cmp(&other.chunk),
            o => o,
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_place(place: &str) -> (u32, u32) {
    let mut parts = place.split(',');

    let chapter = parts
        .next()
        .expect("place missing chapter")
        .parse::<u32>()
        .expect("invalid chapter in place");

    let chunk = parts
        .next()
        .expect("place missing chunk")
        .parse::<u32>()
        .expect("invalid chunk in place");

    assert!(
        parts.next().is_none(),
        "place contains extra components"
    );

    (chapter, chunk)
}