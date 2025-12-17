
use epub::doc::EpubDoc;
use std::{io::BufReader};
use std::fs::File;


pub fn verify_toc(epub: &mut EpubDoc<BufReader<File>>)->Result<u32,Box<dyn std::error::Error>>{
    let toc=epub.toc.clone();
    let spine=epub.spine.clone();
    if toc.len() ==0 {return Err("no toc found".into());}
    let mut init=0;
    for ch in toc{
        let f=ch.content.to_str();
        match f {
            Some(file)=>{
                if let Some((index, _item)) = spine.iter().enumerate().find(|(_, item)| item.idref == clean_path(file) ){
                    epub.set_current_chapter(index);
                    if !check_title(epub, &ch.label) {
                        return Err("Toc not Aligned".into());
                    }else{
                        init=index as u32;
                    }
                }
            },
            None=>{
                return Err("could not extract content str".into())
            }
        }; 
    }
    Ok(init)

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

use scraper::{Html, Selector, ElementRef};
pub fn extract_heading(xhtml: &str) -> Option<String> {
    let doc = Html::parse_document(xhtml);

    let sel = Selector::parse("[class]").unwrap();

    for el in doc.select(&sel) {
        let tag = el.value().name();
        if tag == "body" || tag == "html" || tag == "head" {
            continue;
        }

        let text = extract_clean_text(el);

        if text.is_empty() { continue; }

        let lw = text.to_lowercase();

        if (lw.contains("chapter")
            || lw.starts_with("prologue"))
            && text.split_whitespace().count() <= 12
        {
            return Some(text);
        }
    }

    None
}

fn extract_clean_text(el: ElementRef) -> String {
    el.text()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}