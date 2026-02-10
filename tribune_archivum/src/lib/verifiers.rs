use std::{fs::File, path::Path};

use zip::ZipArchive;


pub fn verify_integrity(path: &Path)-> Result<bool,Box<dyn std::error::Error>>{
    let file=File::open(path)?;
    let archive=ZipArchive::new(file)?;

    let names=archive.file_names().collect::<Vec<&str>>();
    
    if !names.contains(&"toc.ncx"){
        return Ok(true)
    }



    Ok(false)
}