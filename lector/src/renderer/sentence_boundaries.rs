use dioxus::html::{li, u::is};

const ABBREVIATIONS: &[&str] = &[
    "dr", "mr", "mrs", "ms", "prof", "sr", "jr", "vs", "etc", "eg", "ie",
    "st", "ave", "blvd", "rd", "apt", "dept", "inc", "ltd", "co", "corp",
    "vol", "rev", "hon", "col", "gen", "capt", "lt", "gov", "pres", "sen",
    "rep", "us", "uk", "ca", "au", "no", "de", "fr", "it", "es", "pt",
    "am", "pm", "bc", "ad", "cc", "pm", "am", "fig", "tab", "ref", "ch",
    "sec", "art", "par", "vol", "ed", "ed", "rev", "pub", "trans", "comp",
    "et", "al", "cf", "viz", "i.e", "e.g", "a.k.a", "a.m", "p.m",
    "phd", "md", "jr", "sr", "esq", "bachelor", "master", "doctor",
    "st", "dr", "mr", "mrs", "ms", "miss", "rev", "hon", "col", "gen",
    "capt", "lt", "gov", "pres", "sen", "rep", "us", "uk", "ca", "au",
    "no", "de", "fr", "it", "es", "pt", "am", "pm", "bc", "ad", "cc",
    "fig", "tab", "ref", "ch", "sec", "art", "par", "vol", "ed", "pub",
    "trans", "comp", "et", "al", "cf", "viz", "a.k.a", "a.m", "p.m",
];

use regex::Regex;

use crate::renderer::{self, find_page_boundary::{split_html_at, split_html_at_end}, console};



pub fn find_first_sentence_boundary(text: &str, limit_from_end: usize) -> Option<usize> {


    None
}


pub fn find_last_sentence_boundary(text: &str, limit: usize, start:usize) -> Option<usize> {

    renderer::console(&format!("limit calc: {}, start: {}", limit, start));
    let new_limit=limit.max(walk_next_tag(text, walk_closing_punctuation(text, limit)));
    renderer::console(&format!("new limit: {}", new_limit));

    console(&format!("text len: {}",text.len()));
    let split_text= if text.len()!=limit{
        split_html_at_end(text, new_limit-start)
    }else {
        text
    };
    console(&format!("split text len: {}",text.len()));

    if let Some(end)=return_end(split_text, start, limit) {
        console(&format!("got end at: {}", end));
        let walked=walk_next_tag(text, walk_closing_punctuation(text, end));
        return Some(walked.max(end));
    }
    None
    
}

fn return_end(split_text:&str, start: usize, limit: usize)->Option<usize>{
    let tag_end = last_closing_tag(split_text);
    let punct_end = last_sentence_terminator(split_text);
    console(&format!("split_text: {}", split_text));
    match (tag_end,punct_end) {
        (Some(t), Some(p)) => {
            let max= t.max(p);
            console(&format!("split: {}",&split_text[..max]));
            console(&format!("split at: {}",max));
            return Some(start+max)
        }
        (Some(t), None) => {
            console(&format!(" tag found: {}",&split_text[..t]));
            console(&format!(" tag found at: {}",start+t));
            return Some(start + t)
        },
        (None, Some(p)) => {
            console(&format!(" pucnt found: {}",&split_text[..p]));
            console(&format!(" pucnt found at: {}",start+p));
            return Some(start + p)
        },
        (None, None) => {
            console("none found");
            return None
        },
    }
}

pub fn walk_next_tag(text:&str, mut limit: usize)->usize{
    if let Some((mut next,mut new_limit))=next_closing_tag(split_html_at(text, limit)){
        renderer::console(&format!("next start: {}, new lim: {}, given lim: {}",next, new_limit, limit));
        while next==0{
            limit=limit+new_limit;
            renderer::console(&format!("found new limit: {}", limit));
            match next_closing_tag(split_html_at(text, limit)) {
                None=>break,
                Some((n,l))=>{
                    next=n;
                    new_limit=l;
                }
            }
        }
    }
    limit
}

fn next_closing_tag(text: &str) -> Option<(usize, usize)> {
    let re = Regex::new(r"^(\s*</[^>]+>)").unwrap();
    re.find(text).map(|m| (m.start(), m.len()))
}

fn last_closing_tag(text: &str) -> Option<usize> {
    let re = Regex::new(r"</[^>]+>").unwrap();
    re.find_iter(text).last().map(|m| m.end())
}

fn last_sentence_terminator(text: &str) -> Option<usize> {
    let re = Regex::new(r#"[.!?][!?.)}\]'"»』\s]*"#).unwrap();
    re.find_iter(text)
        .filter(|m| {
            let before = text[..m.start()].chars().next_back();
            let after = text[m.end()..].chars().next();
            // skip decimal points: digit.digit
            !matches!((before, after), (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit())
        })
        .last()
        .map(|m| m.end())
}

fn walk_closing_punctuation(text: &str, limit: usize) -> usize {
    let safe_limit = text
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| i <= limit)
        .last()
        .unwrap_or(0);

    let re = Regex::new(r#"^[!?.)}\]'"»』]+"#).unwrap();
    if let Some(m) = re.find(&text[safe_limit..]) {
        safe_limit + m.len()
    } else {
        safe_limit
    }
}