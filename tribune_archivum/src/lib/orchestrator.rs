use walkdir::WalkDir;
use crate::lib::{generator::generate_toc, verifiers::{verify_toc_integrity, verify_zip_integrity}};
use log::{info, warn, error, debug};
use std::fs;
use std::path::Path;
use tempfile::tempdir;


pub fn process_library(root: &Path, processed_dir: &Path, err_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Scanning library at {}", root.display());
    fs::create_dir_all(processed_dir)?;
    fs::create_dir_all(err_dir)?;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let temp_dir = tempdir()?;
        let path = entry.path();
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };

        let result = match ext.as_str() {
            "epub" => {
                debug!("Processing EPUB: {}", path.display());
                process_epub(path).map(|_| path.to_path_buf()) // return original EPUB path
            }
            "pdf" | "mobi" | "azw" | "azw3" | "kfx" => {
                info!("Converting to EPUB: {}", path.display());

                // Generate in temp folder, not in input folder
                let temp_epub_path = temp_dir.path().join(path.file_stem().unwrap()).with_extension("epub");
                convert_to_epub_to(path, &temp_epub_path)?; // new function taking output path

                process_epub(&temp_epub_path)?;
                
                Ok(temp_epub_path) // return temp EPUB path for copying
            }
            _ => continue,
        };

        match result {
            Ok(epub_path) => {
                // Only copy EPUBs to processed_dir
                let file_name = epub_path.file_name().unwrap();
                let dest_path = processed_dir.join(file_name);
                fs::copy(&epub_path, &dest_path)?;
                info!("Copied processed file to {}", dest_path.display());
            }
            Err(e) => {
                // Move original file to err_dir
                let file_name = path.file_name().unwrap();
                let dest_path = err_dir.join(file_name);
                fs::copy(path, &dest_path)?;
                error!("Failed to process {}: {}", path.display(), e);
            }
        }
    }

    info!("Library scan complete.");
    Ok(())
}

use std::process::Command;

pub fn convert_to_epub_to(input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Stdio;

    let status = Command::new("ebook-convert")
        .arg(input)
        .arg(output)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        error!("Calibre conversion failed: {}", input.display());
        return Err("Calibre conversion failed".into());
    }

    info!("Converted → {}", output.display());
    Ok(())
}

pub fn process_epub(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Verifying ZIP integrity: {}", path.display());

    let zip_ok = verify_zip_integrity(path)?;

    if !zip_ok {
        warn!("ZIP structure invalid, attempting TOC regeneration: {}", path.display());
        generate_toc(path)?;
        let zip_verify=verify_zip_integrity(path);
        // re-verify after modification
        if !zip_verify.is_ok() {
            error!("ZIP still invalid after regeneration: {}, {:?}", path.display(), zip_verify );
            return Err("ZIP integrity failed after regeneration".into());
        }

        let toc_ok=verify_toc_integrity(path);
        if !toc_ok.is_ok() {
            error!("TOC regeneration failed: {}, {:?}", path.display(), toc_ok);
            return Err("TOC still invalid after regeneration".into());
        }

        info!("Recovered invalid ZIP: {}", path.display());
        return Ok(());
    }else{
        match verify_toc_integrity(path) {
            Ok(_) => {
                debug!("TOC valid: {}", path.display());
            }
            Err(_) => {
                warn!("TOC invalid or missing, regenerating: {}", path.display());
                generate_toc(path)?;

                let zip_verify=verify_zip_integrity(path);
                // re-verify after modification
                if !zip_verify.is_ok() {
                    error!("ZIP still invalid after regeneration: {}, {:?}", path.display(), zip_verify );
                    return Err("ZIP integrity failed after regeneration".into());
                }

                let toc_ok=verify_toc_integrity(path);
                if !toc_ok.is_ok() {
                    error!("TOC regeneration failed: {}, {:?}", path.display(), toc_ok);
                    return Err("TOC still invalid after regeneration".into());
                }

                info!("TOC regenerated successfully: {}", path.display());
            }
        }
    }
    Ok(())
}