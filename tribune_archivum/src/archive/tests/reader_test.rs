#[cfg(test)]
mod tests {
    use crate::archive::{entry_exists, find_entries, read_entry};
    use std::io::Write;
    use tempfile::TempDir;
    use zip::ZipArchive;

    /// Helper function to create a test ZIP archive with known contents
    fn create_test_archive(temp_dir: &TempDir, name: &str) -> (String, zip::ZipArchive<std::fs::File>) {
        let zip_path = temp_dir.path().join(name);
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        // Add various entries
        archive.start_file("file1.txt", options).unwrap();
        archive.write_all(b"Content of file 1").unwrap();
        
        archive.start_file("file2.txt", options).unwrap();
        archive.write_all(b"Content of file 2").unwrap();
        
        archive.start_file("dir1/file3.txt", options).unwrap();
        archive.write_all(b"Content of file 3 in subdirectory").unwrap();
        
        archive.start_file("dir1/file4.txt", options).unwrap();
        archive.write_all(b"Content of file 4 in subdirectory").unwrap();
        
        archive.start_file("dir1/dir2/file5.txt", options).unwrap();
        archive.write_all(b"Content of file 5 in nested subdirectory").unwrap();
        
        archive.finish().unwrap();
        
        let file = std::fs::File::open(&zip_path).unwrap();
        let zip_archive = ZipArchive::new(file).unwrap();
        
        (zip_path.to_str().unwrap().to_string(), zip_archive)
    }

    #[test]
    fn test_read_entry_success() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let result = read_entry(&mut archive, "file1.txt");
        assert!(result.is_ok(), "Should successfully read existing file");
        
        let contents = result.unwrap();
        assert_eq!(contents, b"Content of file 1");
    }

    #[test]
    fn test_read_entry_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let result = read_entry(&mut archive, "nonexistent.txt");
        assert!(result.is_err(), "Should fail to read non-existent file");
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Entry not found"));
    }

    #[test]
    fn test_read_entry_from_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let result = read_entry(&mut archive, "dir1/file3.txt");
        assert!(result.is_ok(), "Should successfully read file from subdirectory");
        
        let contents = result.unwrap();
        assert_eq!(contents, b"Content of file 3 in subdirectory");
    }

    #[test]
    fn test_read_entry_from_nested_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let result = read_entry(&mut archive, "dir1/dir2/file5.txt");
        assert!(result.is_ok(), "Should successfully read file from nested subdirectory");
        
        let contents = result.unwrap();
        assert_eq!(contents, b"Content of file 5 in nested subdirectory");
    }

    #[test]
    fn test_entry_exists_true() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let exists = entry_exists(&mut archive, "file1.txt");
        assert!(exists, "file1.txt should exist in archive");
    }

    #[test]
    fn test_entry_exists_false() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let exists = entry_exists(&mut archive, "nonexistent.txt");
        assert!(!exists, "nonexistent.txt should not exist in archive");
    }

    #[test]
    fn test_entry_exists_from_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        let exists = entry_exists(&mut archive, "dir1/file3.txt");
        assert!(exists, "dir1/file3.txt should exist in archive");
    }

    #[test]
    fn test_find_entries_with_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        let results = find_entries(&archive, "dir1/");
        assert_eq!(results.len(), 3, "Should find 3 entries under dir1/");
        assert!(results.contains(&"dir1/file3.txt".to_string()));
        assert!(results.contains(&"dir1/file4.txt".to_string()));
        assert!(results.contains(&"dir1/dir2/file5.txt".to_string()));
    }

    #[test]
    fn test_find_entries_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        // Note: find_entries uses .contains(), so ".txt" will match anywhere in the path
        // This test verifies the current implementation behavior
        let results = find_entries(&archive, ".txt");
        // All 5 files end with .txt, so they should all be found
        assert_eq!(results.len(), 5, "Should find 5 .txt files");
        for result in &results {
            assert!(result.ends_with(".txt"), "All results should end with .txt");
        }
    }

    #[test]
    fn test_find_entries_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        let results = find_entries(&archive, ".xyz");
        assert!(results.is_empty(), "Should find no .xyz files");
    }

    #[test]
    fn test_find_entries_exact_match() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        let results = find_entries(&archive, "file1.txt");
        assert_eq!(results.len(), 1, "Should find exactly one file1.txt");
        assert_eq!(results[0], "file1.txt");
    }

    #[test]
    fn test_find_entries_case_sensitive() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        // Archive has "file1.txt" (lowercase)
        let results_lower = find_entries(&archive, "file1.txt");
        let results_upper = find_entries(&archive, "FILE1.TXT");
        
        assert_eq!(results_lower.len(), 1, "Should find file1.txt with lowercase");
        assert!(results_upper.is_empty(), "Should not find file1.txt with uppercase (case-sensitive)");
    }

    #[test]
    fn test_read_entry_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("empty.zip");
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        archive.start_file("empty.txt", options).unwrap();
        // Don't write any content
        archive.finish().unwrap();
        
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut zip_archive = ZipArchive::new(file).unwrap();
        
        let result = read_entry(&mut zip_archive, "empty.txt");
        assert!(result.is_ok(), "Should successfully read empty file");
        
        let contents = result.unwrap();
        assert!(contents.is_empty(), "Empty file should have empty contents");
    }

    #[test]
    fn test_read_entry_binary_content() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("binary.zip");
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        archive.start_file("binary.bin", options).unwrap();
        // Write binary data
        let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        archive.write_all(&binary_data).unwrap();
        archive.finish().unwrap();
        
        let file = std::fs::File::open(&zip_path).unwrap();
        let mut zip_archive = ZipArchive::new(file).unwrap();
        
        let result = read_entry(&mut zip_archive, "binary.bin");
        assert!(result.is_ok(), "Should successfully read binary file");
        
        let contents = result.unwrap();
        assert_eq!(contents, binary_data, "Binary content should match");
    }

    #[test]
    fn test_entry_exists_comprehensive() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, mut archive) = create_test_archive(&temp_dir, "test.zip");
        
        // Test various paths
        assert!(entry_exists(&mut archive, "file1.txt"));
        assert!(entry_exists(&mut archive, "file2.txt"));
        assert!(entry_exists(&mut archive, "dir1/file3.txt"));
        assert!(entry_exists(&mut archive, "dir1/dir2/file5.txt"));
        
        // Test non-existent paths
        assert!(!entry_exists(&mut archive, "missing.txt"));
        assert!(!entry_exists(&mut archive, "dir1/missing.txt"));
        assert!(!entry_exists(&mut archive, "dir1/dir2/dir3/missing.txt"));
    }

    #[test]
    fn test_find_entries_root_level() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        // Find entries at root level (no slashes)
        let results = find_entries(&archive, "file");
        assert!(results.contains(&"file1.txt".to_string()));
        assert!(results.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_find_entries_directory_only() {
        let temp_dir = TempDir::new().unwrap();
        let (_zip_path, archive) = create_test_archive(&temp_dir, "test.zip");
        
        // Find all entries containing "dir1"
        let results = find_entries(&archive, "dir1");
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.starts_with("dir1/")));
    }
}
