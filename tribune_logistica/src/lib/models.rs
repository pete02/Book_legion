use std::collections::HashMap;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookData {
    pub path: String,
    pub initial_chapter: u32,
    pub duration: f32,
    pub current_chunk: u32,
    pub current_chapter: u32,
    pub current_time: f64,
    pub chapter_to_chunk: HashMap<u32,u32>,
    pub max_chapter: u32
}

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct BookStatus {
    pub name:String,
    pub path:String,
    pub chapter: u32,
    pub initial_chapter: u32,
    pub time: f64,
    pub chunk: u32,
    pub json: String,
    pub max_chapter: u32,
    pub chapter_to_chunk: HashMap<u32,u32>,
    pub duration: f32
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioChunkResult {
    pub data: Vec<u8>,
    pub reached_end: bool,
}
