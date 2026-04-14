use std::{fs::{self, File}, io::{Read, Write}, path::Path};

use zip::{ZipArchive, ZipWriter};

use crate::lib::{helpers, nav_model::{Html, Nav}, opf_model::Package, toc_model::{Content, DocTitle, Head, NavLabel, NavMap, NavPoint, Ncx}};


pub fn generate_toc(path: &Path)->Result<(), Box<dyn std::error::Error>>{
    let mut archive=helpers::get_zip(path).map_err(|e|format!("error in poening zip: {}", e))?;
    let opf_struct=helpers::get_opf_struct(&mut archive).map_err(|e|format!("error in getting opf struct: {}",e))?;
    let nav_file=opf_struct.manifest.item.iter().filter(|f|f.id.contains("nav.")).next();

    if let Some(f)=nav_file{
        let mut nav_file=archive.by_name(&f.href).map_err(|e|format!("error in getting nav_file: {}",e))?;
        let mut buf=Default::default();
        nav_file.read_to_string(&mut buf)?;
        let nav:Html=quick_xml::de::from_str(&buf).map_err(|e|format!("Error in reading xml to nav: {}", e))?;

        let toc_nav = nav.body.nav.iter()
            .find(|n| n.epub_type == "toc")
            .ok_or("No nav with epub:type='toc' found")?;

        let toc=nav_to_ncx(&toc_nav, nav.head.title.clone())?;
        let toc_xml=quick_xml::se::to_string(&toc)?;
        rewrite_epub_with_new_file(path, &Path::new(&f.href), &toc_xml).map_err(|e|format!("error in rewriting: {}", e))?;

    }else{
        let opf_path=helpers::read_container_opf_path(&mut archive).map_err(|e|format!("Failed to get opf path: {}",e))?;
        let opf=helpers::get_opf_struct(&mut archive).map_err(|e|format!("failed to get opf struct: {}", e))?;
        let toc=spine_to_ncx(&opf).map_err(|e|format!("failed to generate ncx: {}",e))?;
        let toc_xml=quick_xml::se::to_string(&toc).map_err(|e|format!("failed to make toc xml: {}",e))?;

        rewrite_epub_with_new_file(path, &Path::new(&opf_path), &toc_xml).map_err(|e|format!("error in rewriting: {}", e))?;
    }
 
    Ok(())
}

use zip::write::SimpleFileOptions;
fn spine_to_ncx(opf: &Package) -> Result<Ncx, Box<dyn std::error::Error>> {
    let mut nav_points = Vec::new();

    for (index, itemref) in opf.spine.itemref.iter().enumerate() {
        // Resolve idref → manifest item
        let manifest_item = opf.manifest.item
            .iter()
            .find(|i| i.id == itemref.idref)
            .ok_or("Spine references missing manifest item")?;

        let chapter_number = index + 1;

        let nav_point = NavPoint {
            id: format!("navPoint-{}", chapter_number),
            play_order: chapter_number.to_string(),
            text: None,
            nav_label: NavLabel {
                text: format!("Chapter {}", chapter_number),
            },
            content: Content {
                src: manifest_item.href.clone(),
            },
        };

        nav_points.push(nav_point);
    }

    if nav_points.is_empty() {
        return Err("Spine contains no items".into());
    }

    Ok(Ncx {
        xmlns: "http://www.daisy.org/z3986/2005/ncx/".to_string(),
        text: None,
        head: Head {},
        doc_title: DocTitle {
            text: opf.metadata.title.clone(),
        },
        nav_map: NavMap {
            nav_point: nav_points,
        },
    })
}

pub fn rewrite_epub_with_new_file(
    path: &Path,
    anchor_file: &Path,
    new_contents: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let new_filename="toc.ncx";

    // Open original EPUB
    let original_file = File::open(path)?;
    let mut archive = ZipArchive::new(original_file)?;
    let opf_path=helpers::read_container_opf_path(&mut archive)?;
    
    
    // Temporary file path
    let tmp_path = path.with_extension("tmp");
    let tmp_file = File::create(&tmp_path)?;
    let mut writer = ZipWriter::new(tmp_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    let mut opf_buffer = String::new();
    // First pass: copy everything except toc.ncx and content.opf

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.contains(new_filename) || name.ends_with(".opf") {
            // skip, we will rewrite these
            if name.ends_with(".opf") {
                file.read_to_string(&mut opf_buffer)?;
            }
            continue;
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        writer.start_file(name, options)?;
        writer.write_all(&buffer)?;
    }


    let toc_path=anchor_file.with_file_name("toc.ncx");

    let opf_parent=Path::new(&opf_path).parent().ok_or("opf has no parent")?;

    let path_diff=pathdiff::diff_paths(&toc_path, opf_parent.to_path_buf())
        .ok_or("could not calculate the path diff")?;

    let diff=path_diff .to_str().ok_or("could not convert path diff buf to string")?;
    // --- Modify OPF ---
    let updated_opf = update_opf(&opf_buffer,diff)?;

    // Write updated OPF
    writer.start_file(opf_path, options)?;
    writer.write_all(updated_opf.as_bytes())?;

    // Write new toc.ncx
    writer.start_file(toc_path.to_string_lossy().to_string(), options)?;
    writer.write_all(new_contents.as_bytes())?;

    writer.finish()?;

    // Replace original
    helpers::move_file(&tmp_path, path)?;

    Ok(())
}

use quick_xml::{
    events::{ BytesStart, Event},
    Reader, Writer,
};
use std::io::Cursor;

fn update_opf(opf_xml: &str, toc_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(opf_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut inside_manifest = false;
    let mut manifest_has_ncx = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"manifest" => {
                inside_manifest = true;
                writer.write_event(Event::Start(e.clone()))?;
            }

            Event::End(ref e) if e.name().as_ref() == b"manifest" => {
                // If no ncx item found, inject before closing manifest
                if !manifest_has_ncx {
                    let mut ncx = BytesStart::new("item");
                    ncx.push_attribute(("id", "ncx"));
                    ncx.push_attribute(("href", toc_path));
                    ncx.push_attribute(("media-type", "application/x-dtbncx+xml"));
                    writer.write_event(Event::Empty(ncx))?;
                }

                inside_manifest = false;
                writer.write_event(Event::End(e.clone()))?;
            }

            Event::Empty(ref e) if inside_manifest && e.name().as_ref() == b"item" => {
                // Detect existing NCX
                for attr in e.attributes().with_checks(false) {
                    let attr = attr?;
                    let value: &[u8] = attr.value.as_ref();
                    if attr.key.as_ref() == b"id" && value == b"ncx" {
                        manifest_has_ncx = true;
                    }
                }
                writer.write_event(Event::Empty(e.clone()))?;
            }

            Event::Start(ref e) if e.name().as_ref() == b"spine" => {

                let mut spine = e.clone();
                let mut has_toc_attr = false;

                for attr in spine.attributes().with_checks(false) {
                    let attr = attr?;
                    if attr.key.as_ref() == b"toc" {
                        has_toc_attr = true;
                    }
                }

                if has_toc_attr {
                    spine.clear_attributes();
                    spine.push_attribute(("toc", "ncx"));
                } else {
                    spine.push_attribute(("toc", "ncx"));
                }

                writer.write_event(Event::Start(spine))?;
            }

            Event::Eof => break,

            ev => {
                writer.write_event(ev)?;
            }
        }

        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

use urlencoding::decode;
fn nav_to_ncx(nav: &Nav, head:String) -> Result<Ncx, Box<dyn std::error::Error>> {
    // Ensure this is actually a TOC nav
    if nav.epub_type != "toc" {
        return Err("Nav is not a TOC (epub:type != toc)".into());
    }

    let mut nav_points = Vec::new();

    for (index, li) in nav.ol.li.iter().enumerate() {
        let href = decode(li.a.href.trim())?.into_owned();
        let text = li.a.text
            .as_deref()
            .unwrap_or("")
            .trim();

        // Enforce your invariants (same rules as verifier)
        if text.is_empty() {
            return Err("Nav entry text is empty".into());
        }

        if text.len() >= 100 {
            return Err("Nav entry text too long".into());
        }

        if href.is_empty() {
            return Err("Nav entry href is empty".into());
        }

        let nav_point = NavPoint {
            id: format!("navPoint-{}", index + 1),
            play_order: (index + 1).to_string(),
            text: None, // optional text between elements (not needed)
            nav_label: NavLabel {
                text: text.to_string(),
            },
            content: Content {
                src: href.to_string(),
            },
        };

        nav_points.push(nav_point);
    }

    if nav_points.is_empty() {
        return Err("Nav contains no entries".into());
    }

    Ok(Ncx {
        xmlns: "http://www.daisy.org/z3986/2005/ncx/".to_string(),
        text: None,
        head: Head {},
        doc_title: DocTitle {
            text: head,
        },
        nav_map: NavMap {
            nav_point: nav_points,
        },
    })
}