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
    pub name: Option<String>,
    pub user: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expiry: Option<chrono::DateTime<chrono::Utc>>, // track expiry
}
impl GlobalState{
    pub fn new()->GlobalState{
        return GlobalState { book: None, name: None, user: None, access_token:None, refresh_token:None, token_expiry: None };
    }
}


#[derive(Serialize, Deserialize)]
pub struct RefreshRecord{
    pub username: String,
    pub refresh_token: String
}
impl RefreshRecord{
    pub fn new(user:String, refresh:String)->RefreshRecord{
        return RefreshRecord { username: user, refresh_token: refresh };
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tokens{
    pub access_token: String,
    pub refresh_token:String
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioChunkResult {
    pub data: Vec<u8>,
    pub place:String,
    pub reached_end: bool,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonPayload{
    pub chunks: Vec<AudioChunkResult>
}
