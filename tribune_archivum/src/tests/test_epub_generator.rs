use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;

#[derive(Clone)]
pub struct TocItem {
    pub src: String,
}

impl TocItem {
    pub fn chapter(file: &str) -> Self {
        Self {
            src: file.to_string(),
        }
    }

    pub fn anchor(file: &str, anchor: &str) -> Self {
        Self {
            src: format!("{}#{}", file, anchor),
        }
    }
}

pub struct TestEpub {
    title: String,
    chapters: Vec<String>,
    toc: Option<Vec<TocItem>>,
}

impl TestEpub {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            chapters: vec![],
            toc: None,
        }
    }

    pub fn chapters(mut self, chapters: Vec<&str>) -> Self {
        self.chapters = chapters.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn toc(mut self, toc: Vec<TocItem>) -> Self {
        self.toc = Some(toc);
        self
    }

    pub fn no_toc(mut self) -> Self {
        self.toc = None;
        self
    }

    pub fn write_to(self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        use zip::CompressionMethod;

        
        let stored =  FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .into();
        

        let deflated = FileOptions::default();

        // REQUIRED: mimetype must be first and uncompressed
        zip.start_file("mimetype", stored)?;
        zip.write_all(b"application/epub+zip")?;

        // container.xml
        zip.start_file("META-INF/container.xml", deflated)?;
        zip.write_all(container_xml().as_bytes())?;

        // content.opf
        zip.start_file("OEBPS/content.opf", deflated)?;
        zip.write_all(content_opf(&self).as_bytes())?;

        // chapters
        for ch in &self.chapters {
            zip.start_file(format!("OEBPS/{}", ch), deflated)?;
            zip.write_all(chapter_xhtml(ch).as_bytes())?;
        }

        // optional toc.ncx
        if let Some(toc) = &self.toc {
            zip.start_file("OEBPS/toc.ncx", deflated)?;
            zip.write_all(ncx(toc).as_bytes())?;
        }

        zip.finish()?;
        Ok(())
    }
}


fn container_xml() -> String {
    r#"<?xml version="1.0"?>
<container version="1.0"
 xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"
      media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#
        .to_string()
}


fn content_opf(epub: &TestEpub) -> String {
    let mut manifest = String::new();
    let mut spine = String::new();

    for (i, ch) in epub.chapters.iter().enumerate() {
        manifest.push_str(&format!(
            r#"<item id="c{}" href="{}" media-type="application/xhtml+xml"/>"#,
            i, ch
        ));

        spine.push_str(&format!(
            r#"<itemref idref="c{}"/>"#,
            i
        ));
    }

    if epub.toc.is_some() {
        manifest.push_str(
            r#"<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#,
        );
    }

    format!(
        r#"<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata>
    <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">{}</dc:title>
  </metadata>
  <manifest>{}</manifest>
  <spine toc="ncx">{}</spine>
</package>"#,
        epub.title,
        manifest,
        spine
    )
}


fn chapter_xhtml(name: &str) -> String {
    let anchor = name.replace(".xhtml", "");

    format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml">
<body>
<h1 id="{0}">{0}</h1>
<p>Content</p>
</body>
</html>"#,
        anchor
    )
}


fn ncx(items: &[TocItem]) -> String {
    let nav_points = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            format!(
                r#"<navPoint id="nav{0}" playOrder="{0}">
<navLabel><text>Entry {0}</text></navLabel>
<content src="{1}"/>
</navPoint>"#,
                i + 1,
                item.src
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
<navMap>
{}
</navMap>
</ncx>"#,
        nav_points
    )
}