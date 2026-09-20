#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::core::types::{ErrorCode, Severity, ValidationLocation, ValidationCategory};

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_error_variant() {
        assert_eq!(Severity::Error, Severity::Error);
    }

    #[test]
    fn test_severity_warning_variant() {
        assert_eq!(Severity::Warning, Severity::Warning);
    }

    #[test]
    fn test_severity_debug_format() {
        let error = format!("{:?}", Severity::Error);
        let warning = format!("{:?}", Severity::Warning);
        assert_eq!(error, "Error");
        assert_eq!(warning, "Warning");
    }

    #[test]
    fn test_severity_clone() {
        let severity = Severity::Error;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_ne!(Severity::Error, Severity::Warning);
    }

    // ==================== ValidationCategory Tests ====================

    #[test]
    fn test_validation_category_archive() {
        assert_eq!(ValidationCategory::Archive, ValidationCategory::Archive);
    }

    #[test]
    fn test_validation_category_container() {
        assert_eq!(ValidationCategory::Container, ValidationCategory::Container);
    }

    #[test]
    fn test_validation_category_opf() {
        assert_eq!(ValidationCategory::Opf, ValidationCategory::Opf);
    }

    #[test]
    fn test_validation_category_spine() {
        assert_eq!(ValidationCategory::Spine, ValidationCategory::Spine);
    }

    #[test]
    fn test_validation_category_manifest() {
        assert_eq!(ValidationCategory::Manifest, ValidationCategory::Manifest);
    }

    #[test]
    fn test_validation_category_navigation() {
        assert_eq!(ValidationCategory::Navigation, ValidationCategory::Navigation);
    }

    #[test]
    fn test_validation_category_content() {
        assert_eq!(ValidationCategory::Content, ValidationCategory::Content);
    }

    #[test]
    fn test_validation_category_cover() {
        assert_eq!(ValidationCategory::Cover, ValidationCategory::Cover);
    }

    #[test]
    fn test_validation_category_consistency() {
        assert_eq!(ValidationCategory::Consistency, ValidationCategory::Consistency);
    }

    // ==================== ErrorCode Tests ====================

    #[test]
    fn test_error_code_invalid_zip() {
        assert_eq!(ErrorCode::InvalidZip, ErrorCode::InvalidZip);
    }

    #[test]
    fn test_error_code_unreadable_entry() {
        assert_eq!(ErrorCode::UnreadableEntry, ErrorCode::UnreadableEntry);
    }

    #[test]
    fn test_error_code_duplicate_path() {
        assert_eq!(ErrorCode::DuplicatePath, ErrorCode::DuplicatePath);
    }

    #[test]
    fn test_error_code_missing_container_xml() {
        assert_eq!(ErrorCode::MissingContainerXml, ErrorCode::MissingContainerXml);
    }

    #[test]
    fn test_error_code_invalid_container_xml() {
        assert_eq!(ErrorCode::InvalidContainerXml, ErrorCode::InvalidContainerXml);
    }

    #[test]
    fn test_error_code_missing_opf_reference() {
        assert_eq!(ErrorCode::MissingOpfReference, ErrorCode::MissingOpfReference);
    }

    #[test]
    fn test_error_code_invalid_opf_reference() {
        assert_eq!(ErrorCode::InvalidOpfReference, ErrorCode::InvalidOpfReference);
    }

    #[test]
    fn test_error_code_missing_opf() {
        assert_eq!(ErrorCode::MissingOpf, ErrorCode::MissingOpf);
    }

    #[test]
    fn test_error_code_invalid_opf_xml() {
        assert_eq!(ErrorCode::InvalidOpfXml, ErrorCode::InvalidOpfXml);
    }

    #[test]
    fn test_error_code_missing_spine() {
        assert_eq!(ErrorCode::MissingSpine, ErrorCode::MissingSpine);
    }

    #[test]
    fn test_error_code_missing_manifest() {
        assert_eq!(ErrorCode::MissingManifest, ErrorCode::MissingManifest);
    }

    #[test]
    fn test_error_code_empty_spine() {
        assert_eq!(ErrorCode::EmptySpine, ErrorCode::EmptySpine);
    }

    #[test]
    fn test_error_code_missing_item_ref() {
        assert_eq!(ErrorCode::MissingItemRef, ErrorCode::MissingItemRef);
    }

    #[test]
    fn test_error_code_duplicate_manifest_id() {
        assert_eq!(ErrorCode::DuplicateManifestId, ErrorCode::DuplicateManifestId);
    }

    #[test]
    fn test_error_code_invalid_manifest_id() {
        assert_eq!(ErrorCode::InvalidManifestId, ErrorCode::InvalidManifestId);
    }

    #[test]
    fn test_error_code_missing_media_type() {
        assert_eq!(ErrorCode::MissingMediaType, ErrorCode::MissingMediaType);
    }

    #[test]
    fn test_error_code_missing_nav_document() {
        assert_eq!(ErrorCode::MissingNavDocument, ErrorCode::MissingNavDocument);
    }

    #[test]
    fn test_error_code_invalid_nav_xml() {
        assert_eq!(ErrorCode::InvalidNavXml, ErrorCode::InvalidNavXml);
    }

    #[test]
    fn test_error_code_invalid_ncx_xml() {
        assert_eq!(ErrorCode::InvalidNcxXml, ErrorCode::InvalidNcxXml);
    }

    #[test]
    fn test_error_code_missing_toc_nav() {
        assert_eq!(ErrorCode::MissingTocNav, ErrorCode::MissingTocNav);
    }

    #[test]
    fn test_error_code_empty_toc() {
        assert_eq!(ErrorCode::EmptyToc, ErrorCode::EmptyToc);
    }

    #[test]
    fn test_error_code_invalid_toc_entry() {
        assert_eq!(ErrorCode::InvalidTocEntry, ErrorCode::InvalidTocEntry);
    }

    #[test]
    fn test_error_code_missing_chapter_file() {
        assert_eq!(ErrorCode::MissingChapterFile, ErrorCode::MissingChapterFile);
    }

    #[test]
    fn test_error_code_unparsable_html() {
        assert_eq!(ErrorCode::UnparsableHtml, ErrorCode::UnparsableHtml);
    }

    #[test]
    fn test_error_code_missing_cover() {
        assert_eq!(ErrorCode::MissingCover, ErrorCode::MissingCover);
    }

    #[test]
    fn test_error_code_unreadable_cover() {
        assert_eq!(ErrorCode::UnreadableCover, ErrorCode::UnreadableCover);
    }

    #[test]
    fn test_error_code_invalid_cover_format() {
        assert_eq!(ErrorCode::InvalidCoverFormat, ErrorCode::InvalidCoverFormat);
    }

    #[test]
    fn test_error_code_spine_manifest_mismatch() {
        assert_eq!(ErrorCode::SpineManifestMismatch, ErrorCode::SpineManifestMismatch);
    }

    #[test]
    fn test_error_code_manifest_archive_mismatch() {
        assert_eq!(ErrorCode::ManifestArchiveMismatch, ErrorCode::ManifestArchiveMismatch);
    }

    #[test]
    fn test_error_code_nav_spine_mismatch() {
        assert_eq!(ErrorCode::NavSpineMismatch, ErrorCode::NavSpineMismatch);
    }

    // ==================== ErrorCode Category Tests ====================

    #[test]
    fn test_error_code_category_archive() {
        assert_eq!(ErrorCode::InvalidZip.category(), ValidationCategory::Archive);
        assert_eq!(ErrorCode::UnreadableEntry.category(), ValidationCategory::Archive);
        assert_eq!(ErrorCode::DuplicatePath.category(), ValidationCategory::Archive);
    }

    #[test]
    fn test_error_code_category_container() {
        assert_eq!(ErrorCode::MissingContainerXml.category(), ValidationCategory::Container);
        assert_eq!(ErrorCode::InvalidContainerXml.category(), ValidationCategory::Container);
        assert_eq!(ErrorCode::MissingOpfReference.category(), ValidationCategory::Container);
        assert_eq!(ErrorCode::InvalidOpfReference.category(), ValidationCategory::Container);
    }

    #[test]
    fn test_error_code_category_opf() {
        assert_eq!(ErrorCode::MissingOpf.category(), ValidationCategory::Opf);
        assert_eq!(ErrorCode::InvalidOpfXml.category(), ValidationCategory::Opf);
        assert_eq!(ErrorCode::MissingSpine.category(), ValidationCategory::Opf);
        assert_eq!(ErrorCode::MissingManifest.category(), ValidationCategory::Opf);
    }

    #[test]
    fn test_error_code_category_spine() {
        assert_eq!(ErrorCode::EmptySpine.category(), ValidationCategory::Spine);
        assert_eq!(ErrorCode::MissingItemRef.category(), ValidationCategory::Spine);
    }

    #[test]
    fn test_error_code_category_manifest() {
        assert_eq!(ErrorCode::DuplicateManifestId.category(), ValidationCategory::Manifest);
        assert_eq!(ErrorCode::InvalidManifestId.category(), ValidationCategory::Manifest);
        assert_eq!(ErrorCode::MissingMediaType.category(), ValidationCategory::Manifest);
    }

    #[test]
    fn test_error_code_category_navigation() {
        assert_eq!(ErrorCode::MissingNavDocument.category(), ValidationCategory::Navigation);
        assert_eq!(ErrorCode::InvalidNavXml.category(), ValidationCategory::Navigation);
        assert_eq!(ErrorCode::InvalidNcxXml.category(), ValidationCategory::Navigation);
        assert_eq!(ErrorCode::MissingTocNav.category(), ValidationCategory::Navigation);
        assert_eq!(ErrorCode::EmptyToc.category(), ValidationCategory::Navigation);
        assert_eq!(ErrorCode::InvalidTocEntry.category(), ValidationCategory::Navigation);
    }

    #[test]
    fn test_error_code_category_content() {
        assert_eq!(ErrorCode::MissingChapterFile.category(), ValidationCategory::Content);
        assert_eq!(ErrorCode::UnparsableHtml.category(), ValidationCategory::Content);
    }

    #[test]
    fn test_error_code_category_cover() {
        assert_eq!(ErrorCode::MissingCover.category(), ValidationCategory::Cover);
        assert_eq!(ErrorCode::UnreadableCover.category(), ValidationCategory::Cover);
        assert_eq!(ErrorCode::InvalidCoverFormat.category(), ValidationCategory::Cover);
    }

    #[test]
    fn test_error_code_category_consistency() {
        assert_eq!(ErrorCode::SpineManifestMismatch.category(), ValidationCategory::Consistency);
        assert_eq!(ErrorCode::ManifestArchiveMismatch.category(), ValidationCategory::Consistency);
        assert_eq!(ErrorCode::NavSpineMismatch.category(), ValidationCategory::Consistency);
    }

    // ==================== ErrorCode Message Tests ====================

    #[test]
    fn test_error_code_messages() {
        assert_eq!(ErrorCode::InvalidZip.message(), "Not a valid ZIP archive");
        assert_eq!(ErrorCode::UnreadableEntry.message(), "Cannot read archive entry");
        assert_eq!(ErrorCode::DuplicatePath.message(), "Duplicate archive path");
        assert_eq!(ErrorCode::MissingContainerXml.message(), "META-INF/container.xml not found");
        assert_eq!(ErrorCode::InvalidContainerXml.message(), "container.xml is not valid XML");
        assert_eq!(ErrorCode::MissingOpfReference.message(), "No valid OPF reference in container.xml");
        assert_eq!(ErrorCode::InvalidOpfReference.message(), "OPF file not found");
        assert_eq!(ErrorCode::MissingOpf.message(), "OPF file cannot be read");
        assert_eq!(ErrorCode::InvalidOpfXml.message(), "OPF is not valid XML");
        assert_eq!(ErrorCode::MissingSpine.message(), "OPF missing <spine> element");
        assert_eq!(ErrorCode::MissingManifest.message(), "OPF missing <manifest> element");
        assert_eq!(ErrorCode::EmptySpine.message(), "Spine has no linear items");
        assert_eq!(ErrorCode::MissingItemRef.message(), "Item reference missing from manifest");
        assert_eq!(ErrorCode::DuplicateManifestId.message(), "Duplicate manifest ID");
        assert_eq!(ErrorCode::InvalidManifestId.message(), "Empty manifest ID");
        assert_eq!(ErrorCode::MissingMediaType.message(), "Missing media-type attribute");
        assert_eq!(ErrorCode::MissingNavDocument.message(), "No EPUB3 nav or EPUB2 NCX found");
        assert_eq!(ErrorCode::InvalidNavXml.message(), "Nav document is not valid HTML");
        assert_eq!(ErrorCode::InvalidNcxXml.message(), "NCX document is not valid XML");
        assert_eq!(ErrorCode::MissingTocNav.message(), "No <nav epub:type=\"toc\"> found");
        assert_eq!(ErrorCode::EmptyToc.message(), "TOC has no usable entries");
        assert_eq!(ErrorCode::InvalidTocEntry.message(), "TOC entry has empty href");
        assert_eq!(ErrorCode::MissingChapterFile.message(), "Chapter file not found in archive");
        assert_eq!(ErrorCode::UnparsableHtml.message(), "Chapter HTML cannot be parsed");
        assert_eq!(ErrorCode::MissingCover.message(), "No cover image found");
        assert_eq!(ErrorCode::UnreadableCover.message(), "Cover file cannot be read");
        assert_eq!(ErrorCode::InvalidCoverFormat.message(), "Cover is not a supported format");
        assert_eq!(ErrorCode::SpineManifestMismatch.message(), "Spine itemref not found in manifest");
        assert_eq!(ErrorCode::ManifestArchiveMismatch.message(), "Manifest item not found in archive");
        assert_eq!(ErrorCode::NavSpineMismatch.message(), "Navigation target not in spine");
    }

    #[test]
    fn test_error_code_display() {
        let error = ErrorCode::InvalidZip;
        assert_eq!(format!("{}", error), "Not a valid ZIP archive");
    }

    // ==================== ValidationLocation Tests ====================

    #[test]
    fn test_validation_location_root() {
        let location = ValidationLocation::Root;
        assert_eq!(format!("{}", location), "root");
    }

    #[test]
    fn test_validation_location_path() {
        let location = ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() };
        assert_eq!(format!("{}", location), "META-INF/container.xml");
    }

    #[test]
    fn test_validation_location_opf_element() {
        let location = ValidationLocation::OpfElement { 
            file: "OEBPS/content.opf".to_string(), 
            element: "spine/itemref".to_string() 
        };
        assert_eq!(format!("{}", location), "OPF element OEBPS/content.opf/spine/itemref");
    }

    #[test]
    fn test_validation_location_nav_element() {
        let location = ValidationLocation::NavElement { 
            file: "OEBPS/nav.xhtml".to_string(), 
            element: "nav/ol/li/a".to_string() 
        };
        assert_eq!(format!("{}", location), "nav element OEBPS/nav.xhtml/nav/ol/li/a");
    }

    #[test]
    fn test_validation_location_ncx_element() {
        let location = ValidationLocation::NcxElement { 
            file: "OEBPS/toc.ncx".to_string(), 
            element: "navMap/navPoint/content".to_string() 
        };
        assert_eq!(format!("{}", location), "NCX element OEBPS/toc.ncx/navMap/navPoint/content");
    }

    #[test]
    fn test_validation_location_manifest_item() {
        let location = ValidationLocation::ManifestItem { id: "chapter1".to_string() };
        assert_eq!(format!("{}", location), "manifest item id='chapter1'");
    }

    #[test]
    fn test_validation_location_spine_item() {
        let location = ValidationLocation::SpineItem { index: 3 };
        assert_eq!(format!("{}", location), "spine item #3");
    }

    #[test]
    fn test_validation_location_toc_entry() {
        let location = ValidationLocation::TocEntry { index: 5 };
        assert_eq!(format!("{}", location), "TOC entry #5");
    }

    #[test]
    fn test_validation_location_clone() {
        let location = ValidationLocation::Root;
        let cloned = location.clone();
        assert_eq!(location, cloned);
    }

    #[test]
    fn test_validation_location_equality() {
        assert_eq!(ValidationLocation::Root, ValidationLocation::Root);
        
        let loc1 = ValidationLocation::ArchiveEntry { path: "test.xml".to_string() };
        let loc2 = ValidationLocation::ArchiveEntry { path: "test.xml".to_string() };
        assert_eq!(loc1, loc2);
        
        let loc3 = ValidationLocation::ArchiveEntry { path: "other.xml".to_string() };
        assert_ne!(loc1, loc3);
    }

    // ==================== ValidationError Tests ====================

    #[test]
    fn test_validation_error_new() {
        let error = ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root);
        
        assert_eq!(error.code, ErrorCode::InvalidZip);
        assert_eq!(error.message, "Not a valid ZIP archive");
        assert_eq!(error.severity, Severity::Error);
        assert_eq!(format!("{}", error.location), "root");
    }

    #[test]
    fn test_validation_error_warning() {
        let error = ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::Cover { path: "OEBPS/cover.jpg".to_string() });
        
        assert_eq!(error.code, ErrorCode::MissingCover);
        assert_eq!(error.message, "No cover image found");
        assert_eq!(error.severity, Severity::Warning);
    }

    #[test]
    fn test_validation_error_clone() {
        let error = ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root);
        let cloned = error.clone();
        
        assert_eq!(error.code, cloned.code);
        assert_eq!(error.message, cloned.message);
        assert_eq!(error.severity, cloned.severity);
        assert_eq!(error.location, cloned.location);
    }

    #[test]
    fn test_validation_error_debug() {
        let error = ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root);
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("InvalidZip"));
        assert!(debug_output.contains("Error"));
    }

    // ==================== ArchiveInfo Tests ====================

    #[test]
    fn test_archive_info_empty() {
        let info = ArchiveInfo {
            entries: Vec::new(),
            duplicate_paths: Vec::new(),
        };
        
        assert!(info.entries.is_empty());
        assert!(info.duplicate_paths.is_empty());
    }

    #[test]
    fn test_archive_info_with_entries() {
        let info = ArchiveInfo {
            entries: vec![
                "META-INF/container.xml".to_string(),
                "OEBPS/content.opf".to_string(),
                "OEBPS/chapter1.xhtml".to_string(),
            ],
            duplicate_paths: vec![],
        };
        
        assert_eq!(info.entries.len(), 3);
        assert_eq!(info.entries[0], "META-INF/container.xml");
    }

    #[test]
    fn test_archive_info_with_duplicates() {
        let info = ArchiveInfo {
            entries: vec![
                "META-INF/container.xml".to_string(),
                "OEBPS/content.opf".to_string(),
            ],
            duplicate_paths: vec!["OEBPS/content.opf".to_string()],
        };
        
        assert_eq!(info.duplicate_paths.len(), 1);
        assert_eq!(info.duplicate_paths[0], "OEBPS/content.opf");
    }

    #[test]
    fn test_archive_info_clone() {
        let info = ArchiveInfo {
            entries: vec!["test.xml".to_string()],
            duplicate_paths: vec![],
        };
        let cloned = info.clone();
        
        assert_eq!(info.entries, cloned.entries);
        assert_eq!(info.duplicate_paths, cloned.duplicate_paths);
    }

    // ==================== OpfInfo Tests ====================

    #[test]
    fn test_opf_info() {
        let info = OpfInfo {
            opf_path: "OEBPS/content.opf".to_string(),
            opf_data: b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_vec(),
        };
        
        assert_eq!(info.opf_path, "OEBPS/content.opf");
        assert!(!info.opf_data.is_empty());
    }

    #[test]
    fn test_opf_info_clone() {
        let info = OpfInfo {
            opf_path: "OEBPS/content.opf".to_string(),
            opf_data: vec![1, 2, 3],
        };
        let cloned = info.clone();
        
        assert_eq!(info.opf_path, cloned.opf_path);
        assert_eq!(info.opf_data, cloned.opf_data);
    }

    // ==================== ManifestItem Tests ====================

    #[test]
    fn test_manifest_item() {
        let item = ManifestItem {
            id: "chapter1".to_string(),
            href: "chapter1.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: vec![],
        };
        
        assert_eq!(item.id, "chapter1");
        assert_eq!(item.href, "chapter1.xhtml");
        assert_eq!(item.media_type, "application/xhtml+xml");
        assert!(item.properties.is_empty());
    }

    #[test]
    fn test_manifest_item_with_properties() {
        let item = ManifestItem {
            id: "cover-image".to_string(),
            href: "cover.jpg".to_string(),
            media_type: "image/jpeg".to_string(),
            properties: vec!["cover-image".to_string()],
        };
        
        assert_eq!(item.properties.len(), 1);
        assert_eq!(item.properties[0], "cover-image");
    }

    #[test]
    fn test_manifest_item_clone() {
        let item = ManifestItem {
            id: "test".to_string(),
            href: "test.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: vec!["prop1".to_string()],
        };
        let cloned = item.clone();
        
        assert_eq!(item.id, cloned.id);
        assert_eq!(item.href, cloned.href);
        assert_eq!(item.media_type, cloned.media_type);
        assert_eq!(item.properties, cloned.properties);
    }

    // ==================== SpineItem Tests ====================

    #[test]
    fn test_spine_item_linear() {
        let item = SpineItem {
            idref: "chapter1".to_string(),
            linear: "yes".to_string(),
            properties: vec![],
        };
        
        assert_eq!(item.idref, "chapter1");
        assert_eq!(item.linear, "yes");
    }

    #[test]
    fn test_spine_item_non_linear() {
        let item = SpineItem {
            idref: "cover".to_string(),
            linear: "no".to_string(),
            properties: vec![],
        };
        
        assert_eq!(item.idref, "cover");
        assert_eq!(item.linear, "no");
    }

    #[test]
    fn test_spine_item_clone() {
        let item = SpineItem {
            idref: "test".to_string(),
            linear: "yes".to_string(),
            properties: vec!["prop1".to_string()],
        };
        let cloned = item.clone();
        
        assert_eq!(item.idref, cloned.idref);
        assert_eq!(item.linear, cloned.linear);
        assert_eq!(item.properties, cloned.properties);
    }

    // ==================== SpineInfo Tests ====================

    #[test]
    fn test_spine_info_empty() {
        let info = SpineInfo {
            items: Vec::new(),
            linear_items: Vec::new(),
        };
        
        assert!(info.items.is_empty());
        assert!(info.linear_items.is_empty());
    }

    #[test]
    fn test_spine_info_with_items() {
        let items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                linear: "yes".to_string(),
                properties: vec![],
            },
            SpineItem {
                idref: "cover".to_string(),
                linear: "no".to_string(),
                properties: vec![],
            },
        ];
        
        let info = SpineInfo {
            items: items.clone(),
            linear_items: vec![items[0].clone()],
        };
        
        assert_eq!(info.items.len(), 2);
        assert_eq!(info.linear_items.len(), 1);
    }

    #[test]
    fn test_spine_info_clone() {
        let info = SpineInfo {
            items: vec![
                SpineItem {
                    idref: "test".to_string(),
                    linear: "yes".to_string(),
                    properties: vec![],
                },
            ],
            linear_items: vec![],
        };
        let cloned = info.clone();
        
        assert_eq!(info.items.len(), cloned.items.len());
        assert_eq!(info.linear_items.len(), cloned.linear_items.len());
    }

    // ==================== ManifestInfo Tests ====================

    #[test]
    fn test_manifest_info_empty() {
        let info = ManifestInfo {
            items: Vec::new(),
            id_to_item: std::collections::HashMap::new(),
        };
        
        assert!(info.items.is_empty());
        assert!(info.id_to_item.is_empty());
    }

    #[test]
    fn test_manifest_info_with_items() {
        let mut id_to_item = std::collections::HashMap::new();
        let item = ManifestItem {
            id: "chapter1".to_string(),
            href: "chapter1.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: vec![],
        };
        id_to_item.insert("chapter1".to_string(), item.clone());
        
        let info = ManifestInfo {
            items: vec![item.clone()],
            id_to_item,
        };
        
        assert_eq!(info.items.len(), 1);
        assert_eq!(info.id_to_item.len(), 1);
        assert!(info.id_to_item.contains_key("chapter1"));
    }

    #[test]
    fn test_manifest_info_lookup() {
        let mut id_to_item = std::collections::HashMap::new();
        let item = ManifestItem {
            id: "test".to_string(),
            href: "test.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: vec![],
        };
        id_to_item.insert("test".to_string(), item.clone());
        
        let info = ManifestInfo {
            items: vec![item],
            id_to_item,
        };
        
        let found = info.id_to_item.get("test");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test");
    }

    // ==================== TocEntry Tests ====================

    #[test]
    fn test_toc_entry_simple() {
        let entry = TocEntry {
            title: "Chapter 1".to_string(),
            href: "chapter1.xhtml".to_string(),
            children: Vec::new(),
        };
        
        assert_eq!(entry.title, "Chapter 1");
        assert_eq!(entry.href, "chapter1.xhtml");
        assert!(entry.children.is_empty());
    }

    #[test]
    fn test_toc_entry_with_children() {
        let child1 = TocEntry {
            title: "Section 1.1".to_string(),
            href: "section1-1.xhtml".to_string(),
            children: Vec::new(),
        };
        
        let child2 = TocEntry {
            title: "Section 1.2".to_string(),
            href: "section1-2.xhtml".to_string(),
            children: Vec::new(),
        };
        
        let entry = TocEntry {
            title: "Chapter 1".to_string(),
            href: "chapter1.xhtml".to_string(),
            children: vec![child1, child2],
        };
        
        assert_eq!(entry.children.len(), 2);
        assert_eq!(entry.children[0].title, "Section 1.1");
        assert_eq!(entry.children[1].title, "Section 1.2");
    }

    #[test]
    fn test_toc_entry_clone() {
        let entry = TocEntry {
            title: "Test".to_string(),
            href: "test.xhtml".to_string(),
            children: vec![
                TocEntry {
                    title: "Child".to_string(),
                    href: "child.xhtml".to_string(),
                    children: vec![],
                },
            ],
        };
        let cloned = entry.clone();
        
        assert_eq!(entry.title, cloned.title);
        assert_eq!(entry.href, cloned.href);
        assert_eq!(entry.children.len(), cloned.children.len());
    }

    // ==================== NavInfo Tests ====================

    #[test]
    fn test_nav_info_no_navigation() {
        let info = NavInfo {
            nav_path: None,
            ncx_path: None,
            toc_entries: Vec::new(),
        };
        
        assert!(info.nav_path.is_none());
        assert!(info.ncx_path.is_none());
        assert!(info.toc_entries.is_empty());
    }

    #[test]
    fn test_nav_info_with_nav() {
        let info = NavInfo {
            nav_path: Some("OEBPS/nav.xhtml".to_string()),
            ncx_path: None,
            toc_entries: vec![
                TocEntry {
                    title: "Chapter 1".to_string(),
                    href: "chapter1.xhtml".to_string(),
                    children: vec![],
                },
            ],
        };
        
        assert_eq!(info.nav_path, Some("OEBPS/nav.xhtml".to_string()));
        assert!(info.ncx_path.is_none());
        assert_eq!(info.toc_entries.len(), 1);
    }

    #[test]
    fn test_nav_info_with_ncx() {
        let info = NavInfo {
            nav_path: None,
            ncx_path: Some("OEBPS/toc.ncx".to_string()),
            toc_entries: vec![],
        };
        
        assert!(info.nav_path.is_none());
        assert_eq!(info.ncx_path, Some("OEBPS/toc.ncx".to_string()));
    }

    #[test]
    fn test_nav_info_clone() {
        let info = NavInfo {
            nav_path: Some("OEBPS/nav.xhtml".to_string()),
            ncx_path: None,
            toc_entries: vec![
                TocEntry {
                    title: "Test".to_string(),
                    href: "test.xhtml".to_string(),
                    children: vec![],
                },
            ],
        };
        let cloned = info.clone();
        
        assert_eq!(info.nav_path, cloned.nav_path);
        assert_eq!(info.ncx_path, cloned.ncx_path);
        assert_eq!(info.toc_entries.len(), cloned.toc_entries.len());
    }

    // ==================== OpfPackage Tests ====================

    #[test]
    fn test_opf_package() {
        let package = OpfPackage {
            path: "OEBPS/content.opf".to_string(),
            root_file_path: "OEBPS/content.opf".to_string(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            manifest_items: Vec::new(),
            spine_items: Vec::new(),
        };
        
        assert_eq!(package.path, "OEBPS/content.opf");
        assert_eq!(package.root_file_path, "OEBPS/content.opf");
    }

    #[test]
    fn test_opf_package_with_data() {
        let metadata: serde_json::Value = serde_json::json!({
            "dc:title": "Test Book",
            "dc:creator": "Test Author"
        });
        
        let package = OpfPackage {
            path: "OEBPS/content.opf".to_string(),
            root_file_path: "OEBPS/content.opf".to_string(),
            metadata,
            manifest_items: vec![
                ManifestItem {
                    id: "chapter1".to_string(),
                    href: "chapter1.xhtml".to_string(),
                    media_type: "application/xhtml+xml".to_string(),
                    properties: vec![],
                },
            ],
            spine_items: vec![
                SpineItem {
                    idref: "chapter1".to_string(),
                    linear: "yes".to_string(),
                    properties: vec![],
                },
            ],
        };
        
        assert_eq!(package.manifest_items.len(), 1);
        assert_eq!(package.spine_items.len(), 1);
    }

    #[test]
    fn test_opf_package_clone() {
        let package = OpfPackage {
            path: "OEBPS/content.opf".to_string(),
            root_file_path: "OEBPS/content.opf".to_string(),
            metadata: serde_json::Value::Null,
            manifest_items: vec![],
            spine_items: vec![],
        };
        let cloned = package.clone();
        
        assert_eq!(package.path, cloned.path);
        assert_eq!(package.root_file_path, cloned.root_file_path);
        assert_eq!(package.manifest_items.len(), cloned.manifest_items.len());
        assert_eq!(package.spine_items.len(), cloned.spine_items.len());
    }
}
