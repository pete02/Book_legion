use zip::ZipArchive;

/// Reads an archive entry by path
pub fn read_entry(archive: &mut ZipArchive<std::fs::File>, path: &str) -> Result<Vec<u8>, String> {
    match archive.by_name(path) {
        Ok(mut entry) => {
            let mut contents = Vec::new();
            if let Err(e) = std::io::Read::read_to_end(&mut entry, &mut contents) {
                Err(format!("Failed to read {}: {}", path, e))
            } else {
                Ok(contents)
            }
        }
        Err(e) => Err(format!("Entry not found: {}", e)),
    }
}

/// Checks if an entry exists in the archive
pub fn entry_exists(archive: &mut ZipArchive<std::fs::File>, path: &str) -> bool {
    archive.by_name(path).is_ok()
}

/// Gets all entries matching a pattern
pub fn find_entries(archive: &ZipArchive<std::fs::File>, pattern: &str) -> Vec<String> {
    archive
        .file_names()
        .filter(|name: &&str| name.contains(pattern))
        .map(|s| s.to_string())
        .collect()
}
