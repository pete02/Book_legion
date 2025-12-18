use zip::ZipArchive;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use epub::doc::EpubDoc;
use crate::models::*;
use regex::Regex;


pub fn get_chapter(status:&BookStatus)->Result<String,String>{
    let path=format!("{}/{}.epub", status.path,status.name);
    let mut book=EpubDoc::new(&path).map_err(|_|"Failed to open EPUB in the path".to_owned())?;
    if status.chapter as usize >book.get_num_chapters(){
        return Err(format!("chapter too large: {} max", book.get_num_chapters()).to_owned());
    }
    book.set_current_chapter(status.chapter as usize);

    if let Some((chapter_text, _)) = book.get_current_str() {
        Ok(chapter_text)
    } else {
        Err("No chapter found".into())
    }
}
fn strip_epub_boilerplate(xhtml: &str) -> String {
    let re = Regex::new(r"(?s)^<\?xml[^>]*>\s*<html[^>]*>.*?<body[^>]*>\s*|\s*</body>\s*</html>\s*$").unwrap();
    re.replace_all(xhtml, "").to_string()
}


pub fn get_chunk(status:&BookStatus)->Result<String,String>{
    let text=get_chapter(status)?;
    let stripped=strip_epub_boilerplate(&text);
    let chunks=stripped.split("\n").collect::<Vec<&str>>();
    if chunks.len() > status.chunk as usize{
        let str=chunks[status.chunk as usize].to_owned();
        Ok(str)
    }else{
        Err("requested chunk too far".into())
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

pub fn extract_cover(path: &str)->Result< Vec<u8>, Box<dyn std::error::Error>>{
    match extract_files(path, vec![".jpg", ".jpeg"]) {
        Ok(files)=>{
            let mut values = files.values();
            if values.len() == 1{
                let Some(v)=values.next()else {return Err("no actual cover".into())};
                Ok(v.to_owned())
            }else{
                Err("Cover not unabiguous".into())
            }
        }
        Err(e)=> Err(format!("could not extract cover: {}", e).into())
    }
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