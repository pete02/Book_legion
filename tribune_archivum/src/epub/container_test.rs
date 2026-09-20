#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{entry_exists, read_entry};
    use crate::core::{ErrorCode, OpfInfo, ValidationLocation, ValidationResult, ValidationError};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a minimal valid container.xml content
    fn create_valid_container_xml(opf_path: &str) -> Vec<u8> {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opf:endpoints">
  <rootfiles>
    <rootfile full-path="{}" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
            opf_path
        )
        .into_bytes()
    }

    /// Creates a temporary EPUB file with the given container.xml content
    fn create_temp_epub(container_xml: &[u8], opf_path: &str) -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = temp_dir.path().join("test.epub");
        
        // Create a minimal EPUB structure
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        // Add container.xml
        archive.start_file("META-INF/container.xml", options).unwrap();
        archive.write_all(container_xml).unwrap();
        
        // Add OPF file (minimal)
        let opf_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        );
        
        archive.start_file(opf_path, options).unwrap();
        archive.write_all(opf_content.as_bytes()).unwrap();
        
        // Add a chapter file
        archive.start_file("chapter1.xhtml", options).unwrap();
        archive.write_all(b"<html><body><p>Chapter 1</p></body></html>").unwrap();
        
        // Write to file
        let buffer = archive.finish().unwrap();
        let mut file = File::create(&epub_path).unwrap();
        file.write_all(&buffer).unwrap();
        
        (temp_dir, epub_path.to_str().unwrap().to_string())
    }

    #[test]
    fn test_valid_container_xml() {
        let (temp_dir, epub_path) = create_temp_epub(
            &create_valid_container_xml("OEBPS/content.opf"),
            "OEBPS/content.opf",
        );
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_ok(), "Valid container.xml should pass validation");
        let opf_info = result.unwrap();
        assert_eq!(opf_info.opf_path, "OEBPS/content.opf");
        
        drop(temp_dir);
    }

    #[test]
    fn test_missing_container_xml() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = temp_dir.path().join("test.epub");
        
        // Create an EPUB without container.xml
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        
        // Add OPF file directly (no container.xml)
        let opf_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#;
        
        archive.start_file("OEBPS/content.opf", options).unwrap();
        archive.write_all(opf_content.as_bytes()).unwrap();
        
        let buffer = archive.finish().unwrap();
        let mut file = File::create(&epub_path).unwrap();
        file.write_all(&buffer).unwrap();
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_err(), "Missing container.xml should fail validation");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::MissingContainerXml &&
            matches!(e.location, ValidationLocation::Root)
        }), "Should report MissingContainerXml error");
    }

    #[test]
    fn test_invalid_container_xml() {
        let (temp_dir, epub_path) = {
            let temp_dir = TempDir::new().unwrap();
            let epub_path = temp_dir.path().join("test.epub");
            
            // Create EPUB with invalid container.xml
            let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            
            // Add invalid container.xml
            archive.start_file("META-INF/container.xml", options).unwrap();
            archive.write_all(b"not valid xml {{{").unwrap();
            
            let buffer = archive.finish().unwrap();
            let mut file = File::create(&epub_path).unwrap();
            file.write_all(&buffer).unwrap();
            
            (temp_dir, epub_path.to_str().unwrap().to_string())
        };
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_err(), "Invalid container.xml should fail validation");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::InvalidContainerXml
        }), "Should report InvalidContainerXml error");
    }

    #[test]
    fn test_missing_opf_reference() {
        let (temp_dir, epub_path) = {
            let temp_dir = TempDir::new().unwrap();
            let epub_path = temp_dir.path().join("test.epub");
            
            let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            
            // Add container.xml without rootfile
            let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opf:endpoints">
  <rootfiles>
  </rootfiles>
</container>"#;
            
            archive.start_file("META-INF/container.xml", options).unwrap();
            archive.write_all(container_xml.as_bytes()).unwrap();
            
            let buffer = archive.finish().unwrap();
            let mut file = File::create(&epub_path).unwrap();
            file.write_all(&buffer).unwrap();
            
            (temp_dir, epub_path.to_str().unwrap().to_string())
        };
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_err(), "Missing OPF reference should fail validation");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::MissingOpfReference
        }), "Should report MissingOpfReference error");
    }

    #[test]
    fn test_invalid_opf_reference() {
        let (temp_dir, epub_path) = create_temp_epub(
            &create_valid_container_xml("OEBPS/nonexistent.opf"),
            "OEBPS/nonexistent.opf",
        );
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_err(), "Invalid OPF reference should fail validation");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::InvalidOpfReference
        }), "Should report InvalidOpfReference error");
    }

    #[test]
    fn test_multiple_rootfiles() {
        let (temp_dir, epub_path) = {
            let temp_dir = TempDir::new().unwrap();
            let epub_path = temp_dir.path().join("test.epub");
            
            let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            
            // Add container.xml with multiple rootfiles (should take first)
            let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opf:endpoints">
  <rootfiles>
    <rootfile full-path="OEBPS/content1.opf" media-type="application/oebps-package+xml"/>
    <rootfile full-path="OEBPS/content2.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
            
            archive.start_file("META-INF/container.xml", options).unwrap();
            archive.write_all(container_xml.as_bytes()).unwrap();
            
            // Add both OPF files
            let opf_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#;
            
            archive.start_file("OEBPS/content1.opf", options).unwrap();
            archive.write_all(opf_content.as_bytes()).unwrap();
            
            let buffer = archive.finish().unwrap();
            let mut file = File::create(&epub_path).unwrap();
            file.write_all(&buffer).unwrap();
            
            (temp_dir, epub_path.to_str().unwrap().to_string())
        };
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_ok(), "Multiple rootfiles should accept first one");
        let opf_info = result.unwrap();
        assert_eq!(opf_info.opf_path, "OEBPS/content1.opf");
        
        drop(temp_dir);
    }

    #[test]
    fn test_container_xml_parsing() {
        let container_xml = create_valid_container_xml("OEBPS/content.opf");
        let container_str = String::from_utf8(container_xml.clone()).unwrap();
        
        // Test XML parsing
        let container: ContainerRoot = quick_xml::de::from_str(&container_str).unwrap();
        assert_eq!(container.root_files.root_files.len(), 1);
        assert_eq!(container.root_files.root_files[0].full_path, "OEBPS/content.opf");
    }

    #[test]
    fn test_container_xml_with_special_characters_in_path() {
        let (temp_dir, epub_path) = {
            let temp_dir = TempDir::new().unwrap();
            let epub_path = temp_dir.path().join("test.epub");
            
            let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            
            let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opf:endpoints">
  <rootfiles>
    <rootfile full-path="OEBPS/Content%20Folder/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
            
            archive.start_file("META-INF/container.xml", options).unwrap();
            archive.write_all(container_xml.as_bytes()).unwrap();
            
            let opf_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#;
            
            archive.start_file("OEBPS/Content%20Folder/content.opf", options).unwrap();
            archive.write_all(opf_content.as_bytes()).unwrap();
            
            let buffer = archive.finish().unwrap();
            let mut file = File::create(&epub_path).unwrap();
            file.write_all(&buffer).unwrap();
            
            (temp_dir, epub_path.to_str().unwrap().to_string())
        };
        
        let file = File::open(&epub_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        
        let result = validate_container(&mut archive);
        
        assert!(result.is_ok(), "Container with special characters in path should work");
        let opf_info = result.unwrap();
        assert_eq!(opf_info.opf_path, "OEBPS/Content%20Folder/content.opf");
        
        drop(temp_dir);
    }
}
