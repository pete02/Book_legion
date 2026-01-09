
use epub::doc::{EpubDoc};
use std::{io::BufReader};
use std::fs::File;

use crate::verify::extract_heading;

#[derive(Debug)]
pub struct TocEntry {
    pub title: String,
    pub file: String,
    pub anchor: Option<String>
}

pub fn scan_spine_for_headings(epub: &mut EpubDoc<BufReader<File>>) -> Vec<TocEntry> {
    let spine = epub.spine.clone();
    let mut out = Vec::new();

    for (i, idref) in spine.clone().iter().enumerate() {
        epub.set_current_chapter(i);
        println!("idref: {:?}", idref);

        let text = match epub.get_current_str() {
            Some((txt, _)) => txt,
            None => continue,
        };

        if let Some(title) = extract_heading(&text) {
            println!("title: {:?}", title);
            if title.len() > 0{
                out.push(TocEntry {title, file: "OEBPS/".to_owned()+&idref.clone().idref, anchor:None});
            }
        }
    }

    out
}


pub fn patch_epub(path:String, input:Vec<TocEntry>, book_id: &str)->Result<(),Box<dyn std::error::Error>>{
    println!("inpout: {:?}",input);
    let ncx=make_ncx(&input, book_id);
    replace_file_in_epub(&path, "toc.ncx", &ncx)?;

    Ok(())
}

fn xml_escape(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            _ => c.to_string(),
        })
        .collect::<String>()
}



fn make_ncx(toc: &[TocEntry], book_id: &str) -> String {
    let mut nav_points = String::new();

    for (i, entry) in toc.iter().enumerate() {
        let src = match &entry.anchor {
            Some(a) => format!("{}#{}", entry.file, a),
            None => entry.file.clone(),
        };

        nav_points.push_str(&format!(
r#"<navPoint id="navPoint-{i}" playOrder="{i}">
    <navLabel>
        <text>{}</text>
    </navLabel>
    <content src="{}"/>
</navPoint>
"#,
            xml_escape(&entry.title),
            xml_escape(&src)
        ));
    }

    format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{book_id}"/>
  </head>
  <docTitle><text>{book_id}</text></docTitle>
  <navMap>
    {nav_points}
  </navMap>
</ncx>"#)
}




use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;
use std::io::{Read, Write,Cursor};



pub fn replace_file_in_epub(
    path: &str,
    file_to_replace: &str,
    new_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut src = File::open(path)?;
    let mut zip = ZipArchive::new(&mut src)?;

    let mut out = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(&mut out);

    let mut replaced = false;
    
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_string();

        let options = FileOptions::<()>::default()
            .compression_method(file.compression())
            .unix_permissions(file.unix_mode().unwrap_or(0o644));

        writer.start_file(name.clone(), options)?;

        if name == file_to_replace {
            writer.write_all(new_content.as_bytes())?;
            replaced = true;
        } else {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            writer.write_all(&contents)?;
        }
    }
        println!("hjere");

    // If the file did not exist, create it
    if !replaced {
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        writer.start_file(file_to_replace, options)?;
        writer.write_all(new_content.as_bytes())?;
    }

    writer.finish()?;

    std::fs::write(path, out.into_inner())?;
    Ok(())
}