
use epub::doc::EpubDoc;
use std::io::{BufReader, Write};
use std::fs::File;
use regex::Regex;



pub fn get_clean_chapter(epub: &mut EpubDoc<BufReader<File>>) ->  Result<String, Box<dyn std::error::Error>> {
    let chapter: String = get_chapter(epub)?;
    Ok(strip_epub_boilerplate(&chapter))
}

fn get_chapter(epub: &mut EpubDoc<BufReader<File>>) -> Result<String, String> {
    if let Some((chapter_text, _)) = epub.get_current_str() {
        Ok(chapter_text)
    } else {
        Err("No chapter found".to_string())
    }
}

fn strip_epub_boilerplate(xhtml: &str) -> String {
    let re = Regex::new(r"(?s)^<\?xml[^>]*>\s*<html[^>]*>.*?<body[^>]*>\s*|\s*</body>\s*</html>\s*$").unwrap();
    re.replace_all(xhtml, "").to_string()
}

pub fn clean_html(html:&str)->Result<String, Box<dyn std::error::Error>>{
    let mut text = html.replace("</div>", ". </div>");

    let re = [
        (r"(?s)<\?xml[^>]*\?>", ""),   // Remove XML header
        (r"(?s)<head.*?>.*?</head>", ""), // Remove <head> content
        (r"</p>|</div>|<br\s*/?>", " "),  // Replace block tags with spaces
        (r"<[^>]+>", ""),                // Strip remaining tags
    ];

    for (pattern, replacement) in &re {
        let regex = Regex::new(pattern)?;
        text = regex.replace_all(&text, *replacement).to_string();
    }
    
    Ok(normalize_whitespace(&text))
}

fn normalize_whitespace(text: &str) -> String {
    let re_spaces = Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(&text.replace('\n', " "), " ").trim().to_string()
}

pub fn extract_cover(book:&str, jpg:&str)->Result<(),Box<dyn std::error::Error>>{
    let files=extract_files(book, vec!["cover.jpg", "cover.jpeg"])?;
    if files.len() == 1{
        let mut file=File::create(jpg)?;
        if let Some(vec)=files.values().next(){
            file.write_all(vec)?
        }
    }
    Ok(())   
}
use zip::ZipArchive;
use std::collections::HashMap;
use std::io::Read;

fn extract_files(path: &str, file_types:Vec<&str>)->Result<HashMap<String,Vec<u8>>,Box<dyn std::error::Error>>{
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