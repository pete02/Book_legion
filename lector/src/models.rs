use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct BookStatus {
    pub name:String,
    pub path:String,
    pub chapter: u32,
    pub chunk: u32,
    pub time: f64,
    pub json: String,
    pub max_chapter: u32,
    pub duration: f64
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalState {
    pub book: Option<BookStatus>,
}
impl GlobalState{
    pub fn new()->GlobalState{
        return GlobalState { book: None };
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
