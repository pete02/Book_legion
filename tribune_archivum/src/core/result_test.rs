#[cfg(test)]
mod tests {
    use super::super::result::*;
    use crate::core::types::{ErrorCode, Severity, ValidationLocation, ValidationError};

    // ==================== ValidationResult Tests ====================

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        let error = ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root);
        
        result.add_error(error.clone());
        
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, ErrorCode::InvalidZip);
        assert!(!result.passed);
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();
        let warning = ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() });
        
        result.add_warning(warning.clone());
        
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code, ErrorCode::MissingCover);
        assert!(!result.passed);
    }

    #[test]
    fn test_validation_result_add_error_severity() {
        let mut result = ValidationResult::new();
        let error = ValidationError {
            code: ErrorCode::InvalidZip,
            message: "Not a valid ZIP archive".to_string(),
            location: ValidationLocation::Root,
            severity: Severity::Error,
        };
        
        result.add(error.clone());
        
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_validation_result_add_warning_severity() {
        let mut result = ValidationResult::new();
        let warning = ValidationError {
            code: ErrorCode::MissingCover,
            message: "No cover image found".to_string(),
            location: ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() },
            severity: Severity::Warning,
        };
        
        result.add(warning.clone());
        
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_result_error_count() {
        let mut result = ValidationResult::new();
        
        assert_eq!(result.error_count(), 0);
        
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        assert_eq!(result.error_count(), 1);
        
        result.add_error(ValidationError::new(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() }));
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_validation_result_warning_count() {
        let mut result = ValidationResult::new();
        
        assert_eq!(result.warning_count(), 0);
        
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        assert_eq!(result.warning_count(), 1);
        
        result.add_warning(ValidationError::warning(ErrorCode::MissingChapterFile, ValidationLocation::ArchiveEntry { path: "OEBPS/chapter1.xhtml".to_string() }));
        assert_eq!(result.warning_count(), 2);
    }

    #[test]
    fn test_validation_result_has_errors() {
        let mut result = ValidationResult::new();
        
        assert!(!result.has_errors());
        
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        assert!(result.has_errors());
    }

    #[test]
    fn test_validation_result_has_warnings() {
        let mut result = ValidationResult::new();
        
        assert!(!result.has_warnings());
        
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validation_result_extend() {
        let mut result1 = ValidationResult::new();
        result1.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        result1.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        
        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::new(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() }));
        
        result1.extend(result2);
        
        assert_eq!(result1.error_count(), 2);
        assert_eq!(result1.warning_count(), 1);
        assert!(!result1.passed);
    }

    #[test]
    fn test_validation_result_extend_both_passed() {
        let mut result1 = ValidationResult::new();
        let result2 = ValidationResult::new();
        
        result1.extend(result2);
        
        assert!(result1.passed);
    }

    #[test]
    fn test_validation_result_extend_other_failed() {
        let mut result1 = ValidationResult::new();
        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        
        result1.extend(result2);
        
        assert!(!result1.passed);
    }

    #[test]
    fn test_validation_result_clone() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        
        let cloned = result.clone();
        
        assert_eq!(result.errors.len(), cloned.errors.len());
        assert_eq!(result.warnings.len(), cloned.warnings.len());
        assert_eq!(result.passed, cloned.passed);
    }

    #[test]
    fn test_validation_result_debug() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        
        let debug_output = format!("{:?}", result);
        assert!(debug_output.contains("errors"));
        assert!(debug_output.contains("warnings"));
        assert!(debug_output.contains("passed"));
    }

    // ==================== ValidationResultBuilder Tests ====================

    #[test]
    fn test_validation_result_builder_new() {
        let builder = ValidationResultBuilder::new();
        let result = builder.build();
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_builder_default() {
        let builder = ValidationResultBuilder::default();
        let result = builder.build();
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_builder_with_error() {
        let builder = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root);
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 0);
        assert!(!result.passed);
        assert_eq!(result.errors[0].code, ErrorCode::InvalidZip);
    }

    #[test]
    fn test_validation_result_builder_with_warning() {
        let builder = ValidationResultBuilder::new()
            .with_warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
        assert!(!result.passed);
        assert_eq!(result.warnings[0].code, ErrorCode::MissingCover);
    }

    #[test]
    fn test_validation_result_builder_with_multiple_errors() {
        let builder = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root)
            .with_error(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() })
            .with_error(ErrorCode::MissingContainerXml, ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 3);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_builder_with_multiple_warnings() {
        let builder = ValidationResultBuilder::new()
            .with_warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() })
            .with_warning(ErrorCode::MissingChapterFile, ValidationLocation::ArchiveEntry { path: "OEBPS/chapter1.xhtml".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 2);
    }

    #[test]
    fn test_validation_result_builder_mixed() {
        let builder = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root)
            .with_warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() })
            .with_error(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 2);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_validation_result_builder_with_errors() {
        let errors = vec![
            ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root),
            ValidationError::new(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() }),
        ];
        
        let builder = ValidationResultBuilder::new().with_errors(errors);
        let result = builder.build();
        
        assert_eq!(result.error_count(), 2);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_builder_with_errors_and_warnings() {
        let errors = vec![
            ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root),
        ];
        
        let builder = ValidationResultBuilder::new()
            .with_errors(errors)
            .with_warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() })
            .with_warning(ErrorCode::MissingChapterFile, ValidationLocation::ArchiveEntry { path: "OEBPS/chapter1.xhtml".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 2);
    }

    #[test]
    fn test_validation_result_builder_empty() {
        let builder = ValidationResultBuilder::new();
        let result = builder.build();
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_builder_chaining() {
        let result = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root)
            .with_warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() })
            .with_error(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() })
            .with_warning(ErrorCode::MissingChapterFile, ValidationLocation::ArchiveEntry { path: "OEBPS/chapter1.xhtml".to_string() })
            .build();
        
        assert_eq!(result.error_count(), 2);
        assert_eq!(result.warning_count(), 2);
        assert!(!result.passed);
    }

    // ==================== ValidationResult Edge Cases ====================

    #[test]
    fn test_validation_result_multiple_errors_same_code() {
        let mut result = ValidationResult::new();
        
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::ArchiveEntry { path: "test.epub".to_string() }));
        
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_validation_result_multiple_warnings_same_code() {
        let mut result = ValidationResult::new();
        
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.png".to_string() }));
        
        assert_eq!(result.warning_count(), 2);
    }

    #[test]
    fn test_validation_result_add_error_then_warning() {
        let mut result = ValidationResult::new();
        
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert!(!result.passed);
    }

    #[test]
    fn test_validation_result_add_warning_then_error() {
        let mut result = ValidationResult::new();
        
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::ArchiveEntry { path: "OEBPS/cover.jpg".to_string() }));
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert!(!result.passed);
    }

    #[test]
    fn test_validation_result_extend_empty() {
        let mut result = ValidationResult::new();
        let empty = ValidationResult::new();
        
        result.extend(empty);
        
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_validation_result_extend_only_errors() {
        let mut result1 = ValidationResult::new();
        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        
        result1.extend(result2);
        
        assert_eq!(result1.error_count(), 1);
        assert_eq!(result1.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_extend_only_warnings() {
        let mut result1 = ValidationResult::new();
        let mut result2 = ValidationResult::new();
        result2.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::Cover { path: "OEBPS/cover.jpg".to_string() }));
        
        result1.extend(result2);
        
        assert_eq!(result1.error_count(), 0);
        assert_eq!(result1.warning_count(), 1);
    }

    // ==================== ValidationResultBuilder Edge Cases ====================

    #[test]
    fn test_validation_result_builder_duplicate_errors() {
        let builder = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root)
            .with_error(ErrorCode::InvalidZip, ValidationLocation::ArchiveEntry { path: "test.epub".to_string() });
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_validation_result_builder_large_number_of_errors() {
        let mut builder = ValidationResultBuilder::new();
        
        for i in 0..100 {
            builder = builder.with_error(
                ErrorCode::InvalidZip,
                ValidationLocation::ArchiveEntry { path: format!("test{}.xml", i) }
            );
        }
        
        let result = builder.build();
        
        assert_eq!(result.error_count(), 100);
    }

    #[test]
    fn test_validation_result_builder_large_number_of_warnings() {
        let mut builder = ValidationResultBuilder::new();
        
        for i in 0..100 {
            builder = builder.with_warning(
                ErrorCode::MissingCover,
                ValidationLocation::Cover { path: format!("cover{}.jpg", i) }
            );
        }
        
        let result = builder.build();
        
        assert_eq!(result.warning_count(), 100);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_validation_result_builder_to_extend() {
        let result1 = ValidationResultBuilder::new()
            .with_error(ErrorCode::InvalidZip, ValidationLocation::Root)
            .build();
        
        let result2 = ValidationResultBuilder::new()
            .with_error(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() })
            .build();
        
        let mut combined = result1;
        combined.extend(result2);
        
        assert_eq!(combined.error_count(), 2);
        assert_eq!(combined.warning_count(), 0);
    }

    #[test]
    fn test_validation_result_separation() {
        let mut result = ValidationResult::new();
        
        result.add_error(ValidationError::new(ErrorCode::InvalidZip, ValidationLocation::Root));
        result.add_warning(ValidationError::warning(ErrorCode::MissingCover, ValidationLocation::Cover { path: "OEBPS/cover.jpg".to_string() }));
        result.add_error(ValidationError::new(ErrorCode::UnreadableEntry, ValidationLocation::ArchiveEntry { path: "test.xml".to_string() }));
        result.add_warning(ValidationError::warning(ErrorCode::MissingChapterFile, ValidationLocation::Chapter { path: "OEBPS/chapter1.xhtml".to_string() }));
        
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.warnings.len(), 2);
        
        // Verify errors are errors
        for error in &result.errors {
            assert_eq!(error.severity, Severity::Error);
        }
        
        // Verify warnings are warnings
        for warning in &result.warnings {
            assert_eq!(warning.severity, Severity::Warning);
        }
    }
}
