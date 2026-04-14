use crate::lib::verifiers::validate_zip_safety;
use std::{fs::File, io::{BufReader, Read}, path::Path};

use log::debug;
use regex::Regex;
use zip::{ZipArchive, read::ZipFile};
use anyhow::Result;


pub fn read_toc(toc_file: ZipFile<'_, File>)->Result<Ncx,Box<dyn std::error::Error>>{
    let mut reader = std::io::BufReader::new(toc_file);
    let mut buf = Default::default();
    reader.read_to_string(&mut buf)?;
    Ok(quick_xml::de::from_str(&buf)?)
}

/// Checks that META-INF/container.xml exists and returns the OPF path
pub fn read_container_opf_path(archive:&mut ZipArchive<File>) -> Result<String, Box<dyn std::error::Error>> {
    let container_file =archive.by_name("META-INF/container.xml").map_err(|_|"container.xml does not exist")?;
    let mut reader = std::io::BufReader::new(container_file);
    let mut buf = Default::default();
    reader.read_to_string(&mut buf)?;
    let re = Regex::new(r#"<rootfile[^>]*\sfull-path="([^"]+)""#).unwrap();

    if let Some(caps) = re.captures(&buf) {
        let opf_path = caps.get(1).unwrap().as_str();
        if opf_path.is_empty() {
            return Err("Rootfile full-path is empty".into());
        }
        Ok(opf_path.to_string())
    } else {
        return Err("No rootfile element with full-path found".into());
    }
}

use crate::lib::{opf_model::Package, toc_model::Ncx};

pub fn read_opf_manifest(mut opf_file: ZipFile<'_, File>) -> Result<Package, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(&mut opf_file);
    let mut str= Default::default();
    reader.read_to_string(&mut str)?;

    //println!("xlm: {}", str);

    let opf_struct: Package=quick_xml::de::from_str(&str)?;
    Ok(opf_struct)
}

pub fn get_opf_struct(archive:&mut ZipArchive<File>)->Result<Package,Box<dyn std::error::Error>>{
    let opf=read_container_opf_path(archive)?;
    let opf_file= archive.by_name(&opf).map_err(|_|"listed opf file does not exist")?;
    let opf_struct = read_opf_manifest(opf_file)?;
    return Ok(opf_struct);
}

pub fn get_zip(path:&Path)->Result<ZipArchive<File>,Box<dyn std::error::Error>>{
    debug!("Get zip: {:?}", path);
    validate_zip_safety(path)?;
    debug!("zip {:?} is safe",path);
    let file=File::open(path)?;
    debug!("zip {:?} opened",path);
    return Ok(ZipArchive::new(file)?)
}

pub fn move_file(src: &Path, dst: &Path) -> Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            debug!("rename crossed devices, falling back to copy+delete");
            std::fs::copy(src, dst)?;
            std::fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}