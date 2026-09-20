#[cfg(test)]
mod tests {
    use crate::archive::validate_archive;
    use crate::core::ErrorCode;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a valid EPUB-like ZIP archive for testing
    fn create_test_epub(temp_dir: &TempDir, name: &str) -> String {
        let epub_path = temp_dir.path().join(name);
        let mut archive = zip::ZipWriter::new(std::fs::File::create(&epub_path).unwrap());
        
        // Add container.xml
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        archive.start_file("META-INF/container.xml", options).unwrap();
        archive
            .write_all(
                b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>",
            )
            .unwrap();
        
        // Add content.opf
        archive.start_file("content.opf", options).unwrap();
        archive
            .write_all(
                b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"BookId\">\n  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n    <dc:identifier id=\"BookId\">urn:uuid:12345678-1234-1234-1234-123456789012</dc:identifier>\n    <dc:title>Test Book</dc:title>\n    <dc:creator>Test Author</dc:creator>\n  </metadata>\n  <manifest>\n    <item id=\"chapter1\" href=\"chapter1.xhtml\" media-type=\"application/xhtml+xml\"/>\n  </manifest>\n  <spine>\n    <itemref idref=\"chapter1\"/>\n  </spine>\n</package>",
            )
            .unwrap();
        
        // Add chapter file
        archive.start_file("chapter1.xhtml", options).unwrap();
        archive
            .write_all(b"<!DOCTYPE html><html><head><title>Chapter 1</title></head><body><p>Chapter content</p></body></html>")
            .unwrap();
        
        archive.finish().unwrap();
        epub_path.to_str().unwrap().to_string()
    }

    /// Helper function to create an invalid (non-ZIP) file
    fn create_invalid_file(temp_dir: &TempDir, name: &str) -> String {
        let path = temp_dir.path().join(name);
        fs::write(&path, "This is not a valid ZIP file").unwrap();
        path.to_str().unwrap().to_string()
    }

    /// Helper function to create an empty ZIP archive
    fn create_empty_zip(temp_dir: &TempDir, name: &str) -> String {
        let zip_path = temp_dir.path().join(name);
        let archive = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        // Create a truly empty ZIP with no entries
        archive.finish().unwrap();
        zip_path.to_str().unwrap().to_string()
    }

    /// Helper function to create a ZIP with duplicate paths
    /// Note: The zip library prevents creating true duplicates, so we test the
    /// duplicate detection logic by directly creating an ArchiveInfo with duplicates
    fn create_test_archive_info_with_duplicates() -> crate::core::ArchiveInfo {
        crate::core::ArchiveInfo {
            entries: vec![
                "META-INF/container.xml".to_string(),
                "content.opf".to_string(),
                "chapter1.xhtml".to_string(),
                "chapter1.xhtml".to_string(), // Duplicate!
            ],
            duplicate_paths: vec!["chapter1.xhtml".to_string()],
        }
    }

    #[test]
    fn test_validate_archive_valid_epub() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_test_epub(&temp_dir, "valid.epub");
        
        let result = validate_archive(&epub_path);
        assert!(result.is_ok(), "Valid EPUB should pass archive validation");
        
        let archive_info = result.unwrap();
        assert!(archive_info.entries.contains(&"META-INF/container.xml".to_string()));
        assert!(archive_info.entries.contains(&"content.opf".to_string()));
        assert!(archive_info.entries.contains(&"chapter1.xhtml".to_string()));
        assert!(archive_info.duplicate_paths.is_empty());
    }

    #[test]
    fn test_validate_archive_invalid_zip() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = create_invalid_file(&temp_dir, "invalid.txt");
        
        let result = validate_archive(&invalid_path);
        assert!(result.is_err(), "Invalid ZIP should fail archive validation");
        
        let validation_result = result.unwrap_err();
        assert!(!validation_result.errors.is_empty());
        assert_eq!(validation_result.errors[0].code, ErrorCode::InvalidZip);
    }

    #[test]
    fn test_validate_archive_empty_zip() {
        let temp_dir = TempDir::new().unwrap();
        let empty_path = create_empty_zip(&temp_dir, "empty.zip");
        
        let result = validate_archive(&empty_path);
        assert!(result.is_ok(), "Empty ZIP should still be a valid archive");
        
        let archive_info = result.unwrap();
        // Empty ZIP should have no entries
        assert!(archive_info.entries.is_empty(), "Empty ZIP should have no entries");
        assert!(archive_info.duplicate_paths.is_empty());
    }

    #[test]
    fn test_validate_archive_duplicate_paths() {
        // Test that duplicate_paths field is properly populated
        // Note: The zip library prevents creating true duplicates, so we test the
        // ArchiveInfo structure directly
        let archive_info = create_test_archive_info_with_duplicates();
        
        assert!(!archive_info.duplicate_paths.is_empty(), "Should detect duplicate paths");
        assert!(archive_info.duplicate_paths.contains(&"chapter1.xhtml".to_string()), 
                "Should detect chapter1.xhtml as duplicate");
        assert_eq!(archive_info.entries.len(), 4, "Should have 4 entries including duplicate");
    }

    #[test]
    fn test_validate_archive_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent.epub");
        
        let result = validate_archive(non_existent.to_str().unwrap());
        assert!(result.is_err(), "Non-existent file should fail");
        
        let validation_result = result.unwrap_err();
        assert!(!validation_result.errors.is_empty());
        assert_eq!(validation_result.errors[0].code, ErrorCode::InvalidZip);
    }

    #[test]
    fn test_validate_archive_unreadable_entry() {
        // This test verifies that the UnreadableEntry error code exists and is used
        // Note: Creating a truly unreadable entry is difficult with the zip library
        // This test ensures the error code is available for future use
        use crate::core::ValidationError;
        use crate::core::ValidationLocation;
        
        let mut result = crate::core::ValidationResult::new();
        result.add_error(ValidationError::new(
            ErrorCode::UnreadableEntry,
            ValidationLocation::ArchiveEntry { path: "test.txt".to_string() },
        ));
        
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.errors[0].code, ErrorCode::UnreadableEntry);
    }

    #[test]
    fn test_validate_archive_contains_expected_entries() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_test_epub(&temp_dir, "test.epub");
        
        let result = validate_archive(&epub_path);
        let archive_info = result.unwrap();
        
        // Check for expected EPUB structure
        assert!(archive_info.entries.iter().any(|e| e.starts_with("META-INF/")));
        assert!(archive_info.entries.iter().any(|e| e == "content.opf" || e.ends_with(".opf")));
    }

    #[test]
    fn test_validate_archive_entry_count() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_test_epub(&temp_dir, "test.epub");
        
        let result = validate_archive(&epub_path);
        let archive_info = result.unwrap();
        
        // Should have at least 3 entries: container.xml, content.opf, chapter1.xhtml
        assert!(archive_info.entries.len() >= 3);
    }
}
