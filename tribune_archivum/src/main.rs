mod tests;
pub mod analysis;

use anyhow::Result;
use epub::doc::EpubDoc;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::ZipArchive;

/// A minimal “dumb ToC” generator: reads spine and generates NCX.
pub fn generate_dumb_toc(chapters: Vec<String>, output_path: &Path) -> Result<()> {

    // 5. Generate minimal NCX
    let mut out = File::create(output_path)?;
    write!(
        out,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
<navMap>
"#
    )?;

    for (i, href) in chapters.iter().enumerate() {
        write!(
            out,
            r#"<navPoint id="nav{0}" playOrder="{0}">
  <navLabel><text>Chapter {0}</text></navLabel>
  <content src="{1}"/>
</navPoint>
"#,
            i + 1,
            href
        )?;
    }

    write!(out, "</navMap>\n</ncx>")?;
    Ok(())
}


fn main(){
    let epub_path = Path::new("generate_toc.ncx");

    let mut chapters=Vec::new();
    for i in 1..403{
        chapters.push(format!("OEBPS/Text/Chapter {}",i));
    }

    for i in 1..7{
        chapters.push(format!("OEBPS/Text/Side Story {}",i));
    }
    let _=generate_dumb_toc(chapters, epub_path);
}

