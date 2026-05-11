use dioxus::html::li;

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


pub fn find_last_sentence_boundary(text: &str, limit: usize) -> Option<usize> {
    let limit = limit.min(text.len());
        
    for (i, c) in text[..limit].char_indices().rev() {
        if c == '.' || c == '!' || c == '?' {
            if is_abbreviation_boundary(text, i) {
                continue;
            }
            if is_number_boundary(text, i) {
                continue;
            }
            
            let mut pos = i + c.len_utf8();
            while pos < limit {
                match text[pos..].chars().next() {
                    Some('!' | '?' | '.') => {
                        pos += text[pos..].chars().next().unwrap().len_utf8();
                    }
                    _ => break,
                }
            }
            
            pos = skip_tralings(text, pos, limit);
            return Some(pos);
        }
    }
    
    None
}

// ... existing code ...

pub fn find_first_sentence_boundary(text: &str, limit_from_end: usize) -> Option<usize> {
    // Convert limit from "from end" to "from start"
    let limit = text.len()-limit_from_end.min(text.len());


    for (i, c) in text[limit..].char_indices() {
        let pos = limit + i;
        
        if c == '.' || c == '!' || c == '?' {
            if is_abbreviation_boundary(text, pos) {
                continue;
            }
            if is_number_boundary(text, pos) {
                continue;
            }
            
            // Found a valid sentence terminator
            // Return the position right after the terminator
            let mut start_pos = pos + c.len_utf8();
            
            // Skip any trailing punctuation (!, ?, .)
            while start_pos < text.len() {
                match text[start_pos..].chars().next() {
                    Some('!' | '?' | '.') => {
                        start_pos += text[start_pos..].chars().next().unwrap().len_utf8();
                    }
                    _ => break,
                }
            }
            
            start_pos = skip_trailing_closers(text, start_pos, text.len());
        
            // Skip trailing HTML tags
            start_pos = skip_trailing_html(text, start_pos, text.len());
            
            
            return Some(start_pos);
        }
    }
    
    None
}


fn is_abbreviation_boundary(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }

    let before = &text[..pos];
    let after = &text[pos + 1..]; // skip the dot itself

    // If the dot is followed by a single capital letter and another dot,
    // we're mid-way through a multi-part abbreviation like Ph.D. or U.S.A.
    {
        let mut after_chars = after.chars();
        let c1 = after_chars.next();
        let c2 = after_chars.next();
        if matches!((c1, c2), (Some(c), Some('.')) if c.is_uppercase()) {
            return true;
        }
    }

    // Scan back to nearest whitespace to get the full word (including internal dots).
    let word_start = before
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    let word = before[word_start..].to_lowercase();
    let normalized = word.replace('.', "");

    ABBREVIATIONS.contains(&normalized.as_str())
}


fn is_number_boundary(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    
    let before = &text[..pos];
    let word_start = before.char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace() || *c == '.' || *c == '!' || *c == '?')
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    
    let word = &before[word_start..];
    word.chars().all(|c| c.is_ascii_digit() || c == '.')
}

fn skip_tralings(text: &str, pos: usize, limit: usize) -> usize {
    let mut new_pos = pos;
    new_pos = skip_trailing_closers(text, new_pos, limit);
    new_pos = skip_trailing_html(text, new_pos, limit);
    new_pos
}

fn skip_trailing_closers(text: &str, pos: usize, limit: usize) -> usize {
    let mut new_pos = pos;
    while new_pos < limit {
        match text[new_pos..].chars().next() {
            Some('"' | '\'' | ')' | ']' | '»' | '』') => {
                new_pos += text[new_pos..].chars().next().unwrap().len_utf8();
            }
            _ => break,
        }
    }
    new_pos
}

fn skip_trailing_html(text: &str, mut pos: usize, limit: usize) -> usize {
    while pos < limit {
        if text[pos..].starts_with("</") {
            // Find the closing >
            if let Some(end) = text[pos..].find('>') {
                pos += end + 1;
                continue;
            }
        }
        break;
    }
    pos
}

