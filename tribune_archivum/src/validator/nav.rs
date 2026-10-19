
use std::collections::{HashMap, HashSet};

use xmltree::{Element, XMLNode};

const XHTML_MEDIA_TYPE: &str = "application/xhtml+xml";
const NCX_MEDIA_TYPE: &str = "application/x-dtbncx+xml";
const EPUB_NAMESPACE: &str = "http://www.idpf.org/2007/ops";
const XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationError {
    MissingPackage,
    MissingManifest,
    MissingSpine,
    EmptySpine,
    MissingSpineDocument(String),
    InvalidOpfPath,
    InvalidNavPath,
    InvalidNavXml,
    InvalidNcxXml,
    MissingNavigationItem,
    ConflictingNavigationItems,
    WriteFailed,
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::MissingManifest => write!(f, "OPF has no manifest"),
            Self::MissingSpine => write!(f, "OPF has no spine"),
            Self::EmptySpine => write!(f, "OPF spine has no readable chapters"),
            Self::MissingSpineDocument(id) => {
                write!(f, "spine references missing document: {id}")
            }
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
            Self::InvalidNavPath => write!(f, "navigation path is invalid"),
            Self::InvalidNavXml => write!(f, "navigation document is invalid XML"),
            Self::InvalidNcxXml => write!(f, "NCX document is invalid XML"),
            Self::MissingNavigationItem => write!(f, "navigation manifest item is missing"),
            Self::ConflictingNavigationItems => {
                write!(f, "navigation manifest contains conflicting items")
            }
            Self::WriteFailed => write!(f, "failed to serialize navigation document"),
        }
    }
}

impl std::error::Error for NavigationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavigationReport {
    pub changed: bool,
    pub synthesized: bool,
    pub converted_ncx: bool,
    pub removed_ncx: usize,
    pub removed_spine_refs: usize,
    pub removed_entries: usize,
    pub repaired_labels: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TocEntry {
    href: String,
    label: String,
    children: Vec<TocEntry>,
}

#[derive(Debug, Clone)]
struct SpineDocument {
    id: String,
    path: String,
    label: String,
}

pub fn repair_navigation(
    opf: &mut Element,
    opf_path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
) -> Result<NavigationReport, NavigationError> {
    if opf.name != "package" {
        return Err(NavigationError::MissingPackage);
    }

    let opf_path =normalize_path(opf_path).ok_or(NavigationError::InvalidOpfPath)?;
    let opf_dir = parent_path(&opf_path);
    let manifest_index = find_manifest(opf)?;
    let spine_index = find_spine(opf)?;
    let spine = read_spine(opf, spine_index, manifest_index, &opf_dir, archive)?;

    if spine.is_empty() {
        return Err(NavigationError::EmptySpine);
    }

    let mut report = NavigationReport::default();
    let ncx_entries = read_ncx(opf, manifest_index, &opf_dir, archive)?;
    let nav_path = ensure_nav_item(
        opf,
        manifest_index,
        &opf_dir,
        &mut report,
    )?;

    let nav_path = normalize_path(&nav_path)
        .ok_or(NavigationError::InvalidNavPath)?;

    let allowed: HashSet<&str> = spine.iter().map(|item| item.path.as_str()).collect();
    let nav_bytes = archive.get(&nav_path).cloned();
    let mut document = parse_navigation(nav_bytes.as_deref())?;

    let existing_toc = extract_toc(&document);
    let mut entries = if !existing_toc.is_empty() {
        existing_toc
    } else if !ncx_entries.is_empty() {
        report.converted_ncx = true;
        ncx_entries
    } else {
        report.synthesized = true;
        Vec::new()
    };

    let original_entries = entries.clone();
    entries = normalize_entries(
        entries,
        &nav_path,
        &allowed,
        &spine,
        &mut report,
    );

    if entries.is_empty() {
        report.synthesized = true;
        entries = synthesize_toc(&spine);
    }

    let canonical_nav = build_toc_nav(entries);
    replace_navigation_elements(&mut document, canonical_nav);

    let serialized = serialize(&document)?;
    if archive.get(&nav_path) != Some(&serialized) {
        archive.insert(nav_path.clone(), serialized);
        report.changed = true;
    }

    report.changed |= report.synthesized
        || report.converted_ncx
        || report.removed_entries > 0
        || report.repaired_labels > 0
        || original_entries != extract_toc(&document);

    report.removed_ncx += remove_ncx(opf, archive);
    report.removed_spine_refs += remove_nav_from_spine(opf, &nav_path, &opf_dir);
    report.changed |= report.removed_ncx > 0 || report.removed_spine_refs > 0;

    Ok(report)
}

fn find_manifest(opf: &Element) -> Result<usize, NavigationError> {
    opf.children
        .iter()
        .position(|node| matches!(
            node,
            XMLNode::Element(element) if element.name == "manifest"
        ))
        .ok_or(NavigationError::MissingManifest)
}

fn find_spine(opf: &Element) -> Result<usize, NavigationError> {
    opf.children
        .iter()
        .position(|node| matches!(
            node,
            XMLNode::Element(element) if element.name == "spine"
        ))
        .ok_or(NavigationError::MissingSpine)
}

fn manifest_items(opf: &Element, manifest_index: usize) -> Vec<Element> {
    match &opf.children[manifest_index] {
        XMLNode::Element(manifest) => manifest
            .children
            .iter()
            .filter_map(|node| match node {
                XMLNode::Element(item) if item.name == "item" => Some(item.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn read_spine(
    opf: &Element,
    spine_index: usize,
    manifest_index: usize,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Result<Vec<SpineDocument>, NavigationError> {
    let items = manifest_items(opf, manifest_index);
    let spine = match &opf.children[spine_index] {
        XMLNode::Element(element) => element,
        _ => return Err(NavigationError::MissingSpine),
    };

    let mut documents = Vec::new();

    for node in &spine.children {
        let XMLNode::Element(itemref) = node else {
            continue;
        };

        if itemref.name != "itemref" {
            continue;
        }

        let Some(id) = itemref.attributes.get("idref") else {
            continue;
        };

        let Some(item) = items.iter().find(|item| {
            item.attributes.get("id") == Some(id)
        }) else {
            continue;
        };

        let Some(href) = item.attributes.get("href") else {
            continue;
        };

        let Some(path) = resolve_path(opf_dir, href) else {
            continue;
        };

        let Some(bytes) = archive.get(&path) else {
            return Err(NavigationError::MissingSpineDocument(id.clone()));
        };

        documents.push(SpineDocument {
            id: id.clone(),
            path,
            label: first_heading(bytes).unwrap_or_default(),
        });
    }

    Ok(documents)
}

fn ensure_nav_item(
    opf: &mut Element,
    manifest_index: usize,
    opf_dir: &str,
    report: &mut NavigationReport,
) -> Result<String, NavigationError> {
    let items = manifest_items(opf, manifest_index);
    let nav_items: Vec<Element> = items
        .iter()
        .filter(|item| item.attributes.get("id").is_some_and(|id| id == "nav"))
        .cloned()
        .collect();

    if nav_items.len() > 1 {
        return Err(NavigationError::ConflictingNavigationItems);
    }

    let mut item = nav_items.first().cloned().unwrap_or_else(|| {
        let mut item = Element::new("item");
        item.attributes.insert("id".into(), "nav".into());
        item
    });

    let href = item
        .attributes
        .get("href")
        .cloned()
        .unwrap_or_else(|| "nav.xhtml".into());

    let path = resolve_path(opf_dir, &href)
        .ok_or(NavigationError::InvalidNavPath)?;

    if item.attributes.get("properties").map(String::as_str) != Some("nav")
        || item.attributes.get("media-type").map(String::as_str)
            != Some(XHTML_MEDIA_TYPE)
        || !item.attributes.contains_key("href")
    {
        item.attributes.insert("properties".into(), "nav".into());
        item.attributes
            .insert("media-type".into(), XHTML_MEDIA_TYPE.into());
        item.attributes.insert("href".into(), href);
        report.changed = true;
    }

    if nav_items.is_empty() {
        opf.children
            .iter_mut()
            .filter_map(|node| match node {
                XMLNode::Element(element) if element.name == "manifest" => Some(element),
                _ => None,
            })
            .next()
            .ok_or(NavigationError::MissingManifest)?
            .children
            .push(XMLNode::Element(item));

        report.changed = true;
    } else {
        let manifest = match &mut opf.children[manifest_index] {
            XMLNode::Element(element) => element,
            _ => return Err(NavigationError::MissingManifest),
        };

        if let Some(node) = manifest.children.iter_mut().find(|node| {
            matches!(node, XMLNode::Element(element)
                if element.name == "item"
                    && element.attributes.get("id").is_some_and(|id| id == "nav"))
        }) {
            *node = XMLNode::Element(item);
        }
    }

    Ok(path)
}

fn read_ncx(
    opf: &Element,
    manifest_index: usize,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Result<Vec<TocEntry>, NavigationError> {
    let items = manifest_items(opf, manifest_index);

    for item in items {
        if item.attributes.get("media-type").map(String::as_str)
            != Some(NCX_MEDIA_TYPE)
        {
            continue;
        }

        let Some(href) = item.attributes.get("href") else {
            continue;
        };

        let Some(path) = resolve_path(opf_dir, href) else {
            continue;
        };

        let Some(bytes) = archive.get(&path) else {
            continue;
        };

        let ncx = Element::parse(bytes.as_slice())
            .map_err(|_| NavigationError::InvalidNcxXml)?;

        return Ok(find_element(&ncx, "navMap")
            .map(|nav_map| {
                nav_map
                    .children
                    .iter()
                    .filter_map(|node| match node {
                        XMLNode::Element(point) if point.name == "navPoint" => {
                            parse_ncx_point(point)
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default());
    }

    Ok(Vec::new())
}

fn parse_ncx_point(point: &Element) -> Option<TocEntry> {
    let content = point.children.iter().find_map(|node| match node {
        XMLNode::Element(element) if element.name == "content" => {
            element.attributes.get("src").cloned()
        }
        _ => None,
    })?;

    let label = find_element(point, "navLabel")
        .and_then(|element| find_element(element, "text"))
        .map(text_content)
        .unwrap_or_default();

    let children = point
        .children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "navPoint" => {
                parse_ncx_point(element)
            }
            _ => None,
        })
        .collect();

    Some(TocEntry {
        href: content,
        label,
        children,
    })
}

fn parse_navigation(bytes: Option<&[u8]>) -> Result<Element, NavigationError> {
    let Some(bytes) = bytes else {
        return Ok(empty_xhtml());
    };

    Element::parse(bytes).map_err(|_| NavigationError::InvalidNavXml)
}

fn empty_xhtml() -> Element {
    let mut html = Element::new("html");
    html.attributes
        .insert("xmlns".into(), XHTML_NAMESPACE.into());
    html.attributes
        .insert("xmlns:epub".into(), EPUB_NAMESPACE.into());

    let mut head = Element::new("head");
    let mut title = Element::new("title");
    title.children.push(XMLNode::Text("Contents".into()));
    head.children.push(XMLNode::Element(title));

    let mut body = Element::new("body");
    body.children.push(XMLNode::Element(Element::new("nav")));

    html.children.push(XMLNode::Element(head));
    html.children.push(XMLNode::Element(body));
    html
}

fn extract_toc(document: &Element) -> Vec<TocEntry> {
    find_elements(document, "nav")
        .into_iter()
        .find(|nav| is_toc(nav))
        .and_then(|nav| find_element(nav, "ol"))
        .map(parse_ordered_list)
        .unwrap_or_default()
}

fn is_toc(nav: &Element) -> bool {
    nav.attributes
        .get("epub:type")
        .is_some_and(|value| value.split_whitespace().any(|token| token == "toc"))
}

fn parse_ordered_list(list: &Element) -> Vec<TocEntry> {
    list.children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(item) if item.name == "li" => parse_list_item(item),
            _ => None,
        })
        .collect()
}

fn parse_list_item(item: &Element) -> Option<TocEntry> {
    let anchor = find_element(item, "a")?;
    let href = anchor.attributes.get("href")?.clone();
    let label = text_content(anchor);

    let children = find_element(item, "ol")
        .map(parse_ordered_list)
        .unwrap_or_default();

    Some(TocEntry {
        href,
        label,
        children,
    })
}

fn normalize_entries(
    entries: Vec<TocEntry>,
    nav_path: &str,
    allowed: &HashSet<&str>,
    spine: &[SpineDocument],
    report: &mut NavigationReport,
) -> Vec<TocEntry> {
    entries
        .into_iter()
        .filter_map(|mut entry| {
            let Some(path) = resolve_path(&parent_path(nav_path), &entry.href) else {
                report.removed_entries += 1;
                return None;
            };

            if !allowed.contains(path.as_str()) {
                report.removed_entries += 1;
                return None;
            }

            let Some(document) = spine.iter().find(|document| document.path == path) else {
                report.removed_entries += 1;
                return None;
            };

            let fragment = entry.href.split_once('#').map(|(_, fragment)| fragment);

            entry.href = relative_path(&parent_path(nav_path), &path);

            if let Some(fragment) = fragment {
                if !fragment.is_empty() {
                    entry.href.push('#');
                    entry.href.push_str(fragment);
                }
            }

            entry.children = normalize_entries(
                entry.children,
                nav_path,
                allowed,
                spine,
                report,
            );

            if entry.label.trim().is_empty() {
                entry.label = if document.label.trim().is_empty() {
                    format!("Chapter {}", spine.iter().position(|item| item.id == document.id).unwrap() + 1)
                } else {
                    document.label.clone()
                };
                report.repaired_labels += 1;
            }

            Some(entry)
        })
        .collect()
}

fn synthesize_toc(spine: &[SpineDocument]) -> Vec<TocEntry> {
    spine
        .iter()
        .enumerate()
        .map(|(index, document)| TocEntry {
            href: document.path.clone(),
            label: if document.label.trim().is_empty() {
                format!("Chapter {}", index + 1)
            } else {
                document.label.clone()
            },
            children: Vec::new(),
        })
        .collect()
}

fn build_toc_nav(entries: Vec<TocEntry>) -> Element {
    let mut nav = Element::new("nav");
    nav.attributes.insert("epub:type".into(), "toc".into());

    let mut list = Element::new("ol");
    list.children = entries
        .into_iter()
        .map(|entry| XMLNode::Element(build_list_item(entry)))
        .collect();

    nav.children.push(XMLNode::Element(list));
    nav
}

fn build_list_item(entry: TocEntry) -> Element {
    let mut item = Element::new("li");
    let mut anchor = Element::new("a");

    anchor.attributes.insert("href".into(), entry.href);
    anchor.children.push(XMLNode::Text(entry.label));
    item.children.push(XMLNode::Element(anchor));

    if !entry.children.is_empty() {
        let mut list = Element::new("ol");
        list.children = entry
            .children
            .into_iter()
            .map(|child| XMLNode::Element(build_list_item(child)))
            .collect();
        item.children.push(XMLNode::Element(list));
    }

    item
}

fn replace_navigation_elements(document: &mut Element, nav: Element) {
    if !replace_first_nav(document, &nav) {
        if let Some(body) = find_element_mut(document, "body") {
            body.children.push(XMLNode::Element(nav));
        } else {
            document.children.push(XMLNode::Element(nav));
        }
    }

    remove_extra_navs(document);
}

fn replace_first_nav(parent: &mut Element, nav: &Element) -> bool {
    let Some(index) = parent.children.iter().position(|node| {
        matches!(node, XMLNode::Element(element) if element.name == "nav")
    }) else {
        for node in &mut parent.children {
            if let XMLNode::Element(element) = node {
                if replace_first_nav(element, nav) {
                    return true;
                }
            }
        }
        return false;
    };

    parent.children[index] = XMLNode::Element(nav.clone());
    true
}

fn remove_extra_navs(parent: &mut Element) {
    let mut kept_nav = false;
    parent.children.retain(|node| match node {
        XMLNode::Element(element) if element.name == "nav" => {
            if kept_nav {
                false
            } else {
                kept_nav = true;
                true
            }
        }
        _ => true,
    });

    for node in &mut parent.children {
        if let XMLNode::Element(element) = node {
            remove_extra_navs(element);
        }
    }
}

fn remove_ncx(
    opf: &mut Element,
    archive: &mut HashMap<String, Vec<u8>>,
) -> usize {
    let Some(manifest_index) = opf.children.iter().position(|node| {
        matches!(node, XMLNode::Element(element) if element.name == "manifest")
    }) else {
        return 0;
    };

    let mut removed_paths = Vec::new();
    let mut removed_ids = HashSet::new();

    if let XMLNode::Element(manifest) = &mut opf.children[manifest_index] {
        manifest.children.retain(|node| {
            let XMLNode::Element(item) = node else {
                return true;
            };

            if item.name != "item"
                || item.attributes.get("media-type").map(String::as_str)
                    != Some(NCX_MEDIA_TYPE)
            {
                return true;
            }

            if let Some(id) = item.attributes.get("id") {
                removed_ids.insert(id.clone());
            }
            if let Some(href) = item.attributes.get("href") {
                removed_paths.push(href.clone());
            }
            false
        });
    }

    for node in &mut opf.children {
        let XMLNode::Element(spine) = node else {
            continue;
        };

        if spine.name != "spine" {
            continue;
        }

        spine.attributes.remove("toc");
        spine.children.retain(|node| {
            !matches!(node, XMLNode::Element(itemref)
                if itemref.name == "itemref"
                    && itemref.attributes.get("idref")
                        .is_some_and(|id| removed_ids.contains(id)))
        });
    }

    for path in removed_paths {
        if let Some(path) = normalize_path(&path) {
            archive.remove(&path);
        }
    }

    removed_ids.len()
}

fn remove_nav_from_spine(
    opf: &mut Element,
    nav_path: &str,
    opf_dir: &str,
) -> usize {
    let items = opf.children.iter().find_map(|node| match node {
        XMLNode::Element(element) if element.name == "manifest" => {
            Some(element.children.iter().filter_map(|node| match node {
                XMLNode::Element(item) if item.name == "item" => Some(item),
                _ => None,
            }).filter_map(|item| {
                Some((
                    item.attributes.get("id")?.clone(),
                    resolve_path(opf_dir, item.attributes.get("href")?)?,
                ))
            }).collect::<HashMap<_, _>>())
        }
        _ => None,
    }).unwrap_or_default();

    let Some(spine) = opf.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element) if element.name == "spine" => Some(element),
        _ => None,
    }) else {
        return 0;
    };

    let before = spine.children.len();

    spine.children.retain(|node| {
        let XMLNode::Element(itemref) = node else {
            return true;
        };

        let Some(id) = itemref.attributes.get("idref") else {
            return true;
        };

        items.get(id).is_none_or(|path| path != nav_path)
    });

    before - spine.children.len()
}

fn first_heading(bytes: &[u8]) -> Option<String> {
    let document = Element::parse(bytes).ok()?;

    for name in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        if let Some(heading) = find_element(&document, name) {
            let label = text_content(heading);
            if !label.trim().is_empty() {
                return Some(label.trim().to_owned());
            }
        }
    }

    None
}

fn find_element<'a>(element: &'a Element, name: &str) -> Option<&'a Element> {
    if element.name == name {
        return Some(element);
    }

    element.children.iter().find_map(|node| match node {
        XMLNode::Element(child) => find_element(child, name),
        _ => None,
    })
}

fn find_element_mut<'a>(
    element: &'a mut Element,
    name: &str,
) -> Option<&'a mut Element> {
    if element.name == name {
        return Some(element);
    }

    element.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(child) => find_element_mut(child, name),
        _ => None,
    })
}

fn find_elements<'a>(element: &'a Element, name: &str) -> Vec<&'a Element> {
    let mut found = Vec::new();

    if element.name == name {
        found.push(element);
    }

    for node in &element.children {
        if let XMLNode::Element(child) = node {
            found.extend(find_elements(child, name));
        }
    }

    found
}

fn text_content(element: &Element) -> String {
    element
        .children
        .iter()
        .map(|node| match node {
            XMLNode::Text(text) | XMLNode::CData(text) => text.clone(),
            XMLNode::Element(child) => text_content(child),
            _ => String::new(),
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn serialize(element: &Element) -> Result<Vec<u8>, NavigationError> {
    let mut bytes = Vec::new();

    element
        .write(&mut bytes)
        .map_err(|_| NavigationError::WriteFailed)?;

    Ok(bytes)
}

fn normalize_path(path: &str) -> Option<String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('%')
        || path.contains('\0')
        || path.contains(':')
    {
        return None;
    }

    let mut parts = Vec::new();

    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            _ => parts.push(part),
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default()
}

fn resolve_path(base: &str, href: &str) -> Option<String> {
    if href.starts_with("//")
        || href.starts_with('/')
        || href.contains('\\')
        || href.contains('\0')
        || href.contains(':')
    {
        return None;
    }

    let path = href.split(['?', '#']).next()?;

    if path.is_empty() {
        return None;
    }

    let mut parts: Vec<&str> = base
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();

    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

fn relative_path(base: &str, target: &str) -> String {
    let base_parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    let target_parts: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();

    let common = base_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(left, right)| left == right)
        .count();

    let mut parts = vec![".."; base_parts.len() - common];
    parts.extend_from_slice(&target_parts[common..]);

    parts.join("/")
}

#[cfg(test)]
#[path = "nav_tests.rs"]
mod nav_tests;