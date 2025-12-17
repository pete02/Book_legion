use std::collections::HashMap;

use epub::doc::EpubDoc;
mod verify;
use crate::generate_book::get_clean_chapter;
use crate::models::Book;
use crate::verify::verify_toc;

mod create;
use crate::create::{patch_epub, scan_spine_for_headings};

mod generate_book;
mod models;


pub fn check_epub(path:&str, book_id:&str)->Result<(),Box<dyn std::error::Error>>{
    match verify_epub(path) {
     Ok(index)=>{
        println!("no patch needed");
        generate_book_instance(path, 4)?;
    },
     Err(_)=>{
        generate_toc(path, book_id)?;
        let index=verify_epub(path)?;
        generate_book_instance(path, 4)?;
        println!("book patched");
     }
    }


    Ok(())
}

fn generate_toc(path:&str, book_id:&str)->Result<(),Box<dyn std::error::Error>>{
    println!("toch generation");
    let mut epub=EpubDoc::new(path)?;
    let toc=scan_spine_for_headings(&mut epub);
    println!("found tc: {:?}",toc);
    patch_epub( path.to_owned(),toc,book_id)?;

    Ok(())

}

fn verify_epub(path:&str)->Result<u32,Box< dyn std::error::Error>>{
    let mut epub=EpubDoc::new(path)?;
    verify_toc(&mut epub)
}
use std::fs::File;
fn generate_book_instance(path:&str, init:u32)->Result<(),Box< dyn std::error::Error>> {
    let mut epub=EpubDoc::new(path)?;
    let max=epub.get_num_chapters() as u32;
    println!("init: {}", init);
    let mut book=Book{
        path:path.to_owned(),
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

    serde_json::to_writer_pretty(File::create("book.json")?, &book)?;
    Ok(())
}