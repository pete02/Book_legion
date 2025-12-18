use axum::http::status;
use serde_json::json;
use zip::ZipArchive;
use std::{collections::HashMap, path::Path};
use std::fs::{self,File};
use std::io::Read;
use epub::doc::EpubDoc;
use crate::models::*;

use crate::db_handlers::*;

fn error(msg: &str) -> serde_json::Value {
    json!({ "status": msg, "chapter": -1, "chunk": -1 })
}

pub fn init_book(name: &str, book_type: &str, books_path: &str) -> Result<BookStatus,serde_json::Value> {
    if book_type != "audio" && book_type != "text" {
        return Err(error("incorrect format"));
    }
    let books = load_books(&books_path).map_err(|_|error(&format!("missing library manifest: {}",&books_path)))?;
    let book=books.get(name).ok_or(error("not in library"))?;


    if book_type=="audio" && !Path::exists(Path::new(format!("{}/{}/{}.mp3",prefix(),&book.path,name).as_str())){
        return Err(error("missing audiobook"));
    }

    if book_type=="text" && !Path::exists(Path::new(format!("{}/{}/{}.epub",prefix(),&book.path,name).as_str())){
        return Err(error("missing book"));
    }

    Ok(BookStatus{
        name: name.to_owned(),
        path: format!("{}/{}",prefix(),book.path),
        chapter: book.current_chapter,
        chunk: book.current_chunk,
        chapter_to_chunk: book.chapter_to_chunk.clone(),
        time: book.current_time,
        initial_chapter: book.initial_chapter,
        json: books_path.to_owned(),
        max_chapter: book.max_chapter,
        duration: book.duration
    })
}

pub fn get_chapter(status:&BookStatus)->Result<String,String>{
    let path=format!("{}/{}.epub", status.path,status.name);
    let mut book=EpubDoc::new(&path).map_err(|_|"Failed to open EPUB in the path".to_owned())?;
    
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

pub fn extract_css(path:&str)-> Result<String, Box<dyn std::error::Error>>{
    let mut css="".to_owned();
    let files=extract_files(path, vec![".css"])?;
    for file in files{
        let txt=String::from_utf8(file.1)?;
        css+=&txt;
    }
    Ok(css)
}

pub fn extract_files(path: &str, file_types:Vec<&str>)->Result<HashMap<String,Vec<u8>>,Box<dyn std::error::Error>>{
    let book=File::open(path)?;
    let mut archive=ZipArchive::new(book)?;

    let mut map=HashMap::new();

    for i in 0 ..archive.len(){
        let mut file=archive.by_index(i)?;
        let name=file.name().to_owned();
        if file_types.iter().any(|ft|name.contains(ft)){
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            map.insert(name, data);
        }
    }
    Ok(map)
}