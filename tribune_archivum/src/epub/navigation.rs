use crate::archive::entry_exists;
use crate::archive::read_entry;
use crate::core::{ErrorCode, ManifestInfo, NavInfo, ValidationLocation, ValidationResult, ValidationError};
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;
use zip::ZipArchive;

/// EPUB3 navigation document root element
#[derive(Debug, Deserialize)]
struct NavDocument {
    #[serde(rename = "body")]
    body: NavBody,
}

#[derive(Debug, Deserialize)]
struct NavBody {
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

/// Parsed navigation link with title
#[derive(Debug, Clone)]
struct ParsedNavLink {
    href: String,
    title: String,
}

/// Parsed navigation list item
#[derive(Debug, Clone)]
struct ParsedNavItem {
    links: Vec<ParsedNavLink>,
    subitems: Option<Vec<ParsedNavItem>>,
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

/// Manually parses EPUB3 nav XML using EventReader to extract text content
fn parse_nav_xml(nav_str: &str) -> Result<Vec<ParsedNavItem>, ()> {
    let mut reader = Reader::from_str(nav_str);
    reader.trim_text(true);
    
    let mut in_nav = false;
    let mut in_ol = false;
    let mut in_li = false;
    let mut current_item = ParsedNavItem {
        links: Vec::new(),
        subitems: None,
    };
    let mut current_link = ParsedNavLink {
        href: String::new(),
        title: String::new(),
    };
    let mut in_a = false;
    let mut in_sub_ol = false;
    let mut sub_items: Vec<ParsedNavItem> = Vec::new();
    let mut sub_current_item = ParsedNavItem {
        links: Vec::new(),
        subitems: None,
    };
    let mut sub_in_a = false;
    let mut sub_current_link = ParsedNavLink {
        href: String::new(),
        title: String::new(),
    };
    
    let mut result: Vec<ParsedNavItem> = Vec::new();
    
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local_name = e.name().0;
                
                eprintln!("DEBUG Start: local_name={:?}, in_nav={}, in_ol={}, in_li={}, in_sub_ol={}", 
                         std::str::from_utf8(local_name).unwrap_or("invalid"), in_nav, in_ol, in_li, in_sub_ol);
                
                if local_name == b"nav" {
                    eprintln!("DEBUG: Hit nav branch");
                    in_nav = true;
                } else if in_nav && local_name == b"ol" {
                    eprintln!("DEBUG: Hit nav+ol branch");
                    in_ol = true;
                } else if in_sub_ol && local_name == b"li" {
                    // Handle nested <li> first (before checking parent <li>)
                    eprintln!("DEBUG: Hit sub_ol+li branch");
                    sub_current_item = ParsedNavItem {
                        links: Vec::new(),
                        subitems: None,
                    };
                } else if in_ol && local_name == b"li" {
                    eprintln!("DEBUG: Hit ol+li branch");
                    in_li = true;
                    eprintln!("DEBUG: Found <li>, creating current_item");
                    current_item = ParsedNavItem {
                        links: Vec::new(),
                        subitems: None,
                    };
                } else if in_li && local_name == b"a" {
                    in_a = true;
                    eprintln!("DEBUG: Found <a> in <li>, creating current_link");
                    current_link = ParsedNavLink {
                        href: String::new(),
                        title: String::new(),
                    };
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"href" {
                            current_link.href = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                } else if in_li && local_name == b"ol" {
                    eprintln!("DEBUG: Hit li+ol branch");
                    in_sub_ol = true;
                    sub_items = Vec::new();
                    sub_current_item = ParsedNavItem {
                        links: Vec::new(),
                        subitems: None,
                    };
                } else if in_sub_ol && local_name == b"a" {
                    sub_in_a = true;
                    sub_current_link = ParsedNavLink {
                        href: String::new(),
                        title: String::new(),
                    };
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"href" {
                            sub_current_link.href = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();
                let text = text.trim().to_string();
                
                if in_a {
                    if !text.is_empty() {
                        eprintln!("DEBUG: Setting current_link.title to '{}'", text);
                        current_link.title = text;
                    }
                } else if sub_in_a {
                    if !text.is_empty() {
                        eprintln!("DEBUG: Setting sub_current_link.title to '{}'", text);
                        sub_current_link.title = text;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.name().0;
                
                if local_name == b"a" {
                    if in_a {
                        current_item.links.push(current_link.clone());
                        in_a = false;
                    } else if sub_in_a {
                        sub_current_item.links.push(sub_current_link.clone());
                        sub_in_a = false;
                    }
                } else if local_name == b"li" {
                    if in_sub_ol {
                        eprintln!("DEBUG: Pushing sub_item with {} links", sub_current_item.links.len());
                        sub_items.push(sub_current_item.clone());
                        sub_in_a = false;
                    } else if in_li {
                        eprintln!("DEBUG: Pushing current_item with {} links", current_item.links.len());
                        in_li = false;
                        result.push(current_item.clone());
                    }
                } else if local_name == b"ol" {
                    if in_sub_ol {
                        in_sub_ol = false;
                        current_item.subitems = Some(sub_items.clone());
                    } else if in_ol {
                        in_ol = false;
                    }
                } else if local_name == b"nav" {
                    in_nav = false;
                }
            }
            Ok(Event::Eof) => {
                break;
            }
            Err(_) => {
                return Err(());
            }
            _ => {}
        }
    }
    
    Ok(result)
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
    
    // Parse nav document using manual EventReader parsing to extract text content
    let nav_items = match parse_nav_xml(&nav_str) {
        Ok(items) => items,
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
    
    for (index, item) in nav_items.iter().enumerate() {
        if let Some(link) = item.links.first() {
            let title = link.title.clone();
            let href = link.href.clone();
            
            // Skip fragment-only links and empty hrefs
            if href.is_empty() || href.starts_with('#') {
                // Add error for empty href, but skip fragment-only links silently
                if href.is_empty() {
                    result.add_error(ValidationError::new(
                        ErrorCode::InvalidTocEntry,
                        ValidationLocation::TocEntry { index },
                    ));
                }
                continue;
            }
            
            let mut entry = crate::core::TocEntry {
                title,
                href,
                children: Vec::new(),
            };
            
            // Process subitems
            if let Some(sublist) = item.subitems.as_ref() {
                for subitem in sublist {
                    for sublink in &subitem.links {
                        if !sublink.href.is_empty() && !sublink.href.starts_with('#') {
                            entry.children.push(crate::core::TocEntry {
                                title: sublink.title.clone(),
                                href: sublink.href.clone(),
                                children: Vec::new(),
                            });
                        }
                    }
                }
            }
            
            toc_entries.push(entry);
        }
    }
    
    // Check for empty TOC
    if toc_entries.is_empty() {
        result.add_error(ValidationError::new(
            ErrorCode::EmptyToc,
            ValidationLocation::Navigation { path: nav_path.to_string() },
        ));
    }
    
    // Return error if there are any validation errors
    if !result.errors.is_empty() {
        return Err(result);
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
