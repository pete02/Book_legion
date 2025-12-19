use std::collections::HashMap;
use std::io::Read;

use epub::doc::EpubDoc;
mod verify;
use crate::generate_book::get_clean_chapter;
use crate::models::Book;
use crate::verify::{get_first_chapter, verify_toc};

mod create;
use crate::create::{patch_epub, scan_spine_for_headings};

mod generate_book;
mod models;


pub fn check_epub(path:&str, book_id:&str, name:&str)->Result<(),Box<dyn std::error::Error>>{
    println!("path: {}",path);
    match verify_epub(path) {
     Ok(index)=>{
        println!("no patch needed");
        generate_book_instance(path, index,name)?;
    },
     Err(_)=>{
        generate_toc(path, book_id)?;
        let index=verify_epub(path)?;
        generate_book_instance(path, index,name)?;
        println!("book patched");
        }
    }
    Ok(())
}

fn generate_toc(path:&str, book_id:&str)->Result<(),Box<dyn std::error::Error>>{
    println!("toch generation");
    let mut epub=EpubDoc::new(path)?;
    let toc=scan_spine_for_headings(&mut epub);
    patch_epub( path.to_owned(),toc,book_id)?;

    Ok(())

}

fn verify_epub(path:&str)->Result<u32,Box< dyn std::error::Error>>{
    let mut epub=EpubDoc::new(path)?;
    verify_toc(&mut epub)?;
    get_first_chapter(&mut epub)
}
use std::fs::File;
fn generate_book_instance(path:&str, init:u32, name:&str)->Result<(),Box< dyn std::error::Error>> {
    let mut epub=EpubDoc::new(path)?;
    let max=epub.get_num_chapters() as u32;
    println!("init: {}", init);
    let mut book=Book{
        path:name.to_owned(),
        initial_chapter:init,
        duration:0.0,
        current_chunk:1,
        current_chapter: init,
        current_time:0.0,
        chapter_to_chunk: HashMap::new(),
        max_chapter: max
    };
    for i in init..max{
        epub.set_current_chapter(i as usize);
        let txt=get_clean_chapter(&mut epub)?;
        let vec:Vec<&str>=txt.split('\n').collect();
        book.chapter_to_chunk.insert(i, vec.len() as u32);
    }
    println!("save book");
    save_book(book)?;
    Ok(())
}

use std::path::Path;

fn save_book(book: Book) -> Result<(), Box<dyn std::error::Error>> {
    // Read the existing books
    let mut file = File::open("./book.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut books: HashMap<String, Book> = serde_json::from_str(&contents)?;

    // Use the file stem as the key
    let path = Path::new(&book.path);
    let key = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&book.path)  // fallback if UTF-8 fails
        .to_string();

    // Insert the new book
    books.insert(key, book);

    // Write back
    let mut file = File::create("book.json")?;
    serde_json::to_writer_pretty(&mut file, &books)?;

    Ok(())
}