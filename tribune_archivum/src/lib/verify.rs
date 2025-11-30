
use epub::doc::EpubDoc;
use std::{io::BufReader};
use std::fs::File;


pub fn verify_toc(epub: &mut EpubDoc<BufReader<File>>)->Result<(),Box<dyn std::error::Error>>{
    let toc=epub.toc.clone();
    let spine=epub.spine.clone();
    for ch in toc{
        let f=ch.content.to_str();
        match f {
            Some(file)=>{
                if let Some((index, _item)) = spine.iter().enumerate().find(|(_, item)| item.idref == clean_path(file) ){
                    epub.set_current_chapter(index);
                    if !check_title(epub, &ch.label) {
                        return Err("Toc not Aligned".into());
                    }
                }
            },
            None=>{
                return Err("could not extract content str".into())
            }
        }; 
    }
    Ok(())

}

fn clean_path(content: &str) -> String {
    let file = content.rsplit('/').next().unwrap_or(content);
    file.split(|c| c == '#' || c == '?').next().unwrap().to_string()
}

fn check_title(epub:&mut EpubDoc<BufReader<File>>,title:&str)->bool{
    let  heading=match epub.get_current_str() {
        Some(text)=>extract_heading(&text.0),
        None=>None
    };

    if heading.is_none(){return false}

    let txt=heading.unwrap().to_lowercase();
    
    let strips=title.split_whitespace().collect::<Vec<_>>();
    let a = strips.iter()
        .filter(|v| {
            let cleaned: String = v.chars()
                .filter(|c| c.is_alphanumeric()) 
                .collect();
            !cleaned.is_empty() || v.chars().all(|c| c.is_ascii_digit())
        })
        .collect::<Vec<_>>();
    return a.iter().all(|f| txt.contains(&(f.to_lowercase())));
}

use scraper::{Html, Selector};
pub fn extract_heading(xhtml: &str) -> Option<String> {
    let doc = Html::parse_document(xhtml);

    let selectors = [
        Selector::parse("div[class]").unwrap(),
        Selector::parse("h1[class]").unwrap(),
        Selector::parse("h2[class]").unwrap(),
        Selector::parse("h3[class]").unwrap(),
    ];

    for sel in selectors.iter() {
        for el in doc.select(sel) {
            if let Some(class_attr) = el.value().attr("class") {
                let lower = class_attr.to_lowercase();
                if lower.contains("chapter")
                    || lower.contains("heading")
                    || lower.contains("title")
                    || lower.contains("sect")
                {
                    return Some(el.inner_html());
                }
            }
        }
    }

    None
}
