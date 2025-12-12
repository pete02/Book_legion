use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct BookStatus {
    pub name:String,
    pub path:String,
    pub chapter: u32,
    pub chunk: u32,
    pub time: f64,
    pub initial_chapter: u32,
    pub json: String,
    pub max_chapter: u32,
    pub duration: f64,
    pub chapter_to_chunk: HashMap<u32,u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ManifestEntry {
    pub chapter_to_chunk: HashMap<u32, u32>,
    pub current_chapter: u32,
    pub current_chunk: u32,
    pub current_time: f64,
    pub duration: f64,
    pub initial_chapter: u32,
    pub max_chapter: u32,
    pub path: String,
}
impl From<(String, ManifestEntry)> for BookStatus {
    fn from((name, entry): (String, ManifestEntry)) -> Self {
        BookStatus {
            name,
            path: entry.path,
            chapter: entry.current_chapter,
            chunk: entry.current_chunk,
            time: entry.current_time,
            initial_chapter: entry.initial_chapter,
            json: String::new(), // fill as needed
            max_chapter: entry.max_chapter,
            duration: entry.duration,
            chapter_to_chunk: entry.chapter_to_chunk,
        }
    }
}

pub type Manifest = HashMap<String, ManifestEntry>;


#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalState {
    pub book: Option<BookStatus>,
    pub name: Option<String>
}
impl GlobalState{
    pub fn new()->GlobalState{
        return GlobalState { book: None, name: None };
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkProgress {
    pub chapter_number: u32,
    pub chunk_number: u32,
    pub start_time: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunkmap{
    pub name: String,
    pub map: HashMap<String,ChunkProgress>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData{
    pub data:Chunkmap,
    pub status: String
}
