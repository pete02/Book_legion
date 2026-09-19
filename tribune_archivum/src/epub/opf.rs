use crate::core::{ErrorCode, OpfPackage, ValidationLocation, ValidationResult, ValidationError};
use quick_xml::de::from_str;
use serde::Deserialize;

/// OPF Package document root element
#[derive(Debug, Deserialize)]
struct OpfPackageRoot {
    #[serde(rename = "@xmlns")]
    #[allow(dead_code)]
    namespace: String,
    #[serde(rename = "@version")]
    #[allow(dead_code)]
    version: String,
    #[serde(rename = "metadata")]
    metadata: Metadata,
    #[serde(rename = "manifest")]
    manifest: OpfManifest,
    #[serde(rename = "spine")]
    spine: OpfSpine,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "dc:title", default)]
    title: Option<String>,
    #[serde(rename = "dc:creator", default)]
    creators: Vec<String>,
    #[serde(rename = "dc:language", default)]
    language: Option<String>,
    #[serde(rename = "dc:identifier", default)]
    identifiers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpfManifest {
    #[serde(rename = "item", default)]
    items: Vec<ManifestItem>,
}

#[derive(Debug, Deserialize)]
struct OpfSpine {
    #[serde(rename = "@toc")]
    #[allow(dead_code)]
    toc: Option<String>,
    #[serde(rename = "itemref", default)]
    items: Vec<SpineItem>,
}

#[derive(Debug, Deserialize)]
struct ManifestItem {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@media-type")]
    media_type: String,
    #[serde(rename = "@properties", default)]
    properties: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpineItem {
    #[serde(rename = "@idref")]
    idref: String,
    #[serde(rename = "@linear", default = "default_linear")]
    linear: String,
    #[serde(rename = "@properties", default)]
    properties: Option<String>,
}

fn default_linear() -> String {
    "yes".to_string()
}

/// Parses and validates the OPF document
pub fn parse_opf(opf_data: &[u8]) -> Result<OpfPackage, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Parse OPF XML
    let opf_str = match String::from_utf8(opf_data.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidOpfXml,
                ValidationLocation::Opf { path: String::new() },
            ));
            return Err(result);
        }
    };
    
    let opf: OpfPackageRoot = match from_str(&opf_str) {
        Ok(o) => o,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidOpfXml,
                ValidationLocation::Opf { path: String::new() },
            ));
            return Err(result);
        }
    };
    
    // Check for manifest
    if opf.manifest.items.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::MissingManifest,
            ValidationLocation::Opf { path: String::new() },
        ));
    }
    
    // Check for spine
    if opf.spine.items.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::MissingSpine,
            ValidationLocation::Opf { path: String::new() },
        ));
    }
    
    // Convert to our internal types
    let manifest_items: Vec<crate::core::ManifestItem> = opf
        .manifest
        .items
        .iter()
        .map(|item| crate::core::ManifestItem {
            id: item.id.clone(),
            href: item.href.clone(),
            media_type: item.media_type.clone(),
            properties: item
                .properties
                .clone()
                .map(|p| p.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
        .collect();
    
    let spine_items: Vec<crate::core::SpineItem> = opf
        .spine
        .items
        .iter()
        .map(|item| crate::core::SpineItem {
            idref: item.idref.clone(),
            linear: item.linear.clone(),
            properties: item
                .properties
                .clone()
                .map(|p| p.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
        .collect();
    
    // Convert metadata to JSON
    let metadata = serde_json::json!({
        "title": opf.metadata.title,
        "creators": opf.metadata.creators,
        "language": opf.metadata.language,
        "identifiers": opf.metadata.identifiers,
    });
    
    Ok(OpfPackage {
        path: String::new(),
        root_file_path: String::new(),
        metadata,
        manifest_items,
        spine_items,
    })
}
