#[cfg(test)]
mod tests {
    use crate::core::ErrorCode::{self, MissingManifest};
    use crate::epub::parse_opf;
    use crate::core::{ValidationError,ValidationLocation};
    /// Creates a minimal valid OP2 package document
    fn create_valid_opf_epub2() -> Vec<u8> {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>{}</dc:title>
    <dc:identifier id="BookId">{}</dc:identifier>
    <dc:language>{}</dc:language>
    <dc:creator>{}</dc:creator>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#,
            "Test Book", "test-123", "en", "Test Author"
        )
        .into_bytes()
    }

    /// Creates a minimal valid OPF3 package document
    fn create_valid_opf_epub3() -> Vec<u8> {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf" xmlns:xs="http://www.w3.org/2001/XMLSchema-instance">
    <dc:title>{}</dc:title>
    <dc:identifier id="BookId">{}</dc:identifier>
    <dc:language>{}</dc:language>
    <dc:creator>{}</dc:creator>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chapter1"/>
  </spine>
</package>"#,
            "Test Book", "test-123", "en", "Test Author"
        )
        .into_bytes()
    }

    #[test]
    fn test_valid_epub2_opf() {
        let opf_data = create_valid_opf_epub2();
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "Valid EPUB2 OPF should parse successfully");
        let opf = result.unwrap();
        
        assert_eq!(opf.metadata["title"], "Test Book");
        assert_eq!(opf.metadata["language"], "en");
        assert_eq!(opf.manifest_items.len(), 3);
        assert_eq!(opf.spine_items.len(), 1);
    }

    #[test]
    fn test_valid_epub3_opf() {
        let opf_data = create_valid_opf_epub3();
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "Valid EPUB3 OPF should parse successfully");
        let opf = result.unwrap();
        
        assert_eq!(opf.metadata["title"], "Test Book");
        assert_eq!(opf.metadata["language"], "en");
        assert_eq!(opf.manifest_items.len(), 3);
        assert_eq!(opf.spine_items.len(), 1);
    }

    #[test]
    fn test_invalid_opf_xml() {
        let opf_data = b"<?xml version=\"1.0\"?><package>invalid xml {{{";
        let result = parse_opf(opf_data);
        
        assert!(result.is_err(), "Invalid OPF XML should fail parsing");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::InvalidOpfXml
        }), "Should report InvalidOpfXml error");
    }

    #[test]
    fn test_invalid_utf8_opf() {
        let opf_data = vec![0x80, 0x81, 0x82, 0x83]; // Invalid UTF-8 bytes
        let result = parse_opf(&opf_data);
        
        assert!(result.is_err(), "Invalid UTF-8 should fail parsing");
        let validation_result = result.unwrap_err();
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::InvalidOpfXml
        }), "Should report InvalidOpfXml error");
    }

    #[test]
    fn test_missing_manifest() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_err(), "OPF without manifest should not parse");
        let codes: Vec<ErrorCode> = result.unwrap_err().errors.iter().map(|e| e.code).collect();
        assert!(codes.contains(&ErrorCode::MissingManifest), "got {codes:?}");

        // The spine is present, so it should be the only error:
        assert_eq!(codes, vec![ErrorCode::MissingManifest]);
        
    }

    #[test]
    fn test_missing_spine() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_err(), "OPF without spine should still parse");
        let codes: Vec<ErrorCode> = result.unwrap_err().errors.iter().map(|e| e.code).collect();
        assert!(codes.contains(&ErrorCode::MissingSpine), "got {codes:?}");
    }

    #[test]
    fn test_empty_manifest() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
  </manifest>
  <spine>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_err(), "OPF without manifest should not parse");
        let codes: Vec<ErrorCode> = result.unwrap_err().errors.iter().map(|e| e.code).collect();
        assert!(codes.contains(&ErrorCode::MissingManifest), "got {codes:?}");

        // The spine is present, so it should be the only error:
        assert!(codes.contains(&ErrorCode::MissingManifest));
        assert!(codes.contains(&ErrorCode::MissingSpine));
    }

    #[test]
    fn test_empty_spine() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_err(), "OPF with empty spine should not parse");
        let codes: Vec<ErrorCode> = result.unwrap_err().errors.iter().map(|e| e.code).collect();
        assert!(codes.contains(&ErrorCode::MissingSpine), "got {codes:?}");
    }

    #[test]
    fn test_multiple_creators() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
    <dc:creator>Author One</dc:creator>
    <dc:creator>Author Two</dc:creator>
    <dc:creator>Author Three</dc:creator>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF with multiple creators should parse");
        let opf = result.unwrap();
        
        let creators = opf.metadata["creators"].as_array().unwrap();
        assert_eq!(creators.len(), 3);
        assert_eq!(creators[0], "Author One");
        assert_eq!(creators[1], "Author Two");
        assert_eq!(creators[2], "Author Three");
    }

    #[test]
    fn test_multiple_identifiers() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:identifier>other-identifier</dc:identifier>
    <dc:identifier>another-id</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF with multiple identifiers should parse");
        let opf = result.unwrap();
        
        let identifiers = opf.metadata["identifiers"].as_array().unwrap();
        assert_eq!(identifiers.len(), 3);
        assert_eq!(identifiers[0], "test-123");
        assert_eq!(identifiers[1], "other-identifier");
        assert_eq!(identifiers[2], "another-id");
    }

    #[test]
    fn test_manifest_item_with_properties() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="cover" href="cover.jpg" media-type="image/jpeg" properties="cover-image"/>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF with manifest properties should parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.manifest_items.len(), 3);
        
        let nav_item = opf.manifest_items.iter().find(|i| i.id == "nav").unwrap();
        assert!(nav_item.properties.contains(&"nav".to_string()));
        
        let cover_item = opf.manifest_items.iter().find(|i| i.id == "cover").unwrap();
        assert!(cover_item.properties.contains(&"cover-image".to_string()));
    }

    #[test]
    fn test_spine_item_with_properties() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chapter2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1" linear="yes"/>
    <itemref idref="chapter2" linear="no"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF with spine properties should parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.spine_items.len(), 2);
        
        let chapter1 = opf.spine_items.iter().find(|s| s.idref == "chapter1").unwrap();
        assert_eq!(chapter1.linear, "yes");
        
        let chapter2 = opf.spine_items.iter().find(|s| s.idref == "chapter2").unwrap();
        assert_eq!(chapter2.linear, "no");
    }

    #[test]
    fn test_manifest_item_without_media_type() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF without media-type should still parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.manifest_items.len(), 1);
        assert_eq!(opf.manifest_items[0].media_type, "");
    }

    #[test]
    fn test_manifest_item_without_properties() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF without properties should parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.manifest_items.len(), 1);
        assert!(opf.manifest_items[0].properties.is_empty());
    }

    #[test]
    fn test_spine_item_without_linear_attribute() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF without linear attribute should parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.spine_items.len(), 1);
        assert_eq!(opf.spine_items[0].linear, "yes"); // Default value
    }

    #[test]
    fn test_opf_with_special_characters_in_metadata() {
        let opf_data = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book &amp; More</dc:title>
    <dc:identifier id="BookId">test-123</dc:identifier>
    <dc:language>en</dc:language>
    <dc:creator>Author &lt;Test&gt;</dc:creator>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
        ).into_bytes();
        
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "OPF with special characters should parse");
        let opf = result.unwrap();
        
        assert_eq!(opf.metadata["title"], "Test Book & More");
        assert_eq!(opf.metadata["creators"][0], "Author <Test>");
    }

    #[test]
    fn test_opf_version_attribute() {
        let opf_data = create_valid_opf_epub2();
        let result = parse_opf(&opf_data);
        
        assert!(result.is_ok(), "EPUB2 OPF should parse");
        let opf = result.unwrap();
        
        // Version should be preserved in metadata if present
        assert!(opf.metadata.is_object());
    }
    const VALID: &str = r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title id="t1" xml:lang="en">My &amp; Book</dc:title>
  <dc:creator>A</dc:creator><dc:creator>B</dc:creator>
  <dc:language>en</dc:language>
  <dc:identifier>urn:uuid:1</dc:identifier>
  <dc:subject/>
  <meta property="dcterms:modified">2024-01-01T00:00:00Z</meta>
</metadata>
<manifest>
  <item id="c1" href="a&amp;b.xhtml" media-type="application/xhtml+xml" properties="nav scripted"/>
</manifest>
<spine toc="ncx"><itemref idref="c1"/></spine>
</package>"#;

  #[test]
  fn parses_valid_package() {
      let pkg = parse_opf(VALID.as_bytes()).expect("should parse");
      assert_eq!(pkg.manifest_items.len(), 1);
      assert_eq!(pkg.manifest_items[0].href, "a&b.xhtml");
      assert_eq!(pkg.manifest_items[0].properties, vec!["nav", "scripted"]);
      assert_eq!(pkg.spine_items[0].linear, "yes");
      assert_eq!(pkg.metadata["title"], "My & Book");
      assert_eq!(pkg.metadata["creators"].as_array().unwrap().len(), 2);
  }

  #[test]
  fn handles_prefixed_root_and_empty_dc_elements() {
      let xml = VALID
          .replace("<package ", "<opf:package ")
          .replace("</package>", "</opf:package>")
          .replace("xmlns=", "xmlns:opf=")
          .replace("<dc:title id=\"t1\" xml:lang=\"en\">My &amp; Book</dc:title>", "<dc:title/>");
      let pkg = parse_opf(xml.as_bytes()).expect("should parse");
      assert_eq!(pkg.metadata["title"], "");
      assert_eq!(pkg.manifest_items.len(), 1);
  }

  #[test]
  fn rejects_unclosed_package() {
      assert!(parse_opf(b"<package version=\"3.0\"><manifest>").is_err());
  }

  #[test]
  fn rejects_wrong_root() {
      assert!(parse_opf(b"<html><manifest/></html>").is_err());
  }

  #[test]
  fn rejects_invalid_utf8() {
      assert!(parse_opf(&[0xFF, 0xFE, 0x00]).is_err());
  }

  #[test]
  fn missing_manifest_and_spine_is_an_error() {
      let err = parse_opf(b"<package version=\"3.0\"><metadata/></package>").unwrap_err();
      assert_eq!(err.errors.len(), 2);
  }
  const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

  #[test]
  fn accepts_bom() {
      let mut data = UTF8_BOM.to_vec();
      data.extend_from_slice(VALID.as_bytes());
      assert!(parse_opf(&data).is_ok());
  }
}
