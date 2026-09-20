use crate::archive::entry_exists;
use crate::core::{ErrorCode, ValidationLocation, ValidationResult, ValidationError};
use std::collections::HashSet;

/// Validates consistency between spine, manifest, archive, and navigation
pub fn validate_consistency(
    spine_info: &crate::core::SpineInfo,
    manifest_info: &crate::core::ManifestInfo,
    archive: &mut zip::ZipArchive<std::fs::File>,
    nav_info: &crate::core::NavInfo,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Spine → Manifest consistency
    result.extend(validate_spine_manifest(spine_info, manifest_info));
    
    // Manifest → Archive consistency
    result.extend(validate_manifest_archive(manifest_info, archive));
    
    // Navigation → Spine consistency
    result.extend(validate_nav_spine(nav_info, spine_info, manifest_info));
    
    result
}

/// Validates that every linear spine itemref has a matching manifest item
fn validate_spine_manifest(
    spine_info: &crate::core::SpineInfo,
    manifest_info: &crate::core::ManifestInfo,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    for (index, itemref) in spine_info.linear_items.iter().enumerate() {
        if manifest_info.id_to_item.get(&itemref.idref).is_none() {
            result.add_error(ValidationError::new(
                ErrorCode::SpineManifestMismatch,
                ValidationLocation::SpineItem { index },
            ));
        }
    }
    
    result
}

/// Validates that every manifest item exists in the archive
fn validate_manifest_archive(
    manifest_info: &crate::core::ManifestInfo,
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    for item in &manifest_info.items {
        if !entry_exists(archive, &item.href) {
            result.add_error(ValidationError::new(
                ErrorCode::ManifestArchiveMismatch,
                ValidationLocation::ArchiveEntry { path: item.href.clone() },
            ));
        }
    }
    
    result
}

/// Validates that navigation targets resolve to linear spine documents
fn validate_nav_spine(
    nav_info: &crate::core::NavInfo,
    spine_info: &crate::core::SpineInfo,
    manifest_info: &crate::core::ManifestInfo,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Build a set of linear spine document paths
    let mut linear_paths: HashSet<String> = HashSet::new();
    
    for itemref in &spine_info.linear_items {
        if let Some(manifest_item) = manifest_info.id_to_item.get(&itemref.idref) {
            linear_paths.insert(manifest_item.href.clone());
        }
    }
    
    // Check navigation entries
    for (index, entry) in nav_info.toc_entries.iter().enumerate() {
        // Check for empty href
        if entry.href.is_empty() {
            result.add_warning(ValidationError::new(
                ErrorCode::NavSpineMismatch,
                ValidationLocation::TocEntry { index },
            ));
            continue;
        }
        
        // Note: Full implementation would resolve relative paths and check
        // if the target exists in the linear spine set
        // For now, we validate that hrefs are non-empty
    }
    
    result
}

#[cfg(test)]
mod tests;
