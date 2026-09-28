use std::io::Write;
use tempfile::tempdir;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::core::{ErrorCode, OpfPackage, SpineItem, ManifestInfo};

/// Helper function to create a test EPUB archive with navigation document
fn create_nav_archive(nav_content: &[u8], nav_filename: &str) -> (tempfile::TempDir, String) {
    let temp_dir = tempfile::tempdir().unwrap();
    let epub_path = temp_dir.path().join("test.epub");
    
    {
        let file = std::fs::File::create(&epub_path).unwrap();
        let mut zip = ZipWriter::new(file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        // Add container.xml
        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0"?>
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
fn create_ncx_archive(ncx_content: &[u8], ncx_filename: &str) -> (tempfile::TempDir, String) {
    let temp_dir = tempfile::tempdir().unwrap();
    let epub_path = temp_dir.path().join("test.epub");
    
    {
        let file = std::fs::File::create(&epub_path).unwrap();
        let mut zip = ZipWriter::new(file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        // Add container.xml
        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0"?>
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
        path: "test.opf".to_string(),
        root_file_path: "OEBPS/content.opf".to_string(),
        metadata: serde_json::Value::Null,
        manifest_items: vec![],
        spine_items: vec![SpineItem {
            idref: "chapter1.xhtml".to_string(),
            linear: "yes".to_string(),
            properties: vec![],
        }],
    }
}

/// Helper function to create a valid ManifestInfo for testing
fn create_valid_manifest_info() -> ManifestInfo {
    let mut items = Vec::new();
    let mut id_to_item = std::collections::HashMap::new();
    
    // Add a nav item to the manifest
    let nav_item = crate::core::ManifestItem {
        id: "nav".to_string(),
        href: "OEBPS/nav.xhtml".to_string(),
        media_type: "application/xhtml+xml".to_string(),
        properties: vec!["nav".to_string()],
    };
    items.push(nav_item.clone());
    id_to_item.insert("nav".to_string(), nav_item);
    
    ManifestInfo {
        items,
        id_to_item,
    }
}

mod tests {
    use super::*;
    use crate::epub::navigation::validate_navigation;

    #[test]
    fn test_valid_epub3_nav() {
        let (_temp_dir, epub_path) = create_nav_archive(&create_valid_epub3_nav(), "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected valid EPUB3 nav to pass validation");
        let nav_info = result.unwrap();
        println!("Nav Info: {:?}", nav_info);
        assert!(nav_info.nav_path.is_some());
        assert!(nav_info.ncx_path.is_none());
        assert_eq!(nav_info.toc_entries.len(), 2);
        assert_eq!(nav_info.toc_entries[0].title, "Chapter 1");
        assert_eq!(nav_info.toc_entries[0].children.len(), 2);
        assert_eq!(nav_info.toc_entries[1].title, "Chapter 2");
    }

    #[test]
    fn test_valid_epub2_ncx() {
        let (_temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
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
        let (_temp_dir, epub_path) = create_nav_archive(b"invalid", "OEBPS/missing.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected missing nav document to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::MissingNavDocument));
    }

    #[test]
    fn test_invalid_epub3_nav_xml() {
        let (_temp_dir, epub_path) = create_nav_archive(b"this is not valid xml {{{", "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected invalid EPUB3 nav XML to fail validation");
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::InvalidNavXml));
    }

    #[test]
    fn test_invalid_epub2_ncx_xml() {
        let (_temp_dir, epub_path) = create_ncx_archive(b"this is not valid xml {{{", "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            properties: vec!["nav".to_string()],
            linear: "yes".to_string(),
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
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::InvalidNcxXml));
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
        
        let (_temp_dir, epub_path) = create_nav_archive(&empty_nav, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected empty TOC to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::EmptyToc));
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
        
        let (_temp_dir, epub_path) = create_ncx_archive(&empty_ncx, "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            linear: "no".to_string(),
            properties: vec!["nav".to_string()],
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
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::EmptyToc));
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
        <a href="chapter1.xhtml">Chapter 1: "Quotes" &amp; Ampersands</a>
        <ol>
          <li><a href="chapter1.xhtml#section1">Section with &lt;tags&gt;</a></li>
          <li><a href="chapter1.xhtml#section2">Section with emojis</a></li>
        </ol>
      </li>
    </ol>
  </nav>
</body>
</html>"#.to_vec();
        
        let (_temp_dir, epub_path) = create_nav_archive(&nav_with_special, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_ok(), "Expected nav with special characters to pass validation: {:?}", result.err());
        
        let nav_info = result.unwrap();
        assert_eq!(nav_info.toc_entries.len(), 1);
        assert!(nav_info.toc_entries[0].title.contains('"'));
        assert!(nav_info.toc_entries[0].title.contains('&'));
    }

    #[test]
    fn test_nav_with_fragment_only_links() {
        let nav_with_fragments = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE html>
<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"en\">
<head>
  <title>Test Book</title>
</head>
<body>
  <nav epub:type=\"toc\" id=\"toc\" role=\"doc-toc\">
    <ol>
      <li>
        <a href=\"#section1\">Section 1 (fragment only)</a>
      </li>
      <li>
        <a href=\"chapter1.xhtml\">Chapter 1 (valid)</a>
      </li>
    </ol>
  </nav>
</body>
</html>".to_vec();
        
        let (_temp_dir, epub_path) = create_nav_archive(&nav_with_fragments, "OEBPS/nav.xhtml");
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
        
        let (_temp_dir, epub_path) = create_nav_archive(&nav_with_empty_href, "OEBPS/nav.xhtml");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let opf = create_valid_opf();
        let manifest_info = create_valid_manifest_info();
        
        let result = validate_navigation(&mut archive, &opf, &manifest_info);
        
        assert!(result.is_err(), "Expected nav with empty href to fail validation");
        
        let error = result.unwrap_err();
        assert!(error.errors.iter().any(|e| e.code == ErrorCode::InvalidTocEntry));
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
            zip.write_all(br#"<?xml version="1.0"?>
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
        let (_temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/ncx.ncx");
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&epub_path).unwrap()).unwrap();
        
        let mut opf = create_valid_opf();
        opf.spine_items.push(SpineItem {
            idref: "toc.xhtml".to_string(),
            linear: "yes".to_string(),
            properties: vec!["nav".to_string()],
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
        let (_temp_dir, epub_path) = create_ncx_archive(&create_valid_epub2_ncx(), "OEBPS/toc.ncx");
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
        let (_temp_dir, epub_path) = create_nav_archive(&create_valid_epub3_nav(), "OEBPS/nav.xhtml");
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

    use crate::epub::navigation::parse_ncx_document;
    use crate::epub::navigation::parse_nav_document;
    use crate::core::ValidationResult;

    fn codes(r: &ValidationResult) -> Vec<ErrorCode> {
        r.errors.iter().map(|e| e.code).collect()
    }

    fn nav_doc(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Nav</title></head>
<body>{body}</body></html>"#
        )
    }

    #[test]
    fn nav_titles_from_nested_markup_and_deep_nesting() {
        let xml = nav_doc(
            r#"<nav epub:type="toc"><h1>Contents</h1><ol>
  <li><a href="c1.xhtml"><span>One</span></a>
    <ol><li><a href="c1a.xhtml">Chapter <em>1</em>a</a>
      <ol><li><a href="c1a1.xhtml">Deep</a></li></ol></li></ol></li>
  <li><a href="c2.xhtml">Two</a></li>
</ol></nav>"#,
        );
        let info = parse_nav_document("nav.xhtml", &xml).expect("should parse");
        let e = &info.toc_entries;
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].title, "One");
        assert_eq!(e[0].children[0].title, "Chapter 1a");
        assert_eq!(e[0].children[0].children[0].href, "c1a1.xhtml");
        assert_eq!(e[1].title, "Two");
    }

    #[test]
    fn nav_ignores_non_toc_navs_and_keeps_children_of_headings_and_fragments() {
        let xml = nav_doc(
            r##"<nav epub:type="landmarks"><ol><li><a href="cover.xhtml">Cover</a></li></ol></nav>
<nav epub:type="toc"><ol>
  <li><span>Part I</span><ol><li><a href="a.xhtml">A</a></li></ol></li>
  <li><a href="#frag">Skipped</a><ol><li><a href="b.xhtml">B</a></li></ol></li>
</ol></nav>"##,
        );
        let info = parse_nav_document("nav.xhtml", &xml).expect("should parse");
        let hrefs: Vec<_> = info.toc_entries.iter().map(|e| e.href.as_str()).collect();
        assert_eq!(hrefs, vec!["a.xhtml", "b.xhtml"]);
    }

    #[test]
    fn nav_without_toc_is_missing_toc_nav() {
        let xml = nav_doc(r#"<nav epub:type="landmarks"><ol><li><a href="x.xhtml">X</a></li></ol></nav>"#);
        assert_eq!(codes(&parse_nav_document("n", &xml).unwrap_err()), vec![ErrorCode::MissingTocNav]);
    }

    #[test]
    fn nav_empty_href_and_empty_toc() {
        let xml = nav_doc(r#"<nav epub:type="toc"><ol><li><a href="">Bad</a></li></ol></nav>"#);
        let c = codes(&parse_nav_document("n", &xml).unwrap_err());
        assert!(c.contains(&ErrorCode::InvalidTocEntry) && c.contains(&ErrorCode::EmptyToc), "{c:?}");
    }

    #[test]
    fn nav_malformed_is_invalid_nav_xml() {
        let xml = r#"<html><body><nav epub:type="toc"><ol><li><a href="a">A</a></li></ol>"#;
        assert_eq!(codes(&parse_nav_document("n", xml).unwrap_err()), vec![ErrorCode::InvalidNavXml]);
    }

    const NCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="x"/></head>
  <docTitle><text>Book</text></docTitle>
  <navMap>
    <navPoint id="n1" playOrder="1"><navLabel><text>One</text></navLabel><content src="c1.xhtml"/>
      <navPoint id="n2" playOrder="2"><navLabel><text>One A</text></navLabel><content src="c1.xhtml#a"/></navPoint>
    </navPoint>
    <navPoint id="n3" playOrder="3"><navLabel><text>Two</text></navLabel><content src="c2.xhtml"/></navPoint>
  </navMap>
</ncx>"#;

    #[test]
    fn ncx_parses_with_doctype_and_nesting() {
        let info = parse_ncx_document("toc.ncx", NCX).expect("should parse");
        assert_eq!(info.toc_entries.len(), 2);
        assert_eq!(info.toc_entries[0].children[0].title, "One A");
        assert_eq!(info.ncx_path.as_deref(), Some("toc.ncx"));
    }

    #[test]
    fn ncx_without_namespace_attribute_is_fine() {
        let xml = NCX.replace(r#" xmlns="http://www.daisy.org/z3986/2005/ncx/""#, "");
        assert!(parse_ncx_document("toc.ncx", &xml).is_ok());
    }

    #[test]
    fn ncx_empty_navmap_is_an_error() {
        let xml = r#"<ncx><navMap></navMap></ncx>"#;
        assert_eq!(codes(&parse_ncx_document("toc.ncx", xml).unwrap_err()), vec![ErrorCode::EmptyToc]);
    }

    #[test]
    fn ncx_wrong_root_or_malformed() {
        assert_eq!(codes(&parse_ncx_document("t", "<html/>").unwrap_err()), vec![ErrorCode::InvalidNcxXml]);
        assert_eq!(codes(&parse_ncx_document("t", "<ncx><navMap>").unwrap_err()), vec![ErrorCode::InvalidNcxXml]);
    }
}
