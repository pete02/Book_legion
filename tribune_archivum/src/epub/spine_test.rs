#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ErrorCode, SpineItem, ValidationLocation, ValidationResult, ValidationError};

    #[test]
    fn test_valid_spine_with_linear_items() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter2".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter3".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Valid spine should pass validation");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 3);
        assert_eq!(spine_info.linear_items.len(), 3);
    }

    #[test]
    fn test_spine_with_mixed_linear_items() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "cover".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
            SpineItem {
                idref: "chapter2".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine with mixed linear items should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 3);
        assert_eq!(spine_info.linear_items.len(), 2);
    }

    #[test]
    fn test_empty_spine() {
        let spine_items: Vec<SpineItem> = vec![];

        let result = validate_spine(&spine_items);

        assert!(result.is_err(), "Empty spine should fail validation");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::EmptySpine
        }), "Should report EmptySpine error");
    }

    #[test]
    fn test_spine_with_all_nonlinear_items() {
        let spine_items = vec![
            SpineItem {
                idref: "cover".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
            SpineItem {
                idref: "frontmatter".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_err(), "Spine with all non-linear items should fail");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::EmptySpine
        }), "Should report EmptySpine error");
    }

    #[test]
    fn test_spine_with_single_linear_item() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine with single linear item should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 1);
        assert_eq!(spine_info.linear_items.len(), 1);
    }

    #[test]
    fn test_spine_with_single_nonlinear_item() {
        let spine_items = vec![
            SpineItem {
                idref: "cover".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_err(), "Spine with single non-linear item should fail");
        let validation_result = result.unwrap_err();
        
        assert!(validation_result.errors.iter().any(|e| {
            e.code == ErrorCode::EmptySpine
        }), "Should report EmptySpine error");
    }

    #[test]
    fn test_spine_preserves_item_order() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter2".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
            SpineItem {
                idref: "chapter3".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter4".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine should preserve order");
        let spine_info = result.unwrap();

        // Check that original items are preserved in order
        assert_eq!(spine_info.items[0].idref, "chapter1");
        assert_eq!(spine_info.items[1].idref, "chapter2");
        assert_eq!(spine_info.items[2].idref, "chapter3");
        assert_eq!(spine_info.items[3].idref, "chapter4");

        // Check that linear items are in correct order
        assert_eq!(spine_info.linear_items[0].idref, "chapter1");
        assert_eq!(spine_info.linear_items[1].idref, "chapter3");
        assert_eq!(spine_info.linear_items[2].idref, "chapter4");
    }

    #[test]
    fn test_spine_with_properties() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec!["pagebreak".to_string()],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter2".to_string(),
                id: None,
                properties: vec!["svg".to_string()],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine with properties should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 2);
        assert_eq!(spine_info.linear_items.len(), 2);
    }

    #[test]
    fn test_spine_default_linear_value() {
        // Test that items with "yes" are treated as linear
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter2".to_string(),
                id: None,
                properties: vec![],
                linear: "no".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine should handle linear attribute");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.linear_items.len(), 1);
        assert_eq!(spine_info.linear_items[0].idref, "chapter1");
    }

    #[test]
    fn test_spine_with_multiple_items_same_idref() {
        // This tests edge case where same idref appears multiple times
        let spine_items = vec![
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        // Should pass - spine validation doesn't check for duplicate idrefs
        assert!(result.is_ok(), "Spine with duplicate idrefs should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 2);
        assert_eq!(spine_info.linear_items.len(), 2);
    }

    #[test]
    fn test_spine_with_empty_idref() {
        let spine_items = vec![
            SpineItem {
                idref: "".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        // Empty idref is allowed by spine validation (consistency check elsewhere)
        assert!(result.is_ok(), "Spine with empty idref should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 1);
        assert_eq!(spine_info.linear_items.len(), 1);
    }

    #[test]
    fn test_spine_with_special_characters_in_idref() {
        let spine_items = vec![
            SpineItem {
                idref: "chapter-1".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter_2".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
            SpineItem {
                idref: "chapter.3".to_string(),
                id: None,
                properties: vec![],
                linear: "yes".to_string(),
            },
        ];

        let result = validate_spine(&spine_items);

        assert!(result.is_ok(), "Spine with special characters in idref should pass");
        let spine_info = result.unwrap();

        assert_eq!(spine_info.items.len(), 3);
        assert_eq!(spine_info.linear_items.len(), 3);
    }
}
