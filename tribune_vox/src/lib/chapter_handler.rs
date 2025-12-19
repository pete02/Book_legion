use epub::doc::EpubDoc;

use std::fs::File;
use std::io::BufReader;


use crate::epub_processing::*;

use crate::audio_processing::*;

use crate::models::*;

use crate::helpers::*;



pub struct AudiobookOptions{
    pub name: String,
    pub author: String,
    pub ip: String,
    pub overwrite: bool,
    pub debug: bool,
    pub initial: usize
}


pub fn handle_chapter(
    epub: &mut EpubDoc<BufReader<File>>, 
    ctx: &mut AudioContext,
    chapter:usize,
    tempdir:&str,
    debug:bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("start chapter: {}", chapter);
    epub.set_current_chapter(chapter);
    ctx.current_chapter = chapter;
    ctx.current_chunk = 0;
    ctx.current_time=0.0;
    let mut chapter_map = ChapterAudioMap::new(ctx.current_chapter);
    // Create per-chapter WAV writer
    let chapter_wav_path = format!("{}/chapter_{:03}.wav", tempdir, ctx.current_chapter);
    let mut writer = create_writer(&chapter_wav_path, &ctx.server_ip)?;

    let text = get_clean_chapter(epub)?;
    let chunks: Vec<&str> = text.split('\n').collect();
    let length = chunks.len();
    if debug{
        println!("start chunk processing");
    }

    for chunk in chunks {
        ctx.current_chunk += 1;
        print_progress(ctx, length);
        process_chunk(chunk, &mut chapter_map,&mut writer, ctx, debug)?;
    }

    
    let chapter_map_path = format!("{}/chapter_{:03}.json",tempdir, ctx.current_chapter);
    chapter_map.max_chunk=ctx.current_chunk;
    save_chapter_map_json(&chapter_map_path, &chapter_map)?;
    writer.finalize()?;

    Ok(())
}


fn process_chunk(
    chunk: &str,
    map:&mut ChapterAudioMap,
    writer: &mut hound::WavWriter<std::fs::File>,
    ctx: &mut AudioContext,
    debug:bool
) -> Result<(), Box<dyn std::error::Error>> {

    wait_for_allowed_time(debug);
    if debug{
        println!("chunk processing");
    }
    if chunk.is_empty() {
        return Ok(());
    }

    let text = clean_html(chunk)?;
    let wav_bytes = text_to_wav(&text, "sofia", &ctx.server_ip)?;
    let mut reader = create_reader(wav_bytes)?;
    let duration = write_samples_to_wav(&mut reader, writer)?;
    update_context(ctx, duration, map);

    Ok(())
}
