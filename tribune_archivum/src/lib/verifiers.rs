use std::{fs::File, path::{Component, Path, PathBuf}};
use zip::ZipArchive;
use anyhow::{Result, bail};

use crate::lib::helpers;


const MAX_FILES: usize = 2000;
const MAX_TOTAL_UNCOMPRESSED: u64 = 512 * 1024 * 1024; // 512MB
const MAX_SINGLE_FILE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_COMPRESSION_RATIO: f64 = 100.0;


pub fn validate_zip_safety(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let file_count = archive.len();

    if file_count > MAX_FILES {
        bail!("ZIP contains too many files: {}", file_count);
    }

    let mut total_uncompressed = 0u64;

    for i in 0..file_count {
        let entry = archive.by_index(i)?;

        let compressed = entry.compressed_size();
        let uncompressed = entry.size();
        total_uncompressed += uncompressed;
        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
            bail!("ZIP expands beyond allowed total size");
        }

        if uncompressed > MAX_SINGLE_FILE {
            bail!("File too large: {}", entry.name());
        }

        if compressed > 0 {
            let ratio = uncompressed as f64 / compressed as f64;
            if ratio > MAX_COMPRESSION_RATIO {
                bail!(
                    "Suspicious compression ratio in {} ({}x)",
                    entry.name(),
                    ratio
                );
            }
        }

        if entry.name().contains("..") || entry.name().starts_with('/') {
            bail!("Path traversal detected in {}", entry.name());
        }
    }

    Ok(())
}

pub fn verify_zip_integrity(path: &Path)-> Result<bool,Box<dyn std::error::Error>>{

    let mut archive=helpers::get_zip(path)?;
    let names:Vec<String>=archive.file_names().map(|s|s.to_string()).collect();

    let opf_file=helpers::read_container_opf_path(&mut archive)?;
    let opf_dir=Path::new(&opf_file);
    let opf_struct = helpers::get_opf_struct(&mut archive)?;
    let mut ids=Vec::new();

    if opf_struct.spine.toc.len()==0{
       return Ok(true)
    }

    let mut toc_file="".to_owned();
    for item in opf_struct.manifest.item {
        let href=opf_dir.with_file_name(item.href);

        if !names.contains(&href.to_string_lossy().to_string()) {
            return Err(format!("Missing file in zip: {}", href.to_string_lossy().to_string()).into());
        }
        ids.push(item.id.clone());

        if item.id==opf_struct.spine.toc{
            toc_file=href.to_string_lossy().to_string();
        }
    }

    for itemref in opf_struct.spine.itemref{
        if !ids.contains(&itemref.idref){
            println!("hrefs: {:?}",ids);
            return Err(format!("missing idref: {}", itemref.idref).into())

        }
    }    

    if opf_struct.spine.toc.len()==0 || !names.contains(&toc_file){
        return Ok(true);
    }

    Ok(false)
}


fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut stack = Vec::new();

    for comp in path.components() {
        match comp {
            Component::ParentDir => { stack.pop(); }   // resolve ".."
            Component::CurDir => {}                     // skip "."
            Component::Normal(s) => stack.push(s),
            _ => {}                                     // ignore RootDir and Prefix
        }
    }

    let mut normalized = PathBuf::new();
    for s in stack {
        normalized.push(s);
    }
    normalized
}

pub fn verify_toc_integrity(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = helpers::get_zip(path).map_err(|e|format!("fdailed to get archive: {e}"))?;
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    let opf_struct = helpers::get_opf_struct(&mut archive)?;
    let opf_path=helpers::read_container_opf_path(&mut archive)?;
    // Find TOC file in manifest
    let mut toc_file = opf_struct
        .manifest
        .item
        .iter()
        .find(|item| item.id == opf_struct.spine.toc)
        .ok_or("no toc file found")?
        .href
        .clone();

    let toc_path=Path::new(&opf_path).with_file_name(toc_file);

    let normalized=normalize_relative_path(&toc_path);

    toc_file=normalized.to_string_lossy().to_string();

    let toc_file_handle = archive.by_name(&toc_file).map_err(|e| format!("could not get toc handle {}: {}", toc_file, e))?;
    let toc = helpers::read_toc(toc_file_handle).map_err(|e|format!("could not read ToC: {}",e))?;

    if toc.nav_map.nav_point.is_empty() {
        return Err("ToC did not contain any nav points".into());
    }


    for nav_point in toc.nav_map.nav_point {
        // Resolve relative path: src in NCX is relative to OPF
        let resolved_path = normalized.with_file_name(&nav_point.content.src);
        let normalized_path = normalize_relative_path(&resolved_path).to_string_lossy().to_string();

        if !names.contains(&normalized_path) {
            return Err(format!(
                "Nav point '{}' not found in EPUB (resolved path: '{}')",
                nav_point.content.src, normalized_path
            )
            .into());
        }

        // Validate play order
        validate_playorder(&nav_point.play_order)
            .map_err(|_| format!("play order is not a number: {}", nav_point.play_order))?;

        // Validate nav label text
        if nav_point.nav_label.text.len() > 100 {
            return Err("navpoint text too long".into());
        }
        if nav_point.nav_label.text.is_empty() {
            return Err("navpoint text not existing".into());
        }
    }

    Ok(())
}




fn validate_playorder(play_order: &str) -> Result<u32, String> {
    match play_order.parse::<u32>() {
        Ok(num) if num > 0 => Ok(num),
        Ok(_) => Err(format!("playOrder must be positive: '{}'", play_order)),
        Err(_) => Err(format!("Invalid playOrder, not a number: '{}'", play_order)),
    }
}