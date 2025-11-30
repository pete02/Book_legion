use epub::doc::EpubDoc;
mod verify;
use crate::verify::verify_toc;

mod create;
use crate::create::{patch_epub, scan_spine_for_headings};

pub fn check_epub(path:&str, book_id:&str)->Result<(),Box<dyn std::error::Error>>{
    match verify_epub(path) {
     Ok(_)=>{},
     Err(_)=>{
        generate_toc(path, book_id)?;
        verify_epub(path)?;
        println!("book patched");
     }
    }


    Ok(())
}

fn generate_toc(path:&str, book_id:&str)->Result<(),Box<dyn std::error::Error>>{
    let mut epub=EpubDoc::new(path)?;
    let toc=scan_spine_for_headings(&mut epub);
    patch_epub( path.to_owned(),toc,book_id)?;

    Ok(())

}

fn verify_epub(path:&str)->Result<(),Box< dyn std::error::Error>>{
    let mut epub=EpubDoc::new(path)?;
    verify_toc(&mut epub)
}