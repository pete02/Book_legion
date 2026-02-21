use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;

#[allow(dead_code)]
pub struct TestEpub {
    title: String,
    chapters: Vec<String>,
    toc: Option<Vec<TocItem>>,
    spine: Vec<String>,
    generate_nav: bool,
    remove_files: Vec<String>, // for simulating missing files
    create_container: bool
}

#[derive(Clone)]
pub struct TocItem {
    pub href: String,
    pub text: String,
    pub play_order: Option<String>,
}

#[allow(dead_code)]
impl TocItem {
    pub fn chapter(href: &str) -> Self {
        Self { href: href.to_string(), text: href.to_string(), play_order: None }
    }

    pub fn chapter_with_playorder(href: &str, text: &str,play_order: &str) -> Self {
        Self { href: href.to_string(), text: text.to_string(), play_order: Some(play_order.to_string()) }
    }
}

#[allow(dead_code)]
impl TestEpub {
    pub fn new(title: &str, create:bool) -> Self {
        Self {
            title: title.to_string(),
            chapters: vec![],
            toc: None,
            generate_nav: false,
            spine: vec![],
            remove_files: vec![],
            create_container: create
        }
    }
    pub fn with_nav(mut self) -> Self {
        self.generate_nav = true;
        self
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

        if self.toc.is_some() && !self.generate_nav{
            opf.push_str(r#"<item href="toc.ncx" id="ncx" media-type="application/x-dtbncx+xml"/>"#);
        }

        for chapter in &self.chapters {
            opf.push_str(&format!(
                r#"    <item id="{0}" href="{0}" media-type="application/xhtml+xml"/>"#,
                chapter
            ));
            opf.push('\n');
        }

        if self.toc.is_some() && self.generate_nav{
            opf.push_str(r#"<item href="nav.xhtml" id="nav.xhtml" media-type="application/x-dtbncx+xml"/>"#);
        }
        

        opf.push_str("  </manifest>\n  <spine toc=\"ncx\">\n");

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


        if self.generate_nav {
            zip.start_file("nav.xhtml", options)?;

            let mut nav = String::new();
            nav.push_str(&format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE html>
        <html xmlns="http://www.w3.org/1999/xhtml" 
            xmlns:epub="http://www.idpf.org/2007/ops">
        <head><title>{}</title></head>
        <body>
            <nav epub:type="toc">
            <h1>Table of Contents</h1>
            <ol>
        "#,
                self.title
            ));

            // Use either TOC items or spine to populate nav
            let items = if let Some(toc_items) = &self.toc {
                toc_items.clone()
            } else {
                // fallback: generate from spine
                self.spine
                    .iter()
                    .map(|s| TocItem::chapter(s))
                    .collect()
            };

            for item in items {
                nav.push_str(&format!(
                    r#"        <li><a href="{}">{}</a></li>"#,
                    item.href, item.text
                ));
                nav.push('\n');
            }

            nav.push_str(
                r#"      </ol>
            </nav>
        </body>
        </html>"#,
            );

            zip.write_all(nav.as_bytes())?;
        }

        // write toc.ncx if any
        if let Some(toc_items) = &self.toc && !self.generate_nav {
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
                    play_order_counter, play_order, item.text, item.href
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