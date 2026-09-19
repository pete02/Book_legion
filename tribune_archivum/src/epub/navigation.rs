use crate::archive::entry_exists;
use crate::archive::read_entry;
use crate::core::{ErrorCode, ManifestInfo, NavInfo, ValidationLocation, ValidationResult, ValidationError};
use quick_xml::de::from_str;
use serde::Deserialize;
use zip::ZipArchive;

/// EPUB3 navigation document root element
#[derive(Debug, Deserialize)]
struct NavDocument {
    #[serde(rename = "nav")]
    nav_element: NavElement,
}

#[derive(Debug, Deserialize)]
struct NavElement {
    #[serde(rename = "ol")]
    ordered_list: Option<NavList>,
}

#[derive(Debug, Deserialize)]
struct NavList {
    #[serde(rename = "li")]
    list_items: Vec<NavItem>,
}

#[derive(Debug, Deserialize)]
struct NavItem {
    #[serde(rename = "a", default)]
    links: Vec<NavLink>,
    #[serde(rename = "ol", default)]
    subitems: Option<NavList>,
}

#[derive(Debug, Deserialize)]
struct NavLink {
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "#text", default)]
    title: Option<String>,
}

/// EPUB2 NCX document root element
#[derive(Debug, Deserialize)]
struct NcxDocument {
    #[serde(rename = "@xmlns")]
    #[allow(dead_code)]
    namespace: String,
    #[serde(rename = "navMap")]
    nav_map: NcxNavMap,
}

#[derive(Debug, Deserialize)]
struct NcxNavMap {
    #[serde(rename = "navPoint", default)]
    nav_points: Vec<NcxNavPoint>,
}

#[derive(Debug, Deserialize)]
struct NcxNavPoint {
    #[serde(rename = "@id")]
    #[allow(dead_code)]
    id: String,
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
    #[serde(rename = "@src")]
    src: String,
}

/// Discovers and validates navigation documents (EPUB3 nav or EPUB2 NCX)
pub fn validate_navigation(
    archive: &mut ZipArchive<std::fs::File>,
    opf: &crate::core::OpfPackage,
    manifest_info: &ManifestInfo,
) -> Result<NavInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Try to find EPUB3 nav document first
    if let Some(nav_path) = find_nav_document(manifest_info) {
        if entry_exists(archive, &nav_path) {
            return validate_epub3_nav(archive, &nav_path);
        }
    }
    
    // Fall back to EPUB2 NCX
    if let Some(ncx_path) = find_ncx_document(opf, manifest_info) {
        if entry_exists(archive, &ncx_path) {
            return validate_ncx(archive, &ncx_path);
        }
    }
    
    // No navigation document found
    result.add_error(ValidationError::new(
        ErrorCode::MissingNavDocument,
        ValidationLocation::Root,
    ));
    
    Err(result)
}

/// Finds the EPUB3 navigation document from manifest
fn find_nav_document(manifest_info: &ManifestInfo) -> Option<String> {
    for item in &manifest_info.items {
        if item.properties.contains(&"nav".to_string()) {
            return Some(item.href.clone());
        }
    }
    None
}

/// Finds the EPUB2 NCX document from manifest or spine
fn find_ncx_document(
    opf: &crate::core::OpfPackage,
    manifest_info: &ManifestInfo,
) -> Option<String> {
    // Try to find by media-type first
    for item in &manifest_info.items {
        if item.media_type == "application/x-dtbncx+xml" {
            return Some(item.href.clone());
        }
    }
    
    // Fall back to spine toc reference
    if let Some(toc_idref) = &opf.spine_items.first().and_then(|s| {
        if s.properties.contains(&"nav".to_string()) {
            Some(s.idref.clone())
        } else {
            None
        }
    }) {
        if let Some(item) = manifest_info.id_to_item.get(toc_idref) {
            return Some(item.href.clone());
        }
    }
    
    None
}

/// Validates the EPUB3 navigation document
fn validate_epub3_nav(
    archive: &mut ZipArchive<std::fs::File>,
    nav_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Read nav document
    let nav_data: Vec<u8> = match read_entry(archive, nav_path) {
        Ok(data) => data,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::MissingNavDocument,
                ValidationLocation::Navigation { path: nav_path.to_string() },
            ));
            return Err(result);
        }
    };
    
    // Parse nav document
    let nav_str = match String::from_utf8(nav_data.clone()) {
        Ok(s) => s,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidNavXml,
                ValidationLocation::Navigation { path: nav_path.to_string() },
            ));
            return Err(result);
        }
    };
    
    let nav: NavDocument = match from_str(&nav_str) {
        Ok(n) => n,
        Err(_) => {
            result.add_error(ValidationError::new(
                ErrorCode::InvalidNavXml,
                ValidationLocation::Navigation { path: nav_path.to_string() },
            ));
            return Err(result);
        }
    };
    
    // Check for toc navigation
    let mut toc_entries = Vec::new();
    
    if let Some(nav_list) = nav.nav_element.ordered_list {
        for (index, item) in nav_list.list_items.iter().enumerate() {
            if let Some(link) = item.links.first() {
                // Skip fragment-only links
                if !link.href.is_empty() && !link.href.starts_with('#') {
                    let title = link.title.clone().unwrap_or_default();
                    let href = link.href.clone();
                    
                    // Check for empty href
                    if href.is_empty() {
                        result.add_error(ValidationError::new(
                            ErrorCode::InvalidTocEntry,
                            ValidationLocation::TocEntry { index },
                        ));
                        continue;
                    }
                    
                    let mut entry = crate::core::TocEntry {
                        title,
                        href,
                        children: Vec::new(),
                    };
                    
                    // Process subitems
                    if let Some(sublist) = item.subitems.as_ref() {
                        for subitem in &sublist.list_items {
                            if let Some(link) = subitem.links.first() {
                                if !link.href.is_empty() && !link.href.starts_with('#') {
                                    entry.children.push(crate::core::TocEntry {
                                        title: link.title.clone().unwrap_or_default(),
                                        href: link.href.clone(),
                                        children: Vec::new(),
                                    });
                                }
                            }
                        }
                    }
                    
                    toc_entries.push(entry);
                }
            }
        }
    }
    
    // Check for empty TOC
    if toc_entries.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::EmptyToc,
            ValidationLocation::Navigation { path: nav_path.to_string() },
        ));
    }
    
    Ok(NavInfo {
        nav_path: Some(nav_path.to_string()),
        ncx_path: None,
        toc_entries,
    })
}

/// Validates the EPUB2 NCX document
fn validate_ncx(
    archive: &mut ZipArchive<std::fs::File>,
    ncx_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Read NCX document
    let ncx_data: Vec<u8> = match read_entry(archive, ncx_path) {
        Ok(data) => data,
        Err(_) =>{
            result.add_error(ValidationError::new(
                ErrorCode::MissingNavDocument,
                ValidationLocation::Navigation { path: ncx_path.to_string() },
            ));
            return Err(result)
        },
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
    let toc_entries = extract_toc_entries(&ncx.nav_map.nav_points);
    
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
fn extract_toc_entries(nav_points: &[NcxNavPoint]) -> Vec<crate::core::TocEntry> {
    nav_points
        .iter()
        .filter_map(|point| {
            // Skip fragment-only links
            if point.content.src.is_empty() || point.content.src.starts_with('#') {
                return None;
            }
            
            Some(crate::core::TocEntry {
                title: point.nav_label.text.clone(),
                href: point.content.src.clone(),
                children: extract_toc_entries(&point.children),
            })
        })
        .collect()
}
