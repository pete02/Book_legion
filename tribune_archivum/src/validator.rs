use crate::archive::validate_archive;
use crate::consistency::validate_consistency;
use crate::core::ValidationResult;
use crate::epub::{parse_opf, validate_container, validate_manifest, validate_navigation, validate_spine};

use zip::ZipArchive;
use std::fs::File;

/// Main validation function that orchestrates all six phases
pub fn validate_epub(epub_path: &str) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Phase 1: Archive integrity validation
    let _archive_info = match validate_archive(epub_path) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Open archive for reading
    let mut archive = match ZipArchive::new(File::open(epub_path).unwrap()) {
        Ok(archive) => archive,
        Err(_) => {
            result.add_error(crate::core::ValidationError::new(
                crate::core::ErrorCode::InvalidZip,
                crate::core::ValidationLocation::Root,
            ));
            return result;
        }
    };
    
    // Phase 2: Container validation
    let opf_info = match validate_container(&mut archive) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Phase 3: OPF parsing
    let opf_package = match parse_opf(&opf_info.opf_data) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Phase 3: Spine validation
    let spine_info = match validate_spine(&opf_package.spine_items) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Phase 4: Manifest validation
    let manifest_info = match validate_manifest(&opf_package.manifest_items) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Phase 5: Navigation validation
    let nav_info = match validate_navigation(&mut archive, &opf_package, &manifest_info) {
        Ok(info) => info,
        Err(res) => return res,
    };
    
    // Phase 6: Consistency validation
    let consistency_result = validate_consistency(
        &spine_info,
        &manifest_info,
        &mut archive,
        &nav_info,
    );
    
    // Add consistency results to main result
    result.extend(consistency_result);
    

    
    result
}
