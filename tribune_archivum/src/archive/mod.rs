
use std::{
    io::{Cursor,Read},
    path::{Component, Path, PathBuf}
};
use zip::{ZipArchive,read::ZipFile};




#[derive(Debug, PartialEq)]
pub enum ArchiveError{
    NotAZip,
    TooManyFiles,
    BadEntry,
    SymLink,
    CompressionRate,
    EntrySizeLimit,
    SizeLimit,
    PathTraversal,
    AbsolutePath,
    BackSlash,
    NullByte,
    ControlCharacter
}


pub struct Limits {
    size_limit: u64,
    per_entry_limit: u64,
    compression_rate: u64,
    entry_count: usize
}
impl Default for Limits{
    fn default()->Self{
        Self { 
            size_limit: 500*1000,
            per_entry_limit: 100*100,
            compression_rate: 100,
            entry_count: 10000
        }
    }
}

fn is_symlink<R: Read>(file: &ZipFile<'_, R>) -> bool {
    file.unix_mode()
        .is_some_and(|mode| mode & 0o170000 == 0o120000)
}

fn verify_archive(bytes: &[u8], limits: &Limits) -> Result<Vec<String>, ArchiveError>{
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|_| ArchiveError::NotAZip)?;
    let mut files= Vec::new();
    let mut total_size = 0u64;


    if zip.len() > limits.entry_count{
        return Err(ArchiveError::TooManyFiles);
    }

    for i in 0..zip.len(){
        let entry=zip.by_index(i).map_err(|_| ArchiveError::BadEntry)?;
        let (name, size)=verify_entry(&entry, limits)?;

        total_size +=size;

        if total_size > limits.size_limit{
            return Err(ArchiveError::SizeLimit);
        }
        
        files.push(name);   
    }
    Ok(files)
}

fn verify_entry<R: Read>(entry: &ZipFile<'_, R>, limits: &Limits)->Result<(String,u64), ArchiveError>{
    if is_symlink(entry){
        return Err(ArchiveError::SymLink)
    }
    
    let size = entry.size() ;
    let compressed_size = entry.compressed_size();

    if size > limits.per_entry_limit {
        return Err(ArchiveError::EntrySizeLimit);
    }

    if compressed_size == 0 {
        if size > 0 {
            return Err(ArchiveError::CompressionRate);
        }
    } else if size > limits.compression_rate.saturating_mul(compressed_size) {
        return Err(ArchiveError::CompressionRate);
    }
    let name=normalize_path(entry.name())?;

    Ok((name.to_string_lossy().into_owned(), size))
}





fn normalize_path(path: &str) -> Result<PathBuf, ArchiveError> {
    if path.contains('\\'){
        return Err(ArchiveError::BackSlash);
    }

    if path.contains('\0'){
        return Err(ArchiveError::NullByte);
    }
    if path.chars().any(char::is_control) {
        return Err(ArchiveError::ControlCharacter);
    }

    let path = Path::new(path);

    if path.is_absolute() {
        return Err(ArchiveError::AbsolutePath);
    }

    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(ArchiveError::PathTraversal);
                }
            }
            Component::Normal(component) => {
                normalized.push(component);
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ArchiveError::AbsolutePath);
            }
        }
    }

    Ok(normalized)
}


#[cfg(test)]
#[path = "archive_tests.rs"]
mod archive_tests;