use crate::models::*;
use crate::db_handlers::*;
use std::fs;


pub fn update_progress(status: &BookStatus, map:&AudioMap)->Result<(),String>{
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

    let true_time=map.get((status.chapter as usize, status.chunk as usize));
    match true_time {
        None=>{
            book.current_chunk=status.chunk;
            book.current_chapter=status.chapter;
            book.current_time=status.time;
        },
        Some(time)=>{
            book.current_chunk=status.chunk;
            book.current_chapter=status.chapter;
            book.current_time=time.start_time;
        }
    }

    
    let data = serde_json::to_string_pretty(&books).map_err(|_| "cannot turn books into json")?;
    fs::write(&status.json, data).map_err(|_| "error in writing to library manifest")?;
    Ok(())
}