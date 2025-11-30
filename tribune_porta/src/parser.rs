use scraper::{Html, Selector};
use std::{error::Error};
use std::collections::HashSet;

use crate::driver::List;



pub fn fetch_element_by_selector(html: &str, selector_str: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(html);
    let selector=Selector::parse(selector_str).map_err(|_| "this is an error")?;
    if let Some(element) = document.select(&selector).next() {
        Ok(element.html())
    } else {
        Ok("No match found".to_string())
    }
}


pub fn extract_links(
    html_fragment: &str,
    list:&List,
    attr_filter: Option<(&str, &str)>,
) -> Result<Vec<String>, Box<dyn Error>> {

    let row_selector=Selector::parse(&list.selector).map_err(|_| "this was a mistake")?;
    let wrapped_html = format!("<{}>{}</{}>", list.wrapper,html_fragment,list.wrapper);
    let document = Html::parse_fragment(&wrapped_html);
    let link_selector = Selector::parse("a")?;

    let mut links = HashSet::new();

    for element in document.select(&row_selector) {
        if let Some((attr_name, expected_value)) = attr_filter {
            if let Some(actual_value) = element.value().attr(attr_name) {
                if actual_value != expected_value {
                    continue;
                }
            } else {
                continue;
            }
        }

        for a in element.select(&link_selector) {
            if let Some(href) = a.value().attr("href") {
                links.insert(href.to_string());
            }
        }
    }

    let mut unique_links: Vec<_> = links.into_iter().collect();
    unique_links.sort();
    Ok(unique_links)
}

pub fn extract_text(html: &str, selector: &str) -> Option<String> {
    let document = Html::parse_fragment(html);
    let sel = Selector::parse(selector).ok()?;
    document.select(&sel).next().map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
}
#[allow(dead_code)]
pub fn strip_tags(html_fragment: &str, tags: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let mut html = html_fragment.to_string();

    for tag in tags {
        let selector = Selector::parse(tag).map_err(|_|"this was a mistake")?;
        let document = Html::parse_fragment(&html);

        for element in document.select(&selector) {
            let to_remove = element.html();
            let full_match = format!("<{tag}[^>]*?>.*?</{tag}>");
            let re = regex::Regex::new(&full_match).unwrap();

            html = re.replace_all(&html, "").into_owned();
            html = html.replace(&to_remove, "");
        }
    }

    Ok(html)
}

use scraper::ElementRef;
pub fn strip_top_level_tags(
    html_fragment: &str,
    tags: &Vec<String>,
) -> Result<String, Box<dyn Error>> {

    let document = Html::parse_fragment(html_fragment);

    let mut first_div: Option<ElementRef> = None;
    for node in document.tree.root().descendants() {
        if let Some(el) = ElementRef::wrap(node) {
            if el.value().name() == "div" {
                first_div = Some(el);
                break;
            }
        }
    }

    let div = match first_div {
        Some(d) => d,
        None => return Ok(html_fragment.to_string()),
    };

    let mut inner_html = String::new();

    for child in div.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let tag = el.value().name().to_lowercase();
            if tags.contains(&tag) {
                continue;
            }
            inner_html.push_str(&el.html());
        } else if let Some(text) = child.value().as_text() {
            if text.trim().is_empty() {
                continue;
            }
            inner_html.push_str(text);
        }
    }
    let cleaned = format!("<div>{}</div>", inner_html.replace("&nbsp;", " "));
    Ok(cleaned)
}
