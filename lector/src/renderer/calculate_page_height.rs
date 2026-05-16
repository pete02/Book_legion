
use crate::renderer::sentence_boundaries::*;
pub const PAGE_HEIGHT_FUDGE_FACTOR: f64 = 0.0;

pub fn calculate_page_boundary(
    html: &str,
    start_offset: usize,
    usable_height: f64,
    mut measure: impl FnMut(&str) -> f64,
) -> (usize, usize) {
    let mut accumulated = String::new();
    let mut pos = start_offset;

    while pos < html.len() {
        let token_end = next_token_end(html, pos);
        accumulated.push_str(&html[pos..token_end]);

        if measure(&accumulated) > usable_height {
            // `pos` is the offset immediately before the token that overflowed
            let region = &html[start_offset..pos];
            let snapped = match find_last_sentence_boundary(region, region.len()) {
                Some(rel) => start_offset + rel,
                None => pos, // no sentence boundary found, cut at token edge
            };
            return (start_offset, include_trailing_end_tags(html, snapped));
        }

        pos = token_end;
    }

    (start_offset, html.len())
}

fn next_token_end(html: &str, pos: usize) -> usize {
    if html.as_bytes().get(pos) == Some(&b'<') {
        html[pos..].find('>').map(|i| pos + i + 1).unwrap_or(html.len())
    } else {
        html[pos..].find('<').map(|i| pos + i).unwrap_or(html.len())
    }
}

fn include_trailing_end_tags(html: &str, mut pos: usize) -> usize {
    loop {
        let rest = html[pos..].trim_start();
        let whitespace_len = html[pos..].len() - rest.len();
        if rest.starts_with("</") {
            match rest.find('>') {
                Some(i) => pos += whitespace_len + i + 1,
                None => break,
            }
        } else {
            break;
        }
    }
    pos
}