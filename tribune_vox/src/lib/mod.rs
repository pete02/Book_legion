use epub::doc::EpubDoc;
use std::fs;
use std::fs::File;
use std::io::BufReader;


pub mod epub_processing;
use self::epub_processing::*;

pub mod audio_processing;
use self::audio_processing::*;

pub mod models;
use self::models::*;

pub mod helpers;
use self::helpers::*;

pub mod chapter_finding;

pub struct AudiobookOptions{
    pub name: String,
    pub author: String,
    pub ip: String,
    pub overwrite: bool,
    pub debug: bool,
    pub initial: usize
}


pub fn make_audiobook(options: &AudiobookOptions)->Result<(),Box<dyn std::error::Error>>{

    check_safety(&options)?;

    let (path,_,_,_)=create_paths(options);
    println!("load epub from {}",path);
    let mut epub=EpubDoc::new(path)?;
    let chapter=generate_initial_chapter(options.initial, &mut epub)?;

    let mut ctx = create_ctx_struct(
        create_writer("final.wav",&options.ip)?, 
        AudioMap::new(options.name.clone()),
        &options.ip,
        chapter);

    println!("start creating audiobook");
    create_audiobook(&mut ctx, options,epub)?;
    save_data(ctx, options)?;
    println!("Saved audiobook"); 

    Ok(())
}

fn create_audiobook( ctx: &mut AudioContext,options: &AudiobookOptions, mut epub:EpubDoc<BufReader<File>>) -> Result<(), Box<dyn std::error::Error>> {

    epub.set_current_chapter(ctx.initial_chapter-1);

    ctx.max_chapters=epub.get_num_chapters();

    if options.debug {
        epub.go_next();
        handle_chapter(&mut epub, ctx)?;
    } else {
        while epub.go_next() {
            handle_chapter(&mut epub,  ctx)?;
        }
    }
    Ok(())
}


fn handle_chapter(epub: &mut EpubDoc<BufReader<File>>, 
    ctx:&mut AudioContext
    )-> Result<(), Box<dyn std::error::Error>> {
    let text = get_clean_chapter(epub)?;
    ctx.current_chapter = epub.get_current_chapter();
    ctx.current_chunk=0;

    let chunks:Vec<&str>=text.split('\n').collect();
    let length: usize=chunks.len();

    for chunk in chunks{
        ctx.current_chunk+=1;
        print_progress(ctx,length);
        process_chunk(&chunk, ctx)?;
    }

    ctx.chapter_to_chunk.insert(ctx.current_chapter, ctx.current_chunk);
    Ok(())
}


fn process_chunk(
    chunk: &str,
    ctx:&mut AudioContext,
) -> Result<(), Box<dyn std::error::Error>> {
    if chunk.is_empty() {
        return Ok(());
    }

    let text=clean_html(chunk)?;
    let wav_bytes = text_to_wav(&text, "sofia",&ctx.server_ip)?;
    let mut reader=create_reader(wav_bytes)?;
    let duration=write_samples_to_wav(&mut reader,&mut ctx.writer)?; 
    update_context(ctx, duration);
    Ok(())

}

fn save_data(ctx:AudioContext, options: &AudiobookOptions)->Result<(),Box<dyn std::error::Error>>{
    let (_,jpg,mp3,json)=create_paths(options);
    save_book_to_books_json(create_book_struct(&options.name, &ctx),&options.name,"data/books.json")?;
    ctx.writer.finalize()?;
    save_audio_map_json(&json, &ctx.map)?;

    if options.overwrite{
        fs::remove_dir(&mp3)?;
    }

    format_audiobook("final.wav", &jpg, &mp3, &options.name,&options.author)?; 
    Ok(())
}


