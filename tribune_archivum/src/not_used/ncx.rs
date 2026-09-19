use crate::archive::read_entry;
use crate::core::{ErrorCode, NavInfo, TocEntry, ValidationLocation, ValidationResult};
use quick_xml::de::from_str;
use std::io::Read;

/// EPUB2 NCX document root element
#[derive(Debug, Deserialize)]
struct NcxDocument {
    #[serde(rename = "@xmlns")]
    namespace: String,
    #[serde(rename = "head", default)]
    head: Vec<NcxHead>,
    #[serde(rename = "navMap")]
    nav_map: NcxNavMap,
}

#[derive(Debug, Deserialize)]
struct NcxHead {
    #[serde(rename = "meta")]
    metas: Vec<NcxMeta>,
}

#[derive(Debug, Deserialize)]
struct NcxMeta {
    #[serde(attribute)]
    name: String,
    #[serde(attribute)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct NcxNavMap {
    #[serde(rename = "navPoint", default)]
    nav_points: Vec<NcxNavPoint>,
}

#[derive(Debug, Deserialize)]
struct NcxNavPoint {
    #[serde(attribute)]
    id: String,
    #[serde(attribute)]
    order: Option<String>,
    #[serde(rename = "navLabel")]
    nav_label: NcxNavLabel,
    #[serde(rename = "content")]
    content: NcxContent,
    #[serde(rename = "navPoint", default)]
    children: Vec<NcxNavPoint>,
}

#[derive(Debug, Deserialize)]
struct NcxNavLabel {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct NcxContent {
    #[serde(attribute)]
    src: String,
}

/// Validates the EPUB2 NCX document
pub fn validate_ncx(
    archive: &mut ZipArchive<std::fs::File>,
    ncx_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Read NCX document
    let ncx_data = match read_entry(archive, ncx_path) {
        Ok(data) => data,
        Err((mut res, _)) => return Err(res),
    };
    
    // Parse NCX document
    let ncx_str = match String::from_utf8(ncx_data.clone()) {
        Ok(s) => s,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidNcxXml,
                ValidationLocation::Ncx { path: ncx_path.to_string() },
            ));
            return Err(result);
        }
    };
    
    let ncx: NcxDocument = match from_str(&ncx_str) {
        Ok(n) => n,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidNcxXml,
                ValidationLocation::Ncx { path: ncx_path.to_string() },
            ));
            return Err(result);
        }
    };
    
    // Extract TOC entries from navPoints
    let mut toc_entries = extract_toc_entries(&ncx.nav_map.nav_points);
    
    // Check for empty TOC
    if toc_entries.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::EmptyToc,
            ValidationLocation::Ncx { path: ncx_path.to_string() },
        ));
    }
    
    Ok(NavInfo {
        nav_path: None,
        ncx_path: Some(ncx_path.to_string()),
        toc_entries,
    })
}

/// Recursively extracts TOC entries from NCX navPoints
fn extract_toc_entries(nav_points: &[NcxNavPoint]) -> Vec<TocEntry> {
    nav_points
        .iter()
        .filter_map(|point| {
            // Skip fragment-only links
            if point.content.src.is_empty() || point.content.src.starts_with('#') {
                return None;
            }
            
            Some(TocEntry {
                title: point.nav_label.text.clone(),
                href: point.content.src.clone(),
                children: extract_toc_entries(&point.children),
            })
        })
        .collect()
}
