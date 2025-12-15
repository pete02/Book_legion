use epub::doc::EpubDoc;
use std::fs;
use std::fs::File;
use std::io::BufReader;


pub mod epub_processing;
pub mod audio_processing;
pub mod finalizer;

pub mod models;
use crate::epub_processing::extract_cover;
use crate::finalizer::format_audiobook_from_chapters;
use crate::finalizer::load_global_audio_map_strict;
use crate::finalizer::save_to_book_json;

use self::models::*;

pub mod helpers;
use self::helpers::*;

pub mod chapter_finding;

pub mod chapter_handler;
use self::chapter_handler::handle_chapter;

pub struct AudiobookOptions{
    pub name: String,
    pub author: String,
    pub ip: String,
    pub overwrite: bool,
    pub debug: bool,
    pub initial: usize
}


pub fn make_audiobook(options: &AudiobookOptions)->Result<(),Box<dyn std::error::Error>>{
    let tempdir="./temp";

    check_safety(&options)?;

    let (path,_,_,_)=create_paths(options);
    println!("load epub from {}",path);
    let mut epub=EpubDoc::new(path)?;
    let initial_chapter=generate_initial_chapter(options.initial, &mut epub)?;
    let max_chapter=epub.get_num_chapters();

    let mut ctx = create_ctx_struct(
        AudioMap::new(options.name.clone()),
        &options.ip,
        initial_chapter);

    let missing_chapter=missing_chapters(initial_chapter,max_chapter , tempdir, options);
    println!("start creating audiobook");
    create_audiobook(&mut ctx, options,epub,missing_chapter, tempdir)?;
    println!("Saved audiobook"); 
    finalizer(initial_chapter, max_chapter, options, tempdir, &mut ctx)?;
    Ok(())
}

fn create_audiobook( ctx: &mut AudioContext,options: &AudiobookOptions, mut epub:EpubDoc<BufReader<File>>, missing_chapters:Vec<usize>, tempdir:&str) -> Result<(), Box<dyn std::error::Error>> {
    ctx.max_chapters=epub.get_num_chapters();
    std::fs::create_dir_all(tempdir)?;
    println!("missing chapters: {:?}",missing_chapters );
    if missing_chapters.len() ==0 {
        return Ok(());
    }

    if options.debug{
        handle_chapter(&mut epub, ctx, missing_chapters[0], tempdir, options.debug)?;
    } else {
        for chapter in missing_chapters{
            handle_chapter(&mut epub, ctx, chapter, tempdir, options.debug)?;
        }
    }
    Ok(())
}



use std::path::Path;
pub fn missing_chapters(initial_chapter:usize, max_chapters: usize, tempdir:&str, options: &AudiobookOptions) -> Vec<usize> {
    let mut missing = Vec::new();
    let temp_dir = Path::new(tempdir);

    if !options.debug{
        for chapter in initial_chapter..=max_chapters {
            let wav_path = temp_dir.join(format!("chapter_{:03}.wav", chapter));
            let json_path = temp_dir.join(format!("chapter_{:03}.json", chapter));

            if !wav_path.exists() || !json_path.exists() {
                missing.push(chapter);
            }
        }
    }else{
        let wav_path = temp_dir.join(format!("chapter_{:03}.wav", initial_chapter));
        let json_path = temp_dir.join(format!("chapter_{:03}.json", initial_chapter));

        if !wav_path.exists() || !json_path.exists() {
            missing.push(initial_chapter);
        }
    }

    missing
}


fn finalizer(initial_chapter:usize, max_chapters: usize, options: &AudiobookOptions, tempdir:&str, ctx: &mut AudioContext) -> Result<(),Box<dyn std::error::Error>>{
    let missing=missing_chapters(initial_chapter, max_chapters, tempdir, options);
    if missing.len() > 0 && !options.debug{
        return Err("book is not finalized".into());
    }
    let (epub,jpg,mp3,json)=create_paths(options);
    let map=load_global_audio_map_strict(&options.name, initial_chapter, max_chapters, options)?;
    
    save_audio_map_json(&json, &map)?;
    extract_cover(&epub, &jpg)?;
    save_to_book_json(options, ctx, "data/books.json")?;

    format_audiobook_from_chapters(tempdir,
    initial_chapter,
    max_chapters,
    &jpg,
    &mp3,
        options
    )?;


    if !options.debug{
        fs::remove_file(jpg)?;
        for chapter in initial_chapter..=max_chapters{
            let temp_wav=format!("{}/chapter_{:03}.wav", tempdir, chapter);
            let temp_json =format!("{}/chapter_{:03}.json", tempdir, chapter);
            fs::remove_file(temp_wav)?;
            fs::remove_file(temp_json)?;
        }
    }

    Ok(())
}