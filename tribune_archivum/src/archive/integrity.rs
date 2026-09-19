use crate::core::{ArchiveInfo, ValidationResult,ValidationError,ErrorCode,ValidationLocation};
use std::collections::HashMap;
use zip::ZipArchive;

/// Validates the ZIP archive integrity of an EPUB file
pub fn validate_archive(path: &str) -> Result<ArchiveInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Try to open the ZIP archive
    let file = std::fs::File::open(path).map_err(|_| {
        result.add_error(ValidationError::new(
            ErrorCode::InvalidZip,
            ValidationLocation::Root,
        ));
        result.clone()
    })?;
    
    let mut archive = ZipArchive::new(file).map_err(|_| {
        result.add_error(ValidationError::new(
            ErrorCode::InvalidZip,
            ValidationLocation::Root,
        ));
        result.clone()
    })?;
    let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    // Check for duplicate paths and collect all entries
    let mut entries: HashMap<String, usize> = HashMap::new();
    let mut duplicate_paths: Vec<String> = Vec::new();
    
    for i in 0..archive.len() {
        match archive.by_index(i) {
            Ok(entry) => {
                let path = entry.name().to_string();
                
                // Check for duplicates
                let count = entries.entry(path.clone()).or_insert(0);
                *count += 1;
                if *count == 2 {
                    duplicate_paths.push(path.clone());
                }
            }
            Err(_) => {
                // Try to get the entry name for reporting
                let entry_name = file_names.get(i).cloned().unwrap_or_else(|| "unknown".into());

                result.add_error(ValidationError::new(
                    ErrorCode::UnreadableEntry,
                    ValidationLocation::ArchiveEntry { path: entry_name },
                ));
            }
        }
    }
    
    // Report duplicate paths as errors
    for path in &duplicate_paths {
        result.add_error(ValidationError::new(
            ErrorCode::DuplicatePath,
            ValidationLocation::ArchiveEntry { path: path.clone() },
        ));
    }
    
    let archive_info = ArchiveInfo {
        entries: archive.file_names().map(|s| s.to_string()).collect(),
        duplicate_paths,
    };
    
    Ok(archive_info)
}
