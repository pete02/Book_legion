use crate::archive::{entry_exists, read_entry};
use crate::core::{ErrorCode, OpfInfo, ValidationLocation, ValidationResult, ValidationError};
use quick_xml::de::from_str;
use serde::Deserialize;
use zip::ZipArchive;

/// Container XML root element
#[derive(Debug, Deserialize)]
struct ContainerRoot {
    #[serde(rename = "rootfiles")]
    root_files: RootFiles,
}

#[derive(Debug, Deserialize)]
struct RootFiles {
    #[serde(rename = "rootfile")]
    root_files: Vec<RootFile>,
}

#[derive(Debug, Deserialize)]
struct RootFile {
    #[serde(rename = "@full-path")]
    full_path: String,
}

/// Validates the container.xml file and discovers the OPF document
pub fn validate_container(
    archive: &mut ZipArchive<std::fs::File>,
) -> Result<OpfInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Check if container.xml exists
    if !entry_exists(archive, "META-INF/container.xml") {
        result.add_error(ValidationError::new(
            ErrorCode::MissingContainerXml,
            ValidationLocation::Root,
        ));
        return Err(result);
    }
    
    // Read container.xml
    let container_data = match read_entry(archive, "META-INF/container.xml") {
        Ok(data) => data,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidContainerXml,
                ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() },
            ));
            return Err(result);
        }
    };
    
    // Parse container.xml
    let container_str = match String::from_utf8(container_data.clone()) {
        Ok(s) => s,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidContainerXml,
                ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() },
            ));
            return Err(result);
        }
    };
    
    let container: ContainerRoot = match from_str(&container_str) {
        Ok(c) => c,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidContainerXml,
                ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() },
            ));
            return Err(result);
        }
    };
    
    // Check for rootfile elements
    if container.root_files.root_files.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::MissingOpfReference,
            ValidationLocation::ArchiveEntry { path: "META-INF/container.xml".to_string() },
        ));
        return Err(result);
    }
    
    // Get the first rootfile (should only be one in practice)
    let opf_path = &container.root_files.root_files[0].full_path;
    
    // Check if OPF file exists
    if !entry_exists(archive, opf_path) {
        result.add_error(ValidationError::new(
            ErrorCode::InvalidOpfReference,
            ValidationLocation::ArchiveEntry { path: opf_path.clone() },
        ));
        return Err(result);
    }
    
    // Read OPF file
    let opf_data: Vec<u8> = match read_entry(archive, opf_path) {
        Ok(data) => data,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidOpfReference,
                ValidationLocation::ArchiveEntry { path: opf_path.clone() },
            ));
            return Err(result);
        }
    };
    
    Ok(OpfInfo {
        opf_path: opf_path.clone(),
        opf_data,
    })
}
