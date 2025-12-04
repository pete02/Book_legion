use serde_json::json;
use std::{collections::HashMap, path::Path};
use std::fs::{self,File};
use std::io::Read;
use epub::doc::EpubDoc;
use crate::models::*;


fn error(msg: &str) -> serde_json::Value {
    json!({ "status": msg, "chapter": -1, "chunk": -1 })
}

fn prefix(raw_path:&str)->String{
    format!("../data/{}",raw_path)
}

pub fn load_books(path: &str) -> Result<HashMap<String, BookData>,Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let books: HashMap<String, BookData> = serde_json::from_str(&content)?;
    Ok(books)
}

pub fn get_audiomap(path: &str) -> Result<AudioMap,Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&prefix(path))?;
    let book: AudioMap = serde_json::from_str(&content)?;
    Ok(book)
}



pub fn get_library_manifest(path: &str)->Result<String,Box<dyn std::error::Error>>{
    let content=fs::read_to_string(path)?;
    Ok(content)
}

pub fn init_book(name: &str, book_type: &str, books_path: &str) -> Result<BookStatus,serde_json::Value> {
    if book_type != "audio" && book_type != "text" {
        return Err(error("incorrect format"));
    }
    let books = load_books(&books_path).map_err(|_|error(&format!("missing library manifest: {}",&books_path)))?;
    let book=books.get(name).ok_or(error("not in library"))?;


    if book_type=="audio" && !Path::exists(Path::new(format!("{}/{}.mp3",prefix(&book.path),name).as_str())){
        return Err(error("missing audiobook"));
    }

    if book_type=="text" && !Path::exists(Path::new(format!("{}/{}.epub",prefix(&book.path),name).as_str())){
        return Err(error("missing book"));
    }

    Ok(BookStatus{
        name: name.to_owned(),
        path: book.path.to_owned(),
        chapter: book.current_chapter,
        chunk: book.current_chunk,
        time: book.current_time,
        json: books_path.to_owned(),
        max_chapter: book.max_chapter,
        duration: book.duration
    })
}

pub fn update_progress(book_option:Option<BookStatus>)->Result<(),String>{

    let status=book_option.ok_or("no initialized book")?;
    let mut books= load_books(&status.json).map_err(|_|"missing manifest")?;
    let book=books.get_mut(&status.name).ok_or("not in library")?;
    if status.chapter>book.max_chapter{
        return Err("chapter overflow".into());
    }

    let max_chunks = book.chapter_to_chunk.get(&status.chapter)
    .ok_or("invalid chapter number")?;
    if &status.chunk>max_chunks{
        return Err("chunk overflow".into());
    }

    book.current_chunk=status.chunk;
    book.current_chapter=status.chapter;
    book.current_time=status.time;

    
    let data = serde_json::to_string_pretty(&books).map_err(|_| "cannot turn books into json")?;
    fs::write(&status.json, data).map_err(|_| "error in writing to library manifest")?;
    Ok(())
}

pub fn get_chapter(book_option:Option<BookStatus>)->Result<String,String>{
    let status=book_option.ok_or("no initialized book")?;
    let path=format!("{}/{}.epub",status.path,status.name);
    let mut book=EpubDoc::new(&prefix(&path)).map_err(|_|"Failed to open EPUB in the path".to_owned())?;
    
    if status.chapter as usize >book.get_num_chapters(){
        return Err("chapter too large".to_owned());
    }

    book.set_current_chapter(status.chapter as usize);
    
    if let Some((chapter_text, _)) = book.get_current_str() {
        Ok(chapter_text)
    } else {
        Err("No chapter found".into())
    }
}


pub fn get_audio_chunk(book_option:Option<&BookStatus>, chunk:u32)->Result<AudioChunkResult,Box<dyn std::error::Error>>{
    get_audio_chunk_config(book_option, chunk,false, "chunk.mp3".to_owned())
}

pub fn get_audio_chunk_config(book_option:Option<&BookStatus>, advance:u32, keep:bool, chunk_name:String)->Result<AudioChunkResult,Box<dyn std::error::Error>>{
    let mut file_name=chunk_name;
    let mut reached_end=false;

    let status=book_option.ok_or("no initialized book")?;
    let books=load_books(&status.json).map_err(|_|format!("missing manifest:{}",&status.json))?;
    let book=books.get(&status.name).ok_or("not in library")?;
    let max_chunk=book.chapter_to_chunk.get(&status.chapter).ok_or("no such chapter")?;
    let mut chunk=status.chunk+advance;

    if &chunk>max_chunk{
        chunk=max_chunk.clone();
        reached_end=true;
    }

    if status.chunk>chunk{
        return Err("current progress higher than requested".into())
    }


    let mp3_path=format!("{}/{}.mp3",status.path,status.name.to_lowercase());
    let json_path=format!("{}/{}.json",status.path,status.name.to_lowercase());
    let audiomap=get_audiomap(&json_path)?;
    
    if !keep{
        file_name=format!("chunk_{}_{}_{}.mp3", status.name, status.chapter, chunk)
    }

    let start=audiomap.get((status.chapter as usize,status.chunk as usize)).ok_or("no such starting point")?;
    let end=audiomap.get((status.chapter as  usize,chunk as usize)).ok_or("no such ending point")?;
    println!("sending: {}-{}", start.start_time, end.start_time+end.duration);
    println!("chunks: {}-{}",status.chapter, status.chunk);
    slice_mp3(&mp3_path, &file_name, start.start_time, end.start_time + end.duration)?;
    
    let mut buf = Vec::new();
    File::open(&file_name)?.read_to_end(&mut buf)?;
    
    if !keep {
        fs::remove_file(&file_name)?;
    }

    Ok(AudioChunkResult{data:buf, reached_end: reached_end})

}

use std::process::Command;
fn slice_mp3(input: &str, output: &str, start: f32, end: f32) -> std::io::Result<()> {
    let fixed=prefix(input);
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

