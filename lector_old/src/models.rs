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
impl BookStatus {
    pub fn reached_chapter_end(&self) -> bool {
        // 1. Check that chapter exists
        let Some(&last_chunk) = self.chapter_to_chunk.get(&self.chapter) else {
            return true;
        };

        // 2. Compare chunk
        self.chunk == last_chunk
    }

    pub fn reached_end(&self)-> bool{
        self.reached_chapter_end() && self.chapter == self.max_chapter
    }

    pub fn get_current_pos(&self)->Place{
        Place::new(self.chapter,self.chunk)
    }

    pub fn set_place(&mut self, place:Place){
        if self.max_chapter >= place.chapter && self.initial_chapter <= place.chapter{
            let Some(max_chunk)=self.chapter_to_chunk.get(&place.chapter) else {return;};
            if place.chunk >=1 && place.chunk <=*max_chunk{
                self.chapter=place.chapter;
                self.chunk=place.chunk;
            }
        }
    }

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


use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct Place {
    pub chapter: u32,
    pub chunk: u32,
}

impl Place {
    pub fn new(chapter: u32, chunk: u32) -> Self {
        Self { chapter, chunk }
    }

    pub fn parse(place: &str) -> Place {
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

        Place::new(chapter, chunk)
    }

    pub fn next(&mut self,map:&HashMap<u32,u32>)->Place{
        let Some(max_chunk)=map.get(&self.chapter)else {return *self};
        if self.chunk == *max_chunk{
            if map.keys().any(|f| *f == self.chapter+1){
                self.chapter+=1;
                self.chunk=1;
            }
        }else{
            self.chunk+=1;
        }
        return *self
    }

    pub fn prev(&mut self,map:&HashMap<u32,u32>){
        if self.chunk == 1{
            if map.keys().any(|f| *f == self.chapter-1){
                let Some(max_chunk)=map.get(&(self.chapter-1))else {return};
                self.chapter-=1;
                self.chunk=*max_chunk;
            }
        }else{
            self.chunk-=1;
        }

    }

    pub fn jump_next(&mut self, amount: i32, map:&HashMap<u32,u32>){
        for _i in 0..amount{
            self.next(map);
        }
    }
    pub fn jump_prev(&mut self, amount: i32, map:&HashMap<u32,u32>){
        for _i in 0..amount{
            self.prev(map);
        }
    }

}

impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        self.chapter == other.chapter && self.chunk == other.chunk
    }
}

impl Eq for Place {}

impl PartialOrd for Place {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Place {
    fn cmp(&self, other: &Self) -> Ordering {
        self.chapter
            .cmp(&other.chapter)
            .then_with(|| self.chunk.cmp(&other.chunk))
    }
}


