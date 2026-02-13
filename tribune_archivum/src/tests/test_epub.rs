use std::fs::File;
use std::io::{Write, Cursor};
use std::path::Path;
use zip::write::{FileOptions, SimpleFileOptions};

pub struct TestEpub {
    title: String,
    chapters: Vec<String>,
    toc: Option<Vec<TocItem>>,
    spine: Vec<String>,
    remove_files: Vec<String>, // for simulating missing files
    create_container: bool
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
    pub fn new(title: &str, create:bool) -> Self {
        Self {
            title: title.to_string(),
            chapters: vec![],
            toc: None,
            spine: vec![],
            remove_files: vec![],
            create_container: create
        }
    }

    pub fn chapters(mut self, files: Vec<&str>) -> Self {
        self.chapters = files.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn spine(mut self, refs: Vec<&str>)->Self{
        self.spine=refs.iter().map(|f|f.to_owned().to_owned()).collect();
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

        let container=r#"
            <?xml version="1.0"?>
        <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
        <rootfiles>
            <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
        </rootfiles>
        </container>
        "#;

        if self.create_container{
            zip.start_file("META-INF/container.xml", options)?;
            zip.write_all(container.as_bytes())?;
        }

        let mut opf = String::new();
        opf.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
        <package version="2.0" xmlns="http://www.idpf.org/2007/opf">
        <metadata>
            <title>{}</title>
        </metadata>
        <manifest>
        "#,
            self.title
        ));

        for chapter in &self.chapters {
            opf.push_str(&format!(
                r#"    <item id="{0}" href="{0}" media-type="application/xhtml+xml"/>"#,
                chapter
            ));
            opf.push('\n');
        }

        opf.push_str("  </manifest>\n  <spine toc=\"toc.ncx\">\n");

        for itemref in &self.spine {
            opf.push_str(&format!("    <itemref idref=\"{}\"/>\n", itemref));
        }

        opf.push_str("  </spine>\n</package>");
        zip.start_file("content.opf", options)?;
        zip.write_all(opf.as_bytes())?;



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