use crate::core::{ErrorCode, SpineInfo, ValidationLocation, ValidationResult, ValidationError};

/// Validates the spine structure
pub fn validate_spine(
    spine_items: &[crate::core::SpineItem],
) -> Result<SpineInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Check for empty spine
    if spine_items.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::EmptySpine,
            ValidationLocation::Spine,
        ));
        return Err(result);
    }
    
    // Separate linear and non-linear items
    let linear_items: Vec<crate::core::SpineItem> = spine_items
        .iter()
        .filter(|item| item.linear == "yes")
        .cloned()
        .collect();
    
    // Check if spine is completely non-linear
    if linear_items.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::EmptySpine,
            ValidationLocation::Spine,
        ));
    }
    
    Ok(SpineInfo {
        items: spine_items.to_vec(),
        linear_items,
    })
}
