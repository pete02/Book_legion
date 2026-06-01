#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchLocation {
    pub html_offset: usize,
}

#[derive(Debug)]
struct VisibleChar {
    ch: char,
    html_offset: usize,
}

pub fn find_plain_text_start(
    html: &str,
    plain_text: &str,
) -> Option<usize> {
    let visible = extract_visible_text(html);

    let visible_text: String = visible.iter().map(|c| c.ch).collect();

    let visible_match_start = visible_text.find(plain_text)?;

    let html_offset = visible[visible_match_start].html_offset;

    Some(include_immediately_preceding_tag(html,html_offset))
}

fn extract_visible_text(html: &str) -> Vec<VisibleChar> {
    let mut result = Vec::new();

    let mut in_tag = false;

    for (offset, ch) in html.char_indices() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                result.push(VisibleChar {
                    ch,
                    html_offset: offset,
                });
            }
            _ => {}
        }
    }

    result
}

fn include_immediately_preceding_tag(
    html: &str,
    text_offset: usize,
) -> usize {
    let prefix = &html[..text_offset];

    let Some(tag_start) = prefix.rfind('<') else {
        return text_offset;
    };

    let Some(tag_end_rel) = prefix[tag_start..].find('>') else {
        return text_offset;
    };

    let tag_end = tag_start + tag_end_rel;

    if tag_end + 1 == text_offset {
        tag_start
    } else {
        text_offset
    }
}