use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AudioMapEntry {
    pub chapter_number: usize,
    pub chunk_number: usize,
    pub start_time: f32,
    pub duration: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AudioMap {
    pub name: String,
    pub map: HashMap<String, AudioMapEntry>,
}

impl AudioMap {
    pub fn fix_start_times(&mut self) {
        // Collect entries and sort by chapter_number then chunk_number
        let mut entries: Vec<(&String, &mut AudioMapEntry)> = self.map.iter_mut().collect();
        entries.sort_by(|a, b| {
            a.1.chapter_number
                .cmp(&b.1.chapter_number)
                .then(a.1.chunk_number.cmp(&b.1.chunk_number))
        });

        // Adjust start times cumulatively
        let mut cumulative_time: f32 = 0.0;
        for (_key, entry) in entries {
            entry.start_time = cumulative_time;
            cumulative_time += entry.duration;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the input file
    let file = File::open("eskau.json")?;
    let reader = BufReader::new(file);

    // Deserialize JSON into AudioMap
    let mut audio_map: AudioMap = serde_json::from_reader(reader)?;

    // Fix cumulative start_times
    audio_map.fix_start_times();

    // Write fixed JSON to a new file
    let output_file = File::create("eskau_fixed.json")?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &audio_map)?;

    println!("Start times fixed and saved to eskau_fixed.json");

    Ok(())
}
