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


pub fn find_last_sentence_boundary(text: &str, limit: usize) -> Option<usize> {
    let limit = normalize_limit(text, limit);
    
    for i in (0..limit).rev() {
        if is_valid_cut(text, i) {
            let resolved_pos = resolve_boundary(text, i);
            return Some(resolved_pos);
        }
    }

    None
}

// Returns None, if the text does not contain a sentence boundary within the specified limit from the end.
pub fn find_first_sentence_boundary(text: &str, limit_from_end: usize) -> Option<usize> {
    let limit = normalize_limit(text, text.len()-limit_from_end.min(text.len()));
    if limit==0{
        return Some(0)
    }
    for i in limit..text.len()-1 {
        if is_valid_cut(text, i) {
            let resolved_pos = resolve_boundary(text, i);
            return Some(resolved_pos);
        }
    }

    None
}


fn normalize_limit(text: &str, mut limit: usize) -> usize {
    let len = text.len();
    limit = limit.min(len);

    // If inside HTML tag, move backward until safe
    while is_inside_html_tag_boundary(text, limit) && limit > 0 {
        limit -= 1;
    }

    limit
}

pub fn is_valid_cut(text: &str, pos: usize) -> bool {
    is_between_html_blocks(text, pos) || is_valid_boundary(text, pos)
}

pub fn is_valid_boundary(text: &str, pos: usize) -> bool {
    let terminators=['.', '!', '?'];

    text.chars().nth(pos).is_some_and(|f|
        terminators.contains(&f) 
        && !is_inside_html(text, pos)
        && !is_abbreviation_boundary(text, pos)
        && !is_ellipsis(text, pos)
        && !is_number_boundary(text, pos)
    )
}


pub fn resolve_boundary(text: &str, mut pos: usize) -> usize {
    let len = text.len();
    if pos >= len {
        return len;
    }

    // ONLY advance if we are NOT at a structural HTML boundary
    if !is_between_html_blocks(text, pos) {
        pos += text[pos..].chars().next().unwrap().len_utf8();
    }

    pos = collapse_punctuations(text, pos);
    pos = collapse_closers(text, pos);
    pos = collapse_whitespace(text, pos);
    pos = collapse_html_closers(text, pos);

    pos
}


fn collapse_punctuations(text: &str, mut pos: usize) -> usize {
    loop {
        let next = text[pos..].chars().next();
        match next {
            Some('!' | '?' | '.') => {
                pos += next.unwrap().len_utf8();
            }
            _ => break,
        }
    }
    pos
}

fn collapse_closers(text: &str, mut pos: usize) -> usize {
    loop {
        let next = text[pos..].chars().next();
        match next {
            Some('"' | '\'' | ')' | ']' | '»' | '』') => {
                pos += next.unwrap().len_utf8();
            }
            _ => break,
        }
    }
    pos
}

fn collapse_whitespace(text: &str, mut pos: usize) -> usize {
    while let Some(c) = text[pos..].chars().next() {
        if c.is_whitespace() {
            pos += c.len_utf8();
        } else {
            break;
        }
    }
    pos
}

fn collapse_html_closers(text: &str, mut pos: usize) -> usize {
    loop {
        if text[pos..].starts_with("</") {
            if let Some(end) = text[pos..].find('>') {
                pos += end + 1;
                continue;
            }
        }
        break;
    }
    pos
}

fn is_number_boundary(text: &str, pos: usize) -> bool {
    is_decimal_separator(text, pos) || is_dotted_numeric_token_boundary(text, pos)
}


fn is_inside_html(text: &str, pos: usize) -> bool {
    let last_open = text[..pos].rfind('<');
    let last_close = text[..pos].rfind('>');

    match (last_open, last_close) {
        (Some(o), Some(c)) => o > c,
        (Some(_), None) => true,
        _ => false,
    }
}

fn is_ellipsis(text: &str, pos: usize) -> bool {
    // Check for ... either starting at pos or with dots before it
    let dots_after = text[pos..].chars().take_while(|&c| c == '.').count();
    let dots_before = text[..pos].chars().rev().take_while(|&c| c == '.').count();
    dots_before + dots_after >= 3
}

fn is_abbreviation_boundary(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return false;
    }
    let before = &text[..pos];
    let after = &text[pos + 1..]; // skip the dot itself

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





fn is_decimal_separator(text: &str, pos: usize) -> bool {
    let before = text[..pos].chars().next_back();
    let after = text[pos + 1..].chars().next();

    matches!(
        (before, after),
        (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit()
    )
}

fn is_dotted_numeric_token_boundary(text: &str, pos: usize) -> bool {
    let before = &text[..pos];

    let prev = before.chars().next_back();

    if !matches!(prev, Some(c) if c.is_ascii_digit()) {
        return false;
    }

    let token_start = find_token_start(before);
    let token = &before[token_start..];

    is_dotted_numeric_token(token)
}

fn find_token_start(before: &str) -> usize {
    before
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0)
}

fn is_dotted_numeric_token(token: &str) -> bool {
    token.contains('.')
        && token.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn is_inside_html_tag_boundary(text: &str, pos: usize) -> bool {
    let before = &text[..pos];

    let last_open = before.rfind('<');
    let last_close = before.rfind('>');

    match (last_open, last_close) {
        (Some(o), Some(c)) => o > c,
        (Some(_), None) => true,
        _ => false,
    }
}

fn is_between_html_blocks(text: &str, pos: usize) -> bool {
    if pos == 0 || pos >= text.len() {
        return false;
    }

    if !text.is_char_boundary(pos) {
        return false;
    }

    let before = &text[..pos];
    let after = &text[pos..];

    before.trim_end().ends_with('>')
        && after.trim_start().starts_with('<')
}