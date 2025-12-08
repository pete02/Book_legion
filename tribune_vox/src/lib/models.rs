use std::collections::HashMap;

use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
pub struct AudioMapEntry {
    pub chapter_number: usize,
    pub chunk_number:usize,
    pub start_time: f32, // seconds
    pub duration: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book{
    pub path: String,
    pub initial_chapter: usize,
    pub duration: f32,
    pub current_chunk: usize,
    pub current_chapter: usize,
    pub current_time: f64,
    pub chapter_to_chunk: HashMap<usize,usize>,
    pub max_chapter: usize
}


#[derive(Serialize, Deserialize)]
pub struct AudioMap {
    pub name:String,
    map: HashMap<String, AudioMapEntry>,
}
impl AudioMap {
    pub fn new(n:String) -> Self {
        Self {
            name:n,
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: (usize, usize), value: AudioMapEntry) {
        let key_str = format!("{},{}", key.0, key.1);
        self.map.insert(key_str, value);
    }

    pub fn get(&self, key: (usize, usize)) -> Option<&AudioMapEntry> {
        let key_str = format!("{},{}", key.0, key.1);
        self.map.get(&key_str)
    }

    pub fn get_mut(&mut self, key: (usize, usize)) -> Option<&mut AudioMapEntry> {
        let key_str = format!("{},{}", key.0, key.1);
        self.map.get_mut(&key_str)
    }

    pub fn remove(&mut self, key: (usize, usize)) -> Option<AudioMapEntry> {
        let key_str = format!("{},{}", key.0, key.1);
        self.map.remove(&key_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &AudioMapEntry)> {
        self.map.iter()
    }
}



pub struct AudioContext{
    pub writer: hound::WavWriter<std::fs::File>,
    pub max_chapters: usize,
    pub timer: std::time::Instant,
    pub map: AudioMap,
    pub current_time: f32, // running total of audio length
    pub initial_chapter:usize,
    pub current_chapter: usize,
    pub server_ip:String,
    pub current_chunk: usize,
    pub chapter_to_chunk: HashMap<usize,usize>
}
