use crate::archive::read_entry;
use crate::core::{ErrorCode, NavInfo, TocEntry, ValidationLocation, ValidationResult};
use quick_xml::de::from_str;
use std::io::Read;

/// EPUB3 navigation document root element
#[derive(Debug, Deserialize)]
struct NavDocument {
    #[serde(rename = "nav")]
    nav_element: NavElement,
}

#[derive(Debug, Deserialize)]
struct NavElement {
    #[serde(rename = "@epub:type")]
    epub_type: Option<String>,
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
    #[serde(attribute)]
    href: String,
    #[serde(rename = "#text", default)]
    title: Option<String>,
}

/// Validates the EPUB3 navigation document
pub fn validate_epub3_nav(
    archive: &mut ZipArchive<std::fs::File>,
    nav_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Read nav document
    let nav_data = match read_entry(archive, nav_path) {
        Ok(data) => data,
        Err((mut res, _)) => return Err(res),
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
                    
                    let mut entry = TocEntry {
                        title,
                        href,
                        children: Vec::new(),
                    };
                    
                    // Process subitems
                    if let Some(sublist) = item.subitems.as_ref() {
                        for subitem in &sublist.list_items {
                            if let Some(link) = subitem.links.first() {
                                if !link.href.is_empty() && !link.href.starts_with('#') {
                                    entry.children.push(TocEntry {
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
