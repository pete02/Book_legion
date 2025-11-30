use serde::Deserialize;
use serde_json;
use std::fs;
use std::error::Error;
use reqwest::blocking::get;

use crate::epub_creator::*;
use crate::parser;

// --- JSON Config Structs ---
#[derive(Debug, Deserialize, Clone)]
pub struct SiteConfig {
    parent_url: String,
    chapter: String,
    title: String,
    book: String,
    list: List,
    limiter: String,
    remover: Option<Vec<String>>
}
#[derive(Debug, Deserialize, Clone)]
pub struct List{
    pub wrapper: String,
    pub selector: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub royal_road: SiteConfig,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file_content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&file_content)?;
    
    Ok(config)
}

pub fn scrape_and_build_epub(config: &SiteConfig, mut epub:MyEpub  ,url:&str, limiter_value: &str) -> Result<(), Box<dyn Error>> {    
    let html = get(url)?.text()?;
    let tbody_html = parser::fetch_element_by_selector(&html,&config.book )?;
    let links=parser::extract_links(
        &tbody_html,
        &config.list,
        Some((&config.limiter,limiter_value))
    )?;

    println!("Found {} chapters", links.len());

    for (i, link) in links.iter().enumerate() {
        let full_url = format!("{}{}", config.parent_url, link);
        let chapter_html = get(&full_url)?.text()?;
        
        let mut content_html = parser::fetch_element_by_selector(&chapter_html, &config.chapter)?;
        
        if let Some(text)=&config.remover{
            content_html=parser::strip_top_level_tags(&content_html, text)?;
            println!("cont:{}",content_html);
        }

        let title = parser::extract_text(&chapter_html, &config.title).unwrap_or_else(||format!("Chapter {}",i));
        let ch=Chapter{num:i+1, title: title, html:content_html};

        epub.add_chapter(&ch)?;
    }
    epub.generate()?;

    println!("EPUB generated successfully");

    Ok(())
}
