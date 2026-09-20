#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ErrorCode, ManifestItem, ValidationLocation, ValidationResult, ValidationError};

    #[test]
    fn test_valid_manifest() {
        let manifest_items = vec![
            ManifestItem {
                id: "ncx".to_string(),
                href: "toc.ncx".to_string(),
                media_type: "application/x-dtbncx+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "nav".to_string(),
                href: "nav.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Valid manifest should pass validation");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 3);
        assert_eq!(manifest_info.id_to_item.len(), 3);
    }

    #[test]
    fn test_empty_manifest() {
        let manifest_items: Vec<ManifestItem> = vec![];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_err(), "Empty manifest should fail validation");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::MissingManifest
        }), "Should report MissingManifest error");
    }

    #[test]
    fn test_manifest_with_empty_id() {
        let manifest_items = vec![
            ManifestItem {
                id: "".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_err(), "Manifest with empty ID should fail");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::InvalidManifestId
        }), "Should report InvalidManifestId error");
        
        // Should still return partial result
        let manifest_info = result.unwrap_err();
        assert_eq!(manifest_info.errors.len(), 1);
    }

    #[test]
    fn test_manifest_with_duplicate_ids() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1_backup.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_err(), "Manifest with duplicate IDs should fail");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::DuplicateManifestId
        }), "Should report DuplicateManifestId error");
        
        assert_eq!(validation_result.errors.len(), 1);
    }

    #[test]
    fn test_manifest_with_missing_media_type() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with missing media-type should still pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 2);
        assert_eq!(manifest_info.id_to_item.len(), 2);
    }

    #[test]
    fn test_manifest_with_multiple_missing_media_types() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter3".to_string(),
                href: "chapter3.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with multiple missing media-types should still pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 3);
        assert_eq!(manifest_info.id_to_item.len(), 3);
    }

    #[test]
    fn test_manifest_with_special_characters_in_id() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter-1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter_2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter.3".to_string(),
                href: "chapter3.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with special characters in ID should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 3);
        assert_eq!(manifest_info.id_to_item.len(), 3);
    }

    #[test]
    fn test_manifest_with_special_characters_in_href() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter/1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter/2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with special characters in href should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 2);
        assert_eq!(manifest_info.id_to_item.len(), 2);
    }

    #[test]
    fn test_manifest_id_to_item_lookup() {
        let manifest_items = vec![
            ManifestItem {
                id: "ncx".to_string(),
                href: "toc.ncx".to_string(),
                media_type: "application/x-dtbncx+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "nav".to_string(),
                href: "nav.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest should allow ID lookup");
        let manifest_info = result.unwrap();

        // Test lookup
        let ncx_item = manifest_info.id_to_item.get("ncx").unwrap();
        assert_eq!(ncx_item.href, "toc.ncx");
        assert_eq!(ncx_item.media_type, "application/x-dtbncx+xml");

        let nav_item = manifest_info.id_to_item.get("nav").unwrap();
        assert_eq!(nav_item.href, "nav.xhtml");
    }

    #[test]
    fn test_manifest_with_properties() {
        let manifest_items = vec![
            ManifestItem {
                id: "nav".to_string(),
                href: "nav.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec!["nav".to_string()],
            },
            ManifestItem {
                id: "cover".to_string(),
                href: "cover.jpg".to_string(),
                media_type: "image/jpeg".to_string(),
                properties: vec!["cover-image".to_string()],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with properties should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 3);
        assert_eq!(manifest_info.id_to_item.len(), 3);
    }

    #[test]
    fn test_manifest_with_multiple_duplicate_ids() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1_backup.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1_copy.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_err(), "Manifest with multiple duplicate IDs should fail");
        let validation_result = result.unwrap_err();
        
        // Should report duplicate for each duplicate occurrence
        assert_eq!(validation_result.errors.len(), 2);
        assert!(validation_result.errors.iter().all(|e| {
            e.code == ErrorCode::DuplicateManifestId
        }), "Should report DuplicateManifestId errors");
    }

    #[test]
    fn test_manifest_with_empty_href() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with empty href should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 1);
        assert_eq!(manifest_info.id_to_item.len(), 1);
    }

    #[test]
    fn test_manifest_with_empty_media_type() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with empty media-types should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 2);
        assert_eq!(manifest_info.id_to_item.len(), 2);
    }

    #[test]
    fn test_manifest_single_item() {
        let manifest_items = vec![
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest with single item should pass");
        let manifest_info = result.unwrap();

        assert_eq!(manifest_info.items.len(), 1);
        assert_eq!(manifest_info.id_to_item.len(), 1);
    }

    #[test]
    fn test_manifest_preserves_item_order() {
        let manifest_items = vec![
            ManifestItem {
                id: "ncx".to_string(),
                href: "toc.ncx".to_string(),
                media_type: "application/x-dtbncx+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "nav".to_string(),
                href: "nav.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter1".to_string(),
                href: "chapter1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
            ManifestItem {
                id: "chapter2".to_string(),
                href: "chapter2.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: vec![],
            },
        ];

        let result = validate_manifest(&manifest_items);

        assert!(result.is_ok(), "Manifest should preserve order");
        let manifest_info = result.unwrap();

        // Check that items are in correct order
        assert_eq!(manifest_info.items[0].id, "ncx");
        assert_eq!(manifest_info.items[1].id, "nav");
        assert_eq!(manifest_info.items[2].id, "chapter1");
        assert_eq!(manifest_info.items[3].id, "chapter2");
    }
}
