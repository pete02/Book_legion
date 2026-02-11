use std::{fs::File, io::Read, path::Path};

use zip::{ZipArchive, read::ZipFile};
use anyhow::{Result, bail};


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


pub fn verify_integrity(path: &Path)-> Result<bool,Box<dyn std::error::Error>>{
    validate_zip_safety(path)?;

    let file=File::open(path)?;
    
    let mut archive=ZipArchive::new(file)?;
    let names:Vec<String>=archive.file_names().map(|s|s.to_string()).collect();


    let container_file = match archive.by_name("META-INF/container.xml") {
        Ok(f) => f,
        Err(_) => return Err("container.xml does not exist".into())
    };

    let opf=read_container_opf_path(container_file)?;
    

    
    if !names.contains(&"toc.ncx".to_string()){
       return Ok(true)
    }



    Ok(false)
}



use quick_xml::de::from_reader;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Container {
    #[serde(rename = "rootfiles")]
    rootfiles: Rootfiles,
}

#[derive(Debug, Deserialize)]
struct Rootfiles {
    #[serde(rename = "rootfile")]
    rootfile: Vec<Rootfile>,
}

#[derive(Debug, Deserialize)]
struct Rootfile {
    #[serde(rename = "full-path", default)]
    full_path: String,

    #[serde(rename = "media-type", default)]
    media_type: String,
}

/// Checks that META-INF/container.xml exists and returns the OPF path
pub fn read_container_opf_path(container_file: ZipFile<'_, File>) -> Result<String> {

    let mut reader = std::io::BufReader::new(container_file);
    let container: Container = from_reader(&mut reader)
        .map_err(|e| anyhow::anyhow!("Failed to parse container.xml: {}", e))?;

    if container.rootfiles.rootfile.is_empty() {
        bail!("No rootfile entry in container.xml");
    }

    // Usually the first rootfile is the main OPF
    let opf_path = container.rootfiles.rootfile[0].full_path.clone();

    if opf_path.is_empty() {
        bail!("Rootfile full-path is empty");
    }

    Ok(opf_path)
}