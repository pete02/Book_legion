use std::fs::File;
use std::path::Path;
use serde_json::from_reader;
use std::process::Command;
use std::fs;


use crate::{AudiobookOptions, models::{AudioMap, ChapterAudioMap}};


pub fn load_global_audio_map_strict(
    name: &str,
    initial_chapter: usize,
    end_chapter: usize,
    options:&AudiobookOptions
) -> Result<AudioMap, Box<dyn std::error::Error>> {

    let mut global_map = AudioMap::new(name.to_string());
    for chapter in initial_chapter..=end_chapter {
        let chapter_json_path = format!("./temp/chapter_{:03}.json", chapter);
        let path = Path::new(&chapter_json_path);

        if !path.exists(){
            if !options.debug{
                return Err(format!("Missing chapter JSON file: {}", chapter_json_path).into());
            }
        }else{
            let file = File::open(path)?;
            let chapter_map: ChapterAudioMap = from_reader(file)?;

            for (chunk_str, entry) in chapter_map.map {
                let chunk_num = chunk_str.parse::<usize>().map_err(|_| {
                    format!("Invalid chunk key '{}' in chapter {}", chunk_str, chapter)
                })?;
                global_map.insert((chapter, chunk_num), entry);
            }
        }
    }

    Ok(global_map)
}


pub fn format_audiobook_from_chapters(
    temp_dir: &str,
    initial_chapter: usize,
    end_chapter: usize,
    cover_path: &str,
    output_path: &str,
    options:&AudiobookOptions
) -> Result<(), Box<dyn std::error::Error>> {
    let list_path = format!("{}/chapters.txt", temp_dir);

    let list_file_content =    if !options.debug{
        (initial_chapter..=end_chapter)
        .map(|ch| format!("file '{}'", format!("{}/chapter_{:03}.wav", temp_dir, ch)))
        .collect::<Vec<_>>()
        .join("\n")
    }else{
        format!("file '{}'", format!("chapter_{:03}.wav", initial_chapter))
    };

    fs::write(&list_path, list_file_content)?;

    // First, concatenate all WAV files into a single temporary WAV
    let temp_wav = format!("{}/final_temp.wav", temp_dir);
    let status = Command::new("ffmpeg")
        .args(&[
            "-f", "concat",
            "-safe", "0",
            "-i", &list_path,
            "-c", "copy",
            &temp_wav,
        ])
        .status()?;

    if !status.success() {
        return Err(format!("ffmpeg failed concatenating WAVs, code: {:?}", status.code()).into());
    }

    // Then, convert to MP3 with cover and metadata
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", &temp_wav,
            "-i", cover_path,
            "-map", "0:a",
            "-map", "1:v",
            "-c:a", "libmp3lame",
            "-q:a", "2",
            "-id3v2_version", "3",
            "-metadata", &format!("artist={}", options.author),
            "-metadata", &format!("album={}", options.name),
            "-metadata:s:v", "title=Album cover",
            "-metadata:s:v", "comment=Cover",
            output_path,
        ])
        .status()?;

    if !status.success() {
        return Err(format!("ffmpeg failed creating final audiobook, code: {:?}", status.code()).into());
    }

    // Cleanup temporary files
    if !options.debug{
        fs::remove_file(&temp_wav)?;
        fs::remove_file(&list_path)?;
    }

    Ok(())
}