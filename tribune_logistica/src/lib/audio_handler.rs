
use std::cmp::min;
use std::path::Path;
use std::fs::{self,File};
use std::io::Read;
use crate::models::*;

use crate::db_handlers;



pub fn get_audio_chunks(book_option:Option<&BookStatus>, advance:u32, base:&str)->Result<Vec<AudioChunkResult>,Box<dyn std::error::Error>>{
    let mut vec=Vec::new();
    let Some(status)=book_option.clone() else{return Err("No book".into());};
    let name=&status.name;
    let chapter=status.chapter as usize;
    let chunk=status.chunk;
    
    let max=status.chapter_to_chunk.get(&status.chapter).ok_or("no such chapter")?.clone();

    let end=min(chunk+advance-1, max);

    for i in chunk..=end{
        if i==max{
            vec.push(AudioChunkResult { 
                data: get_audio_chunk(name, chapter, i as usize, "chunk.mp3", false, base)?, 
                place: format!("{},{}",chapter,i),
                reached_end:true });
        }else{
            vec.push(AudioChunkResult { 
                data: get_audio_chunk(name, chapter, i as usize, "chunk.mp3", false, base)?, 
                place: format!("{},{}",chapter,i),
                reached_end:false });
        }
    }

    Ok(vec)
}



pub fn get_audio_chunk(name:&str, chapter:usize, chunk:usize, output: &str, keep:bool, base:&str)->Result<Vec<u8>,Box<dyn std::error::Error>>{
    let input=format!("{}/{}/{}.mp3",base,name,name.to_lowercase());
    let path=format!("{}/{}/{}.json",base,name,name.to_lowercase());
    println!("get audiobook: {}",path);
    let audiomap=db_handlers::get_audiomap(&path)?;
    println!("entry");
    let start: &AudioMapEntry=audiomap.get((chapter as usize,chunk as usize)).ok_or("no such starting point")?;

    println!("slice");
    slice_mp3(&input, output, start.start_time, start.start_time+start.duration, base)?;
    println!("sliced");
    let mut buf = Vec::new();
    File::open(output)?.read_to_end(&mut buf)?;
    if !keep{
        fs::remove_file(output)?;
    }
    println!("sending ok");
    Ok(buf)
}




use std::process::Command;
#[cfg(not(test))]
fn slice_mp3(input: &str, output: &str, start: f32, end: f32, base:&str) -> std::io::Result<()> {
    let fixed=format!("{}/{}",base,input);
    let start_str=format!("{}",start);
    let end_str=format!("{}",end);
    if start>=end{
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid audio range".to_owned()));
    }

    if !Path::new(&fixed).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Input file '{}' not found.", &fixed)));
    }

    let status = Command::new("ffmpeg")
        .args([
            "-y", // overwrite output
            "-hide_banner",
            "-loglevel", "error",
            "-i", &fixed,
            "-ss", &start_str,
            "-to", &end_str,
            "-c", "copy",
            output,
        ])
        .status()?;


    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg failed to slice the file"));
    }
    Ok(())
}

#[cfg(test)]
fn slice_mp3(_input: &str, output: &str, _start: f32, _end: f32, _base:&str) -> std::io::Result<()> {
    std::fs::write(output, vec![0u8; 100])?;
    Ok(())
}