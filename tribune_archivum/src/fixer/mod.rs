
use std::{default, error::Error, fs::File};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
use std::path::{Path, PathBuf};
use std::io::Write;


use crate::validator::{errors::{ ManifestError::{self, DuplicateId, MissingItem}}, read_zip_text};

fn fix_manifest(zip: &str, opf_path: &str, Errors:Vec<ManifestError>)->Result<(),Box<dyn  Error>>{
    let file=File::open(zip)?;
    let mut archive=ZipArchive::new(file)?;
    let mut opf=read_zip_text(&mut archive, opf_path)?;

    
    for error in Errors{
        match error {
            DuplicateId(id)=>opf=remove_duplicate_id_item(&id, &opf, opf_path,&mut archive)?,
            MissingItem(id)=>opf=fix_missing_id_items(&id, &opf, opf_path, &mut archive)?,
            _default=>{}
        }
    }

    write_to_zip(zip, opf_path, &opf)?;

    Ok(())
}

use xmltree::{Element, EmitterConfig, XMLNode};

fn remove_duplicate_id_item(
    id: &str,
    opf: &str,
    opf_path: &str,
    archive: &mut ZipArchive<File>,
) -> Result<String, Box<dyn Error>> {
    let mut root= Element::parse(opf.as_bytes())?;
    let manifest = root.get_mut_child("manifest").ok_or("no manifest")?;

    let mut master=false;
    let mut master_href = None;

    manifest.children.retain(|node|{
        let XMLNode::Element(e) = node else { return true };
        if e.name != "item" { return true }

        let Some(href) = e.attributes.get("href") else { return true };

        let Some(item_id) = e.attributes.get("id") else { return true };

        if item_id!=id{
            return  true;
        }

        match &master_href {
            None => {
                master_href = Some(href.clone());
                true
            },
            Some(master) => href != master,
        }
    });

    let mut out = Vec::new();
    root.write_with_config(&mut out, EmitterConfig::new().perform_indent(true))?;
    Ok(String::from_utf8(out)?)
}


use uuid::Uuid;



fn fix_missing_id_items(id: &str, opf: &str, opf_path:&str, archive: &mut ZipArchive<File>)->Result<String, Box< dyn Error>>{
    let mut root= Element::parse(opf.as_bytes())?;
    let manifest = root.get_mut_child("manifest").ok_or("no manifest")?;

    manifest.children.retain(|n|{
        let XMLNode::Element(e)= n else {return true};
        if e.name != "item" {return  true;}
        e.attributes.contains_key("href")
    });

    for child in &mut manifest.children {
        let Some(el) = child.as_mut_element() else { continue };
        let Some(href) = el.attributes.get("href").cloned() else { continue };

        if el.name == "item" && !el.attributes.contains_key("id") {
            el.attributes.insert(
                "id".to_string(),
                Uuid::new_v4().to_string()
            );
        }

        if !el.attributes.contains_key("media-type") {
            let media_type = mime_guess::from_path(href)
                .first_or_octet_stream()
                .to_string();

            el.attributes.insert("media-type".to_string(), media_type);
        }

    }

    let mut out = Vec::new();
    root.write_with_config(&mut out, EmitterConfig::new().perform_indent(true))?;
    Ok(String::from_utf8(out)?)
}


fn resolve_href(opf_path: &str, href: &str) -> PathBuf {
    Path::new(opf_path)
        .parent()
        .unwrap_or(Path::new(""))
        .join(href)
}

fn write_to_zip(
    zip_path: &str,
    file_path: &str,
    content: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let temp_path = format!("{zip_path}.tmp");
    let temp_file = File::create(&temp_path)?;
    let mut writer = ZipWriter::new(temp_file);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        let options = SimpleFileOptions::default()
            .compression_method(entry.compression());

        writer.start_file(&name, options)?;

        if name == file_path {
            writer.write_all(content.as_bytes())?;
        } else {
            std::io::copy(&mut entry, &mut writer)?;
        }
    }

    writer.finish()?;
    std::fs::rename(temp_path, zip_path)?;

    Ok(())
}

#[cfg(test)]
mod fixer_tests;