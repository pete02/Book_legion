
use std::cmp::min;
use std::path::Path;
use std::fs::{self,File};
use std::io::Read;
use crate::models::*;

use crate::db_handlers;

pub fn get_audio_chunks(book_option:Option<&BookStatus>, advance:u32, base:&str)->Result<Vec<AudioChunkResult>,Box<dyn std::error::Error>>{
    get_audio_chunks_conf(book_option, advance, base, "chunk.mp3")
}

pub fn get_audio_chunks_conf(book_option:Option<&BookStatus>, advance:u32, base:&str, output: &str)->Result<Vec<AudioChunkResult>,Box<dyn std::error::Error>>{
    let mut vec=Vec::new();
    let Some(status)=book_option.clone() else{return Err("No book".into());};
    let chapter=status.chapter as usize;
    let chunk=status.chunk;
    
    let max=status.chapter_to_chunk.get(&status.chapter).ok_or("no such chapter")?.clone();

    let end=min(chunk+advance-1, max);

    for i in chunk..=end{
        if i==max{
            vec.push(AudioChunkResult { 
                data: get_audio_chunk(status, chapter, i as usize, output, false, base)?, 
                place: format!("{},{}",chapter,i),
                reached_end:true });
        }else{
            vec.push(AudioChunkResult { 
                data: get_audio_chunk(status, chapter, i as usize, output, false, base)?, 
                place: format!("{},{}",chapter,i),
                reached_end:false });
        }
    }

    Ok(vec)
}



pub fn get_audio_chunk(status: &BookStatus, chapter:usize, chunk:usize, output: &str, keep:bool, base:&str)->Result<Vec<u8>,Box<dyn std::error::Error>>{
    let input=format!("{}/{}.mp3",&status.path,&status.name.to_lowercase());
    let path=format!("{}/{}.json",&status.path,&status.name.to_lowercase());
    let audiomap=db_handlers::get_audiomap(&status)?;
    let start: &AudioMapEntry=audiomap.get((chapter as usize,chunk as usize)).ok_or("no such starting point")?;

    slice_mp3(&input, output, start.start_time, start.start_time+start.duration, base)?;
    let mut buf = Vec::new();
    
    if std::path::Path::new(output).exists(){
        File::open(output)?.read_to_end(&mut buf)?;
        if !keep{
            fs::remove_file(output)?;
        }
    }else{
        println!("no such file");
        if !TEST{
            return Err("no output file".into());
        }
    }

    Ok(buf)
}



static TEST:bool=true;
fn slice_mp3(input: &str, output: &str, start: f32, end: f32, base: &str) -> std::io::Result<()> {

    if !TEST{

        use std::process::Command;
        use std::path::Path;

        let start_str = start.to_string();
        let end_str = end.to_string();

        if start >= end {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid audio range"));
        }

        if !Path::new(&input).exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Input file '{}' not found.", input)));
        }

        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-hide_banner",
                "-loglevel", "error",
                "-i", &input,
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
    }else{
        std::fs::write(output, vec![0u8; 100])?;
        return Ok(());
    }
}
