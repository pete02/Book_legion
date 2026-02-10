use std::fs::File;
use std::io::{Write, Cursor};
use std::path::Path;
use zip::write::{FileOptions, SimpleFileOptions};

pub struct TestEpub {
    title: String,
    chapters: Vec<String>,
    toc: Option<Vec<TocItem>>,
    remove_files: Vec<String>, // for simulating missing files
}

#[derive(Clone)]
pub struct TocItem {
    pub href: String,
    pub play_order: Option<String>,
}

impl TocItem {
    pub fn chapter(href: &str) -> Self {
        Self { href: href.to_string(), play_order: None }
    }

    pub fn chapter_with_playorder(href: &str, play_order: &str) -> Self {
        Self { href: href.to_string(), play_order: Some(play_order.to_string()) }
    }
}

impl TestEpub {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            chapters: vec![],
            toc: None,
            remove_files: vec![],
        }
    }

    pub fn chapters(mut self, files: Vec<&str>) -> Self {
        self.chapters = files.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn toc(mut self, toc_items: Vec<TocItem>) -> Self {
        self.toc = Some(toc_items);
        self
    }

    pub fn no_toc(mut self) -> Self {
        self.toc = None;
        self
    }

    pub fn remove_file(mut self, file: &str) -> Self {
        self.remove_files.push(file.to_string());
        self
    }

    pub fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);

        
        let options= SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);        

        // write chapters
        for chapter in &self.chapters {
            if self.remove_files.contains(chapter) {
                continue; // simulate missing file
            }
            zip.start_file(chapter, options)?;
            zip.write_all(b"<html><body><p>Test chapter</p></body></html>")?;
        }

        // write toc.ncx if any
        if let Some(toc_items) = &self.toc {
            zip.start_file("toc.ncx", options)?;
            let mut ncx = String::from(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <head></head>
  <docTitle><text>"#,
            );
            ncx.push_str(&self.title);
            ncx.push_str("</text></docTitle><navMap>");
            let mut play_order_counter = 1;
            for item in toc_items {
                let play_order = item.play_order.clone().unwrap_or(play_order_counter.to_string());
                ncx.push_str(&format!(
                    r#"<navPoint id="np{}" playOrder="{}">
        <navLabel><text>{}</text></navLabel>
        <content src="{}"/>
    </navPoint>"#,
                    play_order_counter, play_order, item.href, item.href
                ));
                play_order_counter += 1;
            }
            ncx.push_str("</navMap></ncx>");
            zip.write_all(ncx.as_bytes())?;
        }

        zip.finish()?;
        Ok(())
    }
}