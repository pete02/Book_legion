use crate::AudiobookOptions;
use crate::models::*;
use std::collections::HashMap;
use std::fs::{self,File};
use std::time::Instant;
use std::io::Write;
use std::io::BufReader;
use crate::chapter_finding::get_start_index;
use epub::doc::EpubDoc;


pub fn create_paths(options: &AudiobookOptions)->(String,String,String,String){
    let name=options.name.clone();
    let name_lower=name.to_lowercase();

    let epub=format!("data/{}/{}.epub",name_lower,name_lower);
    let jpg=format!("data/{}/{}.jpg",name_lower,name_lower);

    let mp3=format!("data/{}/{}.mp3",name_lower,name_lower);
    let json=format!("data/{}/{}.json",name_lower,name_lower);

    (epub,jpg,mp3,json)
}

pub fn create_ctx_struct(
    audiomap:AudioMap ,
    ip: &str,
    chapter: usize,
) -> AudioContext {
    AudioContext {
        map: audiomap,
        max_chapters: 0,
        current_time: 0.0,
        initial_chapter:chapter,
        current_chapter: chapter,
        timer: Instant::now(),
        server_ip: ip.to_owned(),
        current_chunk: 0,
        chapter_to_chunk: HashMap::new(),
    }
}

pub fn generate_initial_chapter(chapter: usize, epub: &mut EpubDoc<BufReader<File>> )->Result<usize, Box<dyn std::error::Error>>{
     match get_start_index(epub) {
        Ok(i)=>Ok(i),
            Err(_)=>{
                if chapter==0{
                    Err("EPUB Toc corrupted, please set the initial chapter".into())
                }else{
                    Ok(chapter)
                }
            }
        }
}

pub fn create_book_struct(path:&str,ctx:&AudioContext,)->Book{
    Book{
        path: path.to_owned(),
        initial_chapter: ctx.initial_chapter,
        current_chunk: 1,
        current_time: 0.0,
        current_chapter: ctx.initial_chapter,
        duration: ctx.current_time,
        max_chapter: ctx.max_chapters,
        chapter_to_chunk: ctx.chapter_to_chunk.clone()
    }
}


pub fn print_progress(ctx: &mut AudioContext,length:usize){
    let secs=(Instant::now()-ctx.timer).as_secs_f32();
    let chapter = ctx.current_chapter.saturating_sub(ctx.initial_chapter) + 1;
    let max_chapter=ctx.max_chapters.saturating_sub(ctx.initial_chapter) + 1;
    print!("\rchapter: {}/{} | chunk: {}/{} | last chunk: {:.2}s             ",
        chapter,
        max_chapter,
        ctx.current_chunk,
        length,
        secs);
    ctx.timer=Instant::now();
    std::io::stdout().flush().unwrap();
}

pub fn check_safety(options: &AudiobookOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (epub, _, mp3, json) = create_paths(options);
    let exists = |p: &str| std::path::Path::new(p).exists();

    match () {
        _ if !exists(&epub) => Err(format!("missing epub: {epub}"))?,
        _ if exists(&mp3) && !options.overwrite => Err("mp3 would be overwritten")?,
        _ if exists(&json) && !options.overwrite => Err("json would be overwritten")?,
        _ => Ok(()),
    }
}


pub fn update_context(ctx: &mut AudioContext, time: f32, map:&mut ChapterAudioMap) {
    let audio=AudioMapEntry{
        chapter_number:ctx.current_chapter,
        chunk_number:ctx.current_chunk,
        start_time:ctx.current_time,
        duration:time
    };

    let entry_option=map.map.get_mut(&ctx.current_chunk.to_string());
    if let Some(entry)=entry_option{
        entry.chapter_number=ctx.current_chapter;
        entry.chunk_number=ctx.current_chunk;
        entry.start_time=ctx.current_time;
        entry.duration=time;
    }else{
        map.map.insert(ctx.current_chunk.to_string(), audio);
    }

    ctx.current_time += time;
}

pub 
fn save_book_to_books_json(book:Book,name:&str, json_path:&str)->Result<(),Box<dyn std::error::Error>>{
    let content = fs::read_to_string(json_path).unwrap_or_else(|_| "{}".to_string());
    let mut books: HashMap<String, Book> = serde_json::from_str(&content).unwrap_or_default();
    books.insert(name.to_string(), book);

    let data = serde_json::to_string(&books).map_err(|_| "cannot turn books into json")?;
    
    let tmp_path = json_path.to_owned() + ".tmp";
    fs::write(&tmp_path, data)?;
    fs::rename(&tmp_path, json_path)?;

    Ok(())   
}


pub fn save_audio_map_json(json_path: &str, map: &AudioMap)->Result<(),Box<dyn std::error::Error>>{
    serde_json::to_writer_pretty(File::create(json_path)?, map)?;
    Ok(())
}



pub fn save_chapter_map_json(json_path: &str, map: &ChapterAudioMap)->Result<(),Box<dyn std::error::Error>>{
    serde_json::to_writer_pretty(File::create(json_path)?, map)?;
    Ok(())
}




use chrono::{Local, Timelike};
use std::thread::sleep;
use std::time::Duration;

pub fn wait_for_allowed_time(debug:bool){
    if debug{
        return;
    }

    let now=Local::now();
    let h=now.hour();

    if h>=8 && h< 22 {
        return;
    }

    let sleep_time=if h<8{
        let Some(time)=now.date_naive().and_hms_opt(8, 0, 0) else {return;};
        time
    }else{
        let Some(time)=(now.date_naive()+chrono::Duration::days(1)).and_hms_opt(8, 0, 0) else {return;};
        time
    };

    println!("outside work hours, will sleep until: {:?}",sleep_time);
    let sleep_secs = sleep_time.signed_duration_since(now.naive_local()).num_seconds().max(1);
    sleep(Duration::from_secs(sleep_secs as u64));
}