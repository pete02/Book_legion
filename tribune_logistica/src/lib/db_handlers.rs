use std::collections::HashMap;
use std::fs;
use crate::models::*;


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




pub fn prefix()->String{
    "./data".to_string()
}
