
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

use crate::renderer::{self, find_page_boundary::{split_html_at, split_html_at_end}};



pub fn find_first_sentence_boundary(text: &str, limit: usize, end: usize) -> Option<usize> {
    let new_limit=walk_prev_tag(text, limit);

    if new_limit==0{
        return Some(0);
    }

    let split_text= split_html_at(text, new_limit);
    console("start return");
    if let Some(mut start)=return_start(split_text, end-split_text.len()) {
        if let Some((tagstart,len))=next_closing_tag(&text[start..end]) {
            if tagstart==0{
                start +=len
            } 
        }
        if start==end{
            return None;
        }else{
            return Some(start)
        }
    }

    None
}

fn return_start(split_text:&str, start: usize)->Option<usize>{
    let tag_end = first_opening_tag(split_text);
    let punct_end = first_sentence_terminator(split_text);
    console(&format!("split_text: {}", split_text));
    console(&format!("start: {}", start));
    match (tag_end,punct_end) {
        (Some(t), Some(p)) => {
            let max= t.min(p);
            console(&format!("split: {}",&split_text[..max]));
            console(&format!("split at: {}",max));
            return Some(start+max)
        }
        (Some(t), None) => {
            console(&format!(" tag found: {}",&split_text[..t]));
            console(&format!(" tag found at: {}",start+t));
            return Some(start+t)
        },
        (None, Some(p)) => {
            console(&format!(" pucnt found: {}",&split_text[..p]));
            console(&format!(" pucnt found at: {}",start+p));
            return Some(start+p)
        },
        (None, None) => {
            console("none found");
            return None
        },
    }
}

pub fn walk_prev_tag(text: &str, mut limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }

    if let Some((mut start, mut new_limit)) =
        prev_opening_tag(split_html_at(text, limit))
    {
        renderer::console(&format!(
            "prev start: {}, new lim: {}, given lim: {}",
            start, new_limit, limit
        ));

        while start == 0 {
            if limit <= new_limit {
                break;
            }

            limit = limit - new_limit;

            renderer::console(&format!("moved limit back: {}", limit));

            match prev_opening_tag(split_html_at(text, limit)) {
                None => break,
                Some((s, l)) => {
                    start = s;
                    new_limit = l;
                }
            }
        }
    }

    limit
}



fn prev_opening_tag(text: &str) -> Option<(usize, usize)> {
    let re = Regex::new(r"<[^/][^>]*>").unwrap();

    re.find(text).map(|m| (m.start(), m.end() - m.start()))
}

fn first_opening_tag(text: &str) -> Option<usize> {
    let re = Regex::new(r"<[^/\s>][^>]*>").unwrap();
    re.find(text).map(|m| m.start())
}

fn first_sentence_terminator(text: &str) -> Option<usize> {
    let re = Regex::new(r#"[.!?][!?.)}\]'"»』\s]*"#).unwrap();
    re.find_iter(text)
        .filter(|m| {
            let before = text[..m.start()].chars().next_back();
            let after = text[m.end()..].chars().next();
            
            // Skip decimal points: digit.digit
            if matches!((before, after), (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit()) {
                return false;
            }
            
            // Skip if this is an abbreviation period
            if m.as_str().starts_with('.') && is_abbreviation_period(text, m.start()) {
                return false;
            }
            
            true
        })
        .next()
        .map(|m| m.end())
}


/// Checks if a period at the given position is part of an abbreviation
/// Returns true if the period should NOT be treated as a sentence boundary
fn is_abbreviation_period(text: &str, period_pos: usize) -> bool {
    // Get text before and after the period
    let before = &text[..period_pos];

    // Check if there's text before the period (abbreviation must have something before it)
    if before.is_empty() {
        return false;
    }
    
    // Get the word before the period (may include dots for multi-part abbreviations like "Ph.D.")
    let before_word = before
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    
    // Check if the word before the period matches an abbreviation
    let before_lower = before_word.to_lowercase();
    if ABBREVIATIONS.contains(&before_lower.as_str()) {
        return true;
    }
    
    // Check for multi-part abbreviations like "Ph.D." or "U.S.A."
    // Look for patterns like "X.Y." where X and Y are abbreviations
    if before_word.contains('.') {
        let parts: Vec<&str> = before_word.split('.').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            // Check if the last part before this period is an abbreviation
            let last_part = parts[parts.len() - 1].to_lowercase();
            if ABBREVIATIONS.contains(&last_part.as_str()) {
                return true;
            }
        }
    }
    
    // Check for "et al." pattern
    if before_word.contains("et al") {
        return true;
    }
    
    false
}


pub fn find_last_sentence_boundary(text: &str, limit: usize, start:usize) -> Option<usize> {
    let new_limit=limit.max(walk_next_tag(text, walk_closing_punctuation(text, limit)));
    let split_text= if text.len()!=limit{
        split_html_at_end(text, new_limit-start)
    }else {
        text
    };

    if let Some(end)=return_end(split_text, start) {
        console(&format!("got end at: {}", end));
        let walked=walk_next_tag(text, walk_closing_punctuation(text, end));
        return Some(walked.max(end));
    }
    None
    
}

fn return_end(split_text:&str, start: usize)->Option<usize>{
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

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
fn console(text: &str){
    #[cfg(target_arch = "wasm32")]
    console::log_1(&JsValue::from_str(text));
    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", text);
}
