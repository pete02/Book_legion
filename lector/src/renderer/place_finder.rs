#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleChar {
    ch: char,
    html_offset: usize,
}

pub fn find_plain_text_start(html: &str, needle: &str) -> Option<usize> {

    let mut visible = Vec::<(char, usize)>::new();

    let mut in_tag = false;

    for (i, ch) in html.char_indices() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                visible.push((ch, i));
            }
            _ => {}
        }
    }

    let flat: String = visible.iter().map(|(c, _)| *c).collect();
        println!("flat: {}",flat);
    let match_pos = flat.find(needle)?;
    println!("match: {}",match_pos);
    let html_offset = visible[match_pos].1;

    Some(find_preceding_tag(html, html_offset))
}

fn find_preceding_tag(html: &str, mut pos: usize) -> usize {
    let bytes = html.as_bytes();

    while pos > 0 {
        if bytes[pos] == b'<' {
            return pos;
        }
        pos -= 1;
    }

    0
}