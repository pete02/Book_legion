use crate::core::{ErrorCode, ManifestInfo, ValidationLocation, ValidationResult, ValidationError};
use std::collections::HashMap;

/// Validates the manifest structure
pub fn validate_manifest(
    manifest_items: &[crate::core::ManifestItem],
) -> Result<ManifestInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    let mut id_to_item: HashMap<String, crate::core::ManifestItem> = HashMap::new();
    
    // Check for empty manifest
    if manifest_items.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::MissingManifest,
            ValidationLocation::Manifest,
        ));
        return Err(result);
    }
    
    // Check for duplicate IDs and validate IDs
    for item in manifest_items {
        // Check for empty ID
        if item.id.is_empty() {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidManifestId,
                ValidationLocation::ArchiveEntry { path: item.href.clone() },
            ));
            continue;
        }
        
        // Check for duplicate IDs
        if id_to_item.contains_key(&item.id) {
            result.add_error(ValidationError::new(
                ErrorCode::DuplicateManifestId,
                ValidationLocation::ArchiveEntry { path: item.href.clone() },
            ));
        }
        
        // Check for missing media-type
        if item.media_type.is_empty() {
            result.add_warning(ValidationError::new(
                ErrorCode::MissingMediaType,
                ValidationLocation::ArchiveEntry { path: item.href.clone() },
            ));
        }
        
        id_to_item.insert(item.id.clone(), item.clone());
    }
    
    Ok(ManifestInfo {
        items: manifest_items.to_vec(),
        id_to_item,
    })
}
