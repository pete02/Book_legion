use epub::doc::EpubDoc;
use std::io::BufReader;
use std::fs::File;



pub fn get_start_index(epub: &mut EpubDoc<BufReader<File>>)->Result<usize,Box<dyn std::error::Error>>{

    let toc: Vec<epub::doc::NavPoint>=epub.toc.clone();
    let spine=epub.spine.clone();
    let (file,first)=determine_first_chapter(&toc)?;
    
    if let Some((index, _item)) = spine.iter().enumerate().find(|(_, item)| item.idref == file ) {
        epub.set_current_chapter(index);
        
        if check_first(epub,&first.label){
            Ok(index)
        }else{
            println!("Sourced the wrong index");
            Err("Sourced the wrong index".into())
        }
        
    }else{
        println!("could not extract index");
        Err("Could not extract index".into())
    }


}


fn check_first(epub:&mut EpubDoc<BufReader<File>>,title:&str)->bool{
    let  heading=match epub.get_current_str() {
        Some(text)=>extract_heading(&text.0),
        None=>None
    };

    if heading.is_none(){return false;};

    let txt=heading.unwrap();
    let strips=title.split_whitespace().collect::<Vec<_>>();
    let a = strips.iter()
        .filter(|v| {
            let cleaned: String = v.chars()
                .filter(|c| c.is_alphanumeric()) 
                .collect();
            !cleaned.is_empty() || v.chars().all(|c| c.is_ascii_digit())
        })
        .collect::<Vec<_>>();

    return a.iter().all(|f| txt.contains(*f));
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
            || lw.starts_with("prologue")
            || lw.starts_with("epilogue"))
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

fn is_non_chapter(label: &str) -> bool {
    let l = label.to_lowercase();

    const IGNORE: [&str; 8] = [
        "cover",
        "title page",
        "title",
        "author",
        "note",
        "copyright",
        "dedication",
        "acknowledg",
    ];

    IGNORE.iter().any(|k| l.contains(k))
}

fn clean_path(content: &str) -> String {
    let file = content.rsplit('/').next().unwrap_or(content);
    file.split(|c| c == '#' || c == '?').next().unwrap().to_string()
}


use epub::doc::NavPoint;
fn determine_first_chapter(navpoints: &[NavPoint]) -> Result<(String,&NavPoint), &'static str> {
    // Normalize + filter
    let candidates: Vec<(String, &NavPoint)> = navpoints.iter()
        .filter(|np| !is_non_chapter(&np.label))
        .map(|np| (clean_path(&np.content.to_str().unwrap()), np))
        .collect();

    if candidates.len() <= 2 {
        return Err("TOC too small or unreliable; cannot determine first chapter");
    }
    Ok(candidates[0].clone())
}