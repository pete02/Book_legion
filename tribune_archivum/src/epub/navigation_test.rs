use std::io::Write;
use tempfile::tempdir;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::archive::{entry_exists, read_entry};
use crate::core::{ErrorCode, ManifestInfo, OpfPackage, SpineItem, ValidationLocation, ValidationResult, ValidationError};

/// Helper function to create a test EPUB archive with navigation document
fn create_nav_archive(nav_content: &[u8], nav_filename: &str) -> (tempdir::TempDir, String) {
    let temp_dir = tempdir().unwrap();
    let epub_path = temp_dir.path().join("test.epub");
    
    {
        let file = std::fs::File::create(&epub_path).unwrap();
        let mut zip = ZipWriter::new(file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        // Add container.xml
        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(b r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();
        
        // Add nav document
        zip.start_file(nav_filename, options).unwrap();
        zip.write_all(nav_content).unwrap();
    }
    
    (temp_dir, epub_path.to_str().unwrap().to_string())
}

/// Helper function to create a test EPUB archive with NCX document
fn create_ncx_archive(ncx_content: &[u8], ncx_filename: &str) -> (tempdir::TempDir, String) {
    let temp_dir = tempdir().unwrap();
    let epub_path = temp_dir.path().join("test.epub");
    
    {
        let file = std::fs::File::create(&epub_path).unwrap();
        let mut zip = ZipWriter::new(file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        // Add container.xml
        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(b r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();
        
        // Add NCX document
        zip.start_file(ncx_filename, options).unwrap();
        zip.write_all(ncx_content).unwrap();
    }
    
    (temp_dir, epub_path.to_str().unwrap().to_string())
}

/// Helper function to create a valid EPUB3 nav document
fn create_valid_epub3_nav() -> Vec<u8> {
    br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type="toc" id="toc" role="doc-toc">
    <ol>
      <li>
        <a href="chapter1.xhtml">Chapter 1</a>
        <ol>
          <li><a href="chapter1.xhtml#section1">Section 1</a></li>
          <li><a href="chapter1.xhtml#section2">Section 2</a></li>
        </ol>
      </li>
      <li><a href="chapter2.xhtml">Chapter 2</a></li>
    </ol>
  </nav>
</body>
</html>"#.to_vec()
}

/// Helper function to create a valid EPUB2 NCX document
fn create_valid_epub2_ncx() -> Vec<u8> {
    br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:12345678-1234-1234-1234-123456789012"/>
    <meta name="dtb:depth" content="2"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>Test Book</text></docTitle>
  <navMap>
    <navPoint id="navpoint1" playOrder="1">
      <navLabel><text>Chapter 1</text></navLabel>
      <content src="chapter1.xhtml"/>
      <navPoint id="navpoint2" playOrder="2">
        <navLabel><text>Section 1</text></navLabel>
        <content src="chapter1.xhtml#section1"/>
      </navPoint>
      <navPoint id="navpoint3" playOrder="3">
        <navLabel><text>Section 2</text></navLabel>
        <content src="chapter1.xhtml#section2"/>
      </navPoint>
    </navPoint>
    <navPoint id="navpoint4" playOrder="4">
      <navLabel><text>Chapter 2</text></navLabel>
      <content src="chapter2.xhtml"/>
    </navPoint>
  </navMap>
</ncx>"#.to_vec()
}

/// Helper function to create a valid OpfPackage for testing
fn create_valid_opf() -> OpfPackage {
    OpfPackage {
        package_xml: "".to_string(),
        unique_identifier: "test-id".to_string(),
        metadata_xml: "".to_string(),
        title: "Test Book".to_string(),
        creators: vec!["Test Author".to_string()],
        language: vec!["en".to_string()],
        spine_items: vec![SpineItem {
            idref: "chapter1.xhtml".to_string(),
            properties: vec![],
            linear: "yes".to_string(),
            index: 0,
        }],
        manifest_items: vec![],
        guide_items: vec![],
        cover_items: vec![],
        page_breaks: vec![],
        reading_order: vec![],
        accessibility_features: vec![],
        accessibility_controls: vec![],
        other_metadata: vec![],
    }
}

/// Helper function to create a valid ManifestInfo for testing
fn create_valid_manifest_info() -> ManifestInfo {
    ManifestInfo {
        items: vec![],
        id_to_item: std::collections::HashMap::new(),
    }
}

mod tests {
    use super::*;
    use crate::epub::navigation::{validate_navigation, NavInfo};

    #[test]
    fn test_valid_epub3_nav() {
        let (temp_dir, epub_path) = create_nav_archive(&create_valid_epub3_nav(), "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected valid EPUB3 nav to pass validation");
        
        let nav_info = result.unwrap();
        assert!(nav_info.nav_path.is_some());
        assert!(nav_info.ncx_path.is_none());
        assert_eq!(nav_info.toc_entries.len(), 2);
        assert_eq!(nav_info.toc_entries[0].title, "Chapter 1");
        assert_eq!(nav_info.toc_entries[0].children.len(), 2);
        assert_eq!(nav_info.toc_entries[1].title, "Chapter 2");
    }

    #[test]
    fn test_valid_epub2_ncx() {
        let (temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
            index: 1,
        });
        
        let mut manifest_info = create_valid_manifest_info();
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/ncx.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected valid EPUB2 NCX to pass validation");
        
        let nav_info = result.unwrap();
        assert!(nav_info.nav_path.is_none());
        assert!(nav_info.ncx_path.is_some());
        assert_eq!(nav_info.toc_entries.len(), 2);
        assert_eq!(nav_info.toc_entries[0].title, "Chapter 1");
        assert_eq!(nav_info.toc_entries[0].children.len(), 2);
        assert_eq!(nav_info.toc_entries[1].title, "Chapter 2");
    }

    #[test]
    fn test_missing_nav_document() {
        let (temp_dir, epub_path) = create_nav_archive(b"invalid", "OEBPS/missing.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected missing nav document to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::MissingNavDocument));
    }

    #[test]
    fn test_invalid_epub3_nav_xml() {
        let (temp_dir, epub_path) = create_nav_archive(b"this is not valid xml {{{", "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected invalid EPUB3 nav XML to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::InvalidNavXml));
    }

    #[test]
    fn test_invalid_epub2_ncx_xml() {
        let (temp_dir, epub_path) = create_ncx_archive(b"this is not valid xml {{{", "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
            index: 1,
        });
        
        let mut manifest_info = create_valid_manifest_info();
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/ncx.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected invalid EPUB2 NCX XML to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::InvalidNcxXml));
    }

    #[test]
    fn test_empty_toc_epub3() {
        let empty_nav = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type="toc" id="toc" role="doc-toc">
  </nav>
</body>
</html>"#.to_vec();
        
        let (temp_dir, epub_path) = create_nav_archive(&empty_nav, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected empty TOC to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::EmptyToc));
    }

    #[test]
    fn test_empty_toc_ncx() {
        let empty_ncx = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:12345678-1234-1234-1234-123456789012"/>
    <meta name="dtb:depth" content="0"/>
  </head>
  <docTitle><text>Test Book</text></docTitle>
  <navMap>
  </navMap>
</ncx>"#.to_vec();
        
        let (temp_dir, epub_path) = create_ncx_archive(&empty_ncx, "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
            index: 1,
        });
        
        let mut manifest_info = create_valid_manifest_info();
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/ncx.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected empty NCX TOC to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::EmptyToc));
    }

    #[test]
    fn test_nav_with_special_characters() {
        let nav_with_special = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type="toc" id="toc" role="doc-toc">
    <ol>
      <li>
        <a href="chapter1.xhtml">Chapter 1: "Quotes" & Ampersands</a>
        <ol>
          <li><a href="chapter1.xhtml#section1">Section with <tags></a></li>
          <li><a href="chapter1.xhtml#section2">Section with émojis 🎉</a></li>
        </ol>
      </li>
    </ol>
  </nav>
</body>
</html>"#.to_vec();
        
        let (temp_dir, epub_path) = create_nav_archive(&nav_with_special, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected nav with special characters to pass validation");
        
        let nav_info = result.unwrap();
        assert_eq!(nav_info.toc_entries.len(), 1);
        assert!(nav_info.toc_entries[0].title.contains('"'));
        assert!(nav_info.toc_entries[0].title.contains('&'));
    }

    #[test]
    fn test_nav_with_fragment_only_links() {
        let nav_with_fragments = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type="toc" id="toc" role="doc-toc">
    <ol>
      <li>
        <a href="#section1">Section 1 (fragment only)</a>
      </li>
      <li>
        <a href="chapter1.xhtml">Chapter 1 (valid)</a>
      </li>
    </ol>
  </nav>
</body>
</html>"#.to_vec();
        
        let (temp_dir, epub_path) = create_nav_archive(&nav_with_fragments, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected nav with fragment links to pass validation");
        
        let nav_info = result.unwrap();
        assert_eq!(nav_info.toc_entries.len(), 1);
        assert_eq!(nav_info.toc_entries[0].title, "Chapter 1 (valid)");
    }

    #[test]
    fn test_nav_with_empty_href() {
        let nav_with_empty_href = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type="toc" id="toc" role="doc-toc">
    <ol>
      <li>
        <a href="">Empty href</a>
      </li>
      <li>
        <a href="chapter1.xhtml">Valid link</a>
      </li>
    </ol>
  </nav>
</body>
</html>"#.to_vec();
        
        let (temp_dir, epub_path) = create_nav_archive(&nav_with_empty_href, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected nav with empty href to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.error_code == ErrorCode::InvalidTocEntry));
    }

    #[test]
    fn test_nav_prefer_epub3_over_epub2() {
        // Create archive with both EPUB3 nav and EPUB2 NCX
        let temp_dir = tempdir().unwrap();
        let epub_path = temp_dir.path().join("test.epub");
        
        {
            let file = std::fs::File::create(&epub_path).unwrap();
            let mut zip = ZipWriter::new(file);
            
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            
            // Add container.xml
            zip.start_file("META-INF/container.xml", options).unwrap();
            zip.write_all(b r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();
            
            // Add both EPUB3 nav and EPUB2 NCX
            zip.start_file("OEBPS/nav.xhtml", options).unwrap();
            zip.write_all(&create_valid_epub3_nav()).unwrap();
            
            zip.start_file("OEBPS/ncx.ncx", options).unwrap();
            zip.write_all(&create_valid_epub2_ncx()).unwrap();
        }
        
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let mut manifest_info = create_valid_manifest_info();
        
        // Add both nav and ncx to manifest
        manifest_info.items.push(crate::core::ManifestItem {
            id: "nav".to_string(),
            href: "OEBPS/nav.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: vec!["nav".to_string()],
        });
        manifest_info.id_to_item.insert("nav".to_string(), manifest_info.items.last().unwrap().clone());
        
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/ncx.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected validation to pass with both nav and ncx");
        
        let nav_info = result.unwrap();
        // Should prefer EPUB3 nav over EPUB2 NCX
        assert!(nav_info.nav_path.is_some());
        assert!(nav_info.ncx_path.is_none());
    }

    #[test]
    fn test_ncx_fallback_when_no_epub3_nav() {
        let (temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
            index: 1,
        });
        
        let mut manifest_info = create_valid_manifest_info();
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/ncx.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected NCX fallback to work");
        
        let nav_info = result.unwrap();
        assert!(nav_info.nav_path.is_none());
        assert!(nav_info.ncx_path.is_some());
    }

    #[test]
    fn test_ncx_by_media_type() {
        let (temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/toc.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        
        let mut manifest_info = create_valid_manifest_info();
        manifest_info.items.push(crate::core::ManifestItem {
            id: "ncx".to_string(),
            href: "OEBPS/toc.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: vec![],
        });
        manifest_info.id_to_item.insert("ncx".to_string(), manifest_info.items.last().unwrap().clone());
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected NCX lookup by media-type to work");
        
        let nav_info = result.unwrap();
        assert!(nav_info.ncx_path.is_some());
        assert!(nav_info.ncx_path.unwrap().contains("toc.ncx"));
    }

    #[test]
    fn test_nav_info_structure() {
        let (temp_dir, epub_path) = create_nav_archive(&create_valid_epub3_nav(), "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok());
        
        let nav_info = result.unwrap();
        
        // Verify NavInfo structure
        assert!(nav_info.nav_path.is_some());
        assert_eq!(nav_info.nav_path.unwrap(), "OEBPS/nav.xhtml");
        assert!(nav_info.ncx_path.is_none());
        assert!(!nav_info.toc_entries.is_empty());
        
        // Verify TOC entry structure
        let entry = &nav_info.toc_entries[0];
        assert!(!entry.title.is_empty());
        assert!(!entry.href.is_empty());
        assert!(!entry.children.is_empty());
    }
}
