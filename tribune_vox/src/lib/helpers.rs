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
    writer: hound::WavWriter<std::fs::File>,
    audiomap:AudioMap ,
    ip: &str,
    page: usize,
) -> AudioContext {
    AudioContext {
        writer:writer,
        map: audiomap,
        max_pages: 0,
        current_time: 0.0,
        initial_page:page,
        current_page: page,
        timer: Instant::now(),
        server_ip: ip.to_owned(),
        current_chunk: 0,
        page_to_chunk: HashMap::new(),
    }
}

pub fn generate_initial_page(page: usize, epub: &mut EpubDoc<BufReader<File>> )->Result<usize, Box<dyn std::error::Error>>{
     match get_start_index(epub) {
        Ok(i)=>Ok(i),
            Err(_)=>{
                if page==0{
                    Err("EPUB Toc corrupted, please set the initial page".into())
                }else{
                    Ok(page)
                }
            }
        }
}

pub fn create_book_struct(path:&str,ctx:&AudioContext,)->Book{
    Book{
        path: path.to_owned(),
        initial_page: ctx.initial_page,
        current_chunk: 0,
        current_page: ctx.initial_page,
        duration: ctx.current_time,
        max_page: ctx.max_pages,
        page_to_chunk: ctx.page_to_chunk.clone()
    }
}


pub fn print_progress(ctx: &mut AudioContext,length:usize){
    let secs=(Instant::now()-ctx.timer).as_secs_f32();
    let page = ctx.current_page.saturating_sub(ctx.initial_page) + 1;
    let max_page=ctx.max_pages.saturating_sub(ctx.initial_page) + 1;
    print!("\rchapter: {}/{} | chunk: {}/{} | last chunk: {:.2}s             ",
        page,
        max_page,
        ctx.current_chunk,
        length,
        secs);
    ctx.timer=Instant::now();
    std::io::stdout().flush().unwrap();
}

pub fn check_safety(options: &AudiobookOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (epub, jpg, mp3, json) = create_paths(options);
    let exists = |p: &str| std::path::Path::new(p).exists();

    match () {
        _ if !exists(&epub) => Err(format!("missing epub: {epub}"))?,
        _ if !exists(&jpg) => Err(format!("missing jpg: {jpg}"))?,
        _ if exists(&mp3) && !options.overwrite => Err("mp3 would be overwritten")?,
        _ if exists(&json) && !options.overwrite => Err("json would be overwritten")?,
        _ => Ok(()),
    }
}


pub fn update_context(ctx: &mut AudioContext, time: f32) {
    let audio=AudioMapEntry{
        page_number:ctx.current_page,
        chunk_number:ctx.current_chunk,
        start_time:ctx.current_time,
        duration:time
    };

    let entry_option=ctx.map.get_mut((ctx.current_page,ctx.current_chunk));
    if let Some(entry)=entry_option{
        entry.page_number=ctx.current_page;
        entry.chunk_number=ctx.current_chunk;
        entry.start_time=ctx.current_time;
        entry.duration=time;
    }else{
        ctx.map.insert((ctx.current_page,ctx.current_chunk), audio);
    }

    ctx.current_time += time;
}

pub 
fn save_book_to_books_json(book:Book,name:&str, path:&str)->Result<(),Box<dyn std::error::Error>>{
    let content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let mut books: HashMap<String, Book> = serde_json::from_str(&content).unwrap_or_default();
    books.insert(name.to_string(), book);

    let data = serde_json::to_string(&books).map_err(|_| "cannot turn books into json")?;
    
    let tmp_path = path.to_owned() + ".tmp";
    fs::write(&tmp_path, data)?;
    fs::rename(&tmp_path, path)?;

    Ok(())   
}


pub fn save_audio_map_json(json_path: &str, map: &AudioMap)->Result<(),Box<dyn std::error::Error>>{
    serde_json::to_writer_pretty(File::create(json_path)?, map)?;
    Ok(())
}