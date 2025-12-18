use epub::doc::EpubDoc;
use std::io::{BufReader};
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