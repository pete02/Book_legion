

use regex::Regex;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", 
    "link", "meta", "param", "source", "track", "wbr"
];

pub fn decode_html(html: &str) -> String {
    let mut text=html_escape::decode_html_entities(html).to_string().trim().to_string();
    text=strip_whitespace_between_tags(&text);
    text=strip_html_comments(&text);

    text
}


pub fn strip_html_comments(text: &str) -> String {
    static COMMENT_RE: std::sync::LazyLock<Regex> = 
        std::sync::LazyLock::new(|| Regex::new(r"<!--.*?-->").unwrap());
    COMMENT_RE.replace_all(text, "").to_string()
}

use std::sync::LazyLock;
pub fn strip_whitespace_between_tags(text: &str) -> String {
    static WS_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)(</\w+>)\s+(<\w)").unwrap()
    });
    WS_RE.replace_all(text, "$1$2").to_string()
}


pub fn heal_html(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut tag_stack: Vec<String> = Vec::new();
    let mut chars = html.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if ch != '<' {
            output.push(ch);
            continue;
        }

        if matches!(chars.peek(), Some((_, '/'))) {
            chars.next(); // consume '/'
            handle_closing_tag(&mut chars, &mut output, &mut tag_stack);
        } else {
            handle_opening_tag(&mut chars, &mut output, &mut tag_stack, html, i);
        }
    }

    close_unclosed_tags(&mut output, tag_stack);
    output
}

pub fn handle_closing_tag(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    output: &mut String,
    tag_stack: &mut Vec<String>,
) {
    let tag_name = read_tag_name(chars);
    skip_to_tag_end(chars);

    let matches_top = tag_stack.last().map_or(false, |top| {
        top.eq_ignore_ascii_case(&tag_name)
    });

    if matches_top {
        tag_stack.pop();
        output.push_str("</");
        output.push_str(&tag_name);
        output.push('>');
    }

}

pub fn handle_opening_tag(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    output: &mut String,
    tag_stack: &mut Vec<String>,
    source: &str,
    tag_start: usize,
) {
    let tag_name = read_tag_name(chars);
    let is_self_closing = consume_if_self_closing(chars);
    skip_to_tag_end(chars);

    // raw_end is the position of '>' in source — peek at chars after skip
    // to find where we actually landed
    let raw_end = chars
        .peek()
        .map(|(i, _)| i.saturating_sub(1))
        .unwrap_or(source.len().saturating_sub(1));

    if !is_void_element(&tag_name) && !is_self_closing {
        tag_stack.push(tag_name.to_lowercase());
    }

    emit_open_tag(output, is_self_closing, source, tag_start, raw_end);
}

pub fn emit_open_tag(
    output: &mut String,
    is_self_closing: bool,
    source: &str,
    raw_start: usize,
    raw_end: usize,
) {
    if is_self_closing {
        // Self-closing: reconstruct without the slash since we normalise to
        // paired tags, but preserve attributes from the source slice.
        // source[raw_start..=raw_end] looks like `<foo attr="x"/>`
        // Emit as `<foo attr="x">` (drop the slash before `>`).
        let inner = source[raw_start..=raw_end]
            .trim_end_matches('>')
            .trim_end_matches('/')
            .trim_end();
        output.push_str(inner);
        output.push('>');
    } else {
        // Void or normal: emit the raw tag as-is — attributes included.
        output.push_str(&source[raw_start..=raw_end]);
    }
}


pub fn close_unclosed_tags(output: &mut String, tag_stack: Vec<String>) {
    for tag in tag_stack.into_iter().rev() {
        output.push_str("</");
        output.push_str(&tag);
        output.push('>');
    }
}



pub fn read_tag_name(chars: &mut std::iter::Peekable<std::str::CharIndices>) -> String {
    let mut name = String::new();
    while matches!(chars.peek(), Some((_, c)) if !c.is_whitespace() && *c != '>' && *c != '/') {
        name.push(chars.next().unwrap().1);
    }
    name
}

/// Advance the iterator until `>` has been consumed, returning its byte index.
fn skip_to_tag_end(chars: &mut std::iter::Peekable<std::str::CharIndices>) -> usize {
    let mut last = 0;
    while let Some((i, ch)) = chars.peek() {
        if *ch == '>' {
            last = *i;
            chars.next();
            break;
        }
        last = *i;
        chars.next();
    }
    last
}

pub fn consume_if_self_closing(chars: &mut std::iter::Peekable<std::str::CharIndices>) -> bool {
    if matches!(chars.peek(), Some((_, '/'))) {
        chars.next();
        true
    } else {
        false
    }
}

pub fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_lowercase().as_str())
}



