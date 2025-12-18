use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::json;

use crate::models::*;
fn error(msg: &str) -> serde_json::Value {
    json!({ "status": msg, "chapter": -1, "chunk": -1 })
}


pub fn load_books(path: &str) -> Result<HashMap<String, BookData>,Box<dyn std::error::Error>> {
    let content = get_library_manifest(path)?;
    let books: HashMap<String, BookData> = serde_json::from_str(&content)?;
    Ok(books)
}


pub fn load_book(status:&BookStatus) ->Result<BookData, Box<dyn std::error::Error>>{
    let books=load_books(&status.json)?;
    let Some(book)=books.get(&status.name) else{return Err("No such book in library".into());};
    Ok(book.clone())
}

pub fn get_audiomap(status:&BookStatus) -> Result<AudioMap,Box<dyn std::error::Error>> {
    let map=format!("{}/{}.json",status.path,status.name);
    let content = fs::read_to_string(map)?;
    let book: AudioMap = serde_json::from_str(&content)?;
    Ok(book)
}

pub fn get_library_manifest(path: &str)->Result<String,Box<dyn std::error::Error>>{
    let content=fs::read_to_string(path)?;
    Ok(content)
}

pub fn init_book(name: &str, book_type: &str, json_path: &str, base_path:&str) -> Result<BookStatus,serde_json::Value> {
    if book_type != "audio" && book_type != "text" {
        return Err(error("incorrect format"));
    }
    let books = load_books(&json_path).map_err(|_|error(&format!("missing library manifest: {}",&json_path)))?;
    let book=books.get(name).ok_or(error("not in library"))?;


    if book_type=="audio" && !Path::exists(Path::new(format!("{}/{}/{}.mp3",base_path,&book.path,name).as_str())){
        return Err(error("missing audiobook"));
    }

    if book_type=="text" && !Path::exists(Path::new(format!("{}/{}/{}.epub",base_path,&book.path,name).as_str())){
        return Err(error("missing book"));
    }

    Ok(BookStatus::new(name, base_path, book.clone(), json_path))
}


pub fn prefix()->String{
    "./data".to_string()
}
