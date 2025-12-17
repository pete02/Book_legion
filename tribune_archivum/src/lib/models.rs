use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book{
    pub path: String,
    pub initial_chapter: u32,
    pub duration: f32,
    pub current_chunk: u32,
    pub current_chapter: u32,
    pub current_time: f64,
    pub chapter_to_chunk: HashMap<u32,u32>,
    pub max_chapter: u32
}