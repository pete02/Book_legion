use std::collections::{HashMap, HashSet, VecDeque};

use xmltree::{Element, XMLNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    MissingPackage,
    MissingManifest,
    InvalidOpfPath,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::MissingManifest => write!(f, "OPF has no manifest"),
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
        }
    }
}

impl std::error::Error for ManifestError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManifestReport {
    pub retained_paths: HashSet<String>,
    pub removed_paths: HashSet<String>,
    pub changed: bool,
}

#[derive(Debug)]
struct Item {
    element: Element,
    id: String,
    path: String,
    media_type: String,
}

pub fn repair_manifest(
    opf: &mut Element,
    opf_path: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Result<ManifestReport, ManifestError> {
    validate_package_root(opf)?;
    let opf_path = normalize_archive_path(opf_path).ok_or(ManifestError::InvalidOpfPath)?;
    let opf_dir = parent_path(&opf_path);
    let (manifest_indices, first_manifest_index) = locate_manifests(opf)?;

    let source_items = manifest_items_from(opf, &manifest_indices);
    let (items, mut used_ids, mut next_id, mut changed) =
        normalize_items(source_items, &opf_dir, archive);
    changed |= manifest_indices.len() > 1;

    let retained_paths = determine_retained_paths(opf, &opf_path, &items, archive);
    let (items, removed_unreachable_items) = filter_items_to_retained_paths(items, &retained_paths);
    changed |= removed_unreachable_items;
    let (items, added_missing_items) = add_missing_manifest_items(
        items,
        &retained_paths,
        &opf_path,
        &opf_dir,
        archive,
        &mut used_ids,
        &mut next_id,
    );
    changed |= added_missing_items;

    let merged_manifest = build_manifest(items);
    replace_manifests(
        opf,
        &manifest_indices,
        first_manifest_index,
        merged_manifest,
    );
    Ok(build_manifest_report(retained_paths, archive, changed))
}

fn validate_package_root(opf: &Element) -> Result<(), ManifestError> {
    if opf.name == "package" {
        Ok(())
    } else {
        Err(ManifestError::MissingPackage)
    }
}

fn locate_manifests(opf: &Element) -> Result<(Vec<usize>, usize), ManifestError> {
    let indices = child_element_indices(opf, "manifest");
    let first: usize = indices
        .first()
        .copied()
        .ok_or(ManifestError::MissingManifest)?;
    Ok((indices, first))
}

fn determine_retained_paths(
    opf: &Element,
    opf_path: &str,
    items: &[Item],
    archive: &HashMap<String, Vec<u8>>,
) -> HashSet<String> {
    let spine_ids = spine_references(opf);
    let seed_documents = items
        .iter()
        .filter(|item| {
            item.media_type == "application/xhtml+xml"
                && (has_property(&item.element, "nav") || spine_ids.contains(&item.id))
        })
        .map(|item| item.path.clone());

    let mut retained = special_archive_paths(archive);
    retained.insert(opf_path.to_string());
    retained.extend(discover_resources(seed_documents, archive));
    retain_supported_resources(&mut retained, opf_path, archive);
    retained
}

fn retain_supported_resources(
    retained: &mut HashSet<String>,
    opf_path: &str,
    archive: &HashMap<String, Vec<u8>>,
) {
    retained.retain(|path| {
        is_required_archive_path(path, opf_path) || supported_media_type(path, archive).is_some()
    });
}

fn is_required_archive_path(path: &str, opf_path: &str) -> bool {
    path == opf_path || path == "mimetype" || path.starts_with("META-INF/")
}

fn supported_media_type(path: &str, archive: &HashMap<String, Vec<u8>>) -> Option<String> {
    let bytes = archive.get(path)?;
    sniff_media_type(path, bytes).filter(|kind| is_allowed_media_type(kind))
}

fn filter_items_to_retained_paths(
    items: Vec<Item>,
    retained_paths: &HashSet<String>,
) -> (Vec<Item>, bool) {
    let original_len = items.len();
    let retained_items = items
        .into_iter()
        .filter(|item| retained_paths.contains(&item.path))
        .collect::<Vec<_>>();
    let changed = retained_items.len() != original_len;
    (retained_items, changed)
}

fn add_missing_manifest_items(
    mut items: Vec<Item>,
    retained_paths: &HashSet<String>,
    opf_path: &str,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
    used_ids: &mut HashSet<String>,
    next_id: &mut usize,
) -> (Vec<Item>, bool) {
    let represented_paths: HashSet<String> = items.iter().map(|item| item.path.clone()).collect();
    let mut changed = false;

    for path in retained_paths {
        if represented_paths.contains(path) || is_required_archive_path(path, opf_path) {
            continue;
        }
        let Some(media_type) = supported_media_type(path, archive) else {
            continue;
        };

        let id = generate_id(used_ids, next_id);
        items.push(new_manifest_item(id, path, opf_dir, media_type));
        changed = true;
    }

    (items, changed)
}

fn new_manifest_item(id: String, path: &str, opf_dir: &str, media_type: String) -> Item {
    let mut element = Element::new("item");
    element.attributes.insert("id".into(), id.clone());
    element
        .attributes
        .insert("href".into(), relative_href(opf_dir, path));
    element
        .attributes
        .insert("media-type".into(), media_type.clone());
    Item {
        element,
        id,
        path: path.to_string(),
        media_type,
    }
}

fn build_manifest(items: Vec<Item>) -> Element {
    let mut manifest = Element::new("manifest");
    manifest
        .children
        .extend(items.into_iter().map(|item| XMLNode::Element(item.element)));
    manifest
}

fn build_manifest_report(
    retained_paths: HashSet<String>,
    archive: &HashMap<String, Vec<u8>>,
    changed: bool,
) -> ManifestReport {
    let all_paths: HashSet<String> = archive.keys().cloned().collect();
    let removed_paths = all_paths.difference(&retained_paths).cloned().collect();
    ManifestReport {
        retained_paths,
        removed_paths,
        changed,
    }
}

fn child_element_indices(parent: &Element, name: &str) -> Vec<usize> {
    parent
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            matches!(node, XMLNode::Element(element) if element.name == name).then_some(index)
        })
        .collect()
}

fn manifest_items_from(opf: &Element, indices: &[usize]) -> Vec<Element> {
    indices
        .iter()
        .filter_map(|&index| match &opf.children[index] {
            XMLNode::Element(manifest) => Some(manifest),
            _ => None,
        })
        .flat_map(|manifest| manifest.children.iter())
        .filter_map(|node| match node {
            XMLNode::Element(item) if item.name == "item" => Some(item.clone()),
            _ => None,
        })
        .collect()
}

fn replace_manifests(opf: &mut Element, indices: &[usize], first: usize, merged: Element) {
    let indices: HashSet<usize> = indices.iter().copied().collect();
    let mut children = Vec::with_capacity(opf.children.len());
    for (index, child) in std::mem::take(&mut opf.children).into_iter().enumerate() {
        if indices.contains(&index) {
            if index == first {
                children.push(XMLNode::Element(merged.clone()));
            }
        } else {
            children.push(child);
        }
    }
    opf.children = children;
}

fn normalize_items(
    source_items: Vec<Element>,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> (Vec<Item>, HashSet<String>, usize, bool) {
    let mut items = Vec::new();
    let mut used_ids = HashSet::new();
    let mut next_id = 1;
    let mut changed = false;

    for mut element in source_items {
        let Some((path, media_type)) = resolve_supported_item(&element, opf_dir, archive) else {
            changed = true;
            continue;
        };

        let (id, id_changed) = keep_or_generate_id(&element, &mut used_ids, &mut next_id);
        let href = relative_href(opf_dir, &path);
        let attributes_changed = set_item_attributes(&mut element, &id, href, &media_type);

        changed |= id_changed || attributes_changed;
        items.push(Item {
            element,
            id,
            path,
            media_type,
        });
    }
    (items, used_ids, next_id, changed)
}

fn resolve_supported_item(
    element: &Element,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<(String, String)> {
    let href = element
        .attributes
        .get("href")
        .map(String::as_str)
        .unwrap_or("")
        .trim();
    let path = resolve_href(opf_dir, href, archive)?;
    let media_type = supported_media_type(&path, archive)?;
    Some((path, media_type))
}

fn keep_or_generate_id(
    element: &Element,
    used_ids: &mut HashSet<String>,
    next_id: &mut usize,
) -> (String, bool) {
    let old_id = element.attributes.get("id").cloned().unwrap_or_default();
    if old_id.trim().is_empty() || used_ids.contains(&old_id) {
        (generate_id(used_ids, next_id), true)
    } else {
        used_ids.insert(old_id.clone());
        (old_id, false)
    }
}

/// Returns true if href or media-type differed from what was there.
fn set_item_attributes(element: &mut Element, id: &str, href: String, media_type: &str) -> bool {
    let changed = element.attributes.get("href") != Some(&href)
        || element.attributes.get("media-type").map(String::as_str) != Some(media_type);
    element.attributes.insert("id".into(), id.to_owned());
    element.attributes.insert("href".into(), href);
    element
        .attributes
        .insert("media-type".into(), media_type.to_owned());
    changed
}

fn generate_id(used_ids: &mut HashSet<String>, next_id: &mut usize) -> String {
    loop {
        let candidate = format!("generated-item-{}", *next_id);
        *next_id += 1;
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn special_archive_paths(archive: &HashMap<String, Vec<u8>>) -> HashSet<String> {
    archive
        .keys()
        .filter(|path| *path == "mimetype" || path.starts_with("META-INF/"))
        .cloned()
        .collect()
}

fn discover_resources<I>(seeds: I, archive: &HashMap<String, Vec<u8>>) -> HashSet<String>
where
    I: IntoIterator<Item = String>,
{
    let mut retained = HashSet::new();
    let mut queue: VecDeque<String> = seeds.into_iter().collect();
    let mut visited = HashSet::new();

    while let Some(path) = queue.pop_front() {
        if !visited.insert(path.clone()) {
            continue;
        }
        retained.insert(path.clone());

        for target in referenced_paths(&path, archive) {
            retained.insert(target.clone());
            queue.push_back(target);
        }
    }
    retained
}

fn referenced_paths(path: &str, archive: &HashMap<String, Vec<u8>>) -> Vec<String> {
    let Some(bytes) = archive.get(path) else {
        return Vec::new();
    };
    let references = match sniff_media_type(path, bytes).as_deref() {
        Some("application/xhtml+xml") => extract_xhtml_references(bytes),
        Some("text/css") => extract_css_references(bytes),
        _ => return Vec::new(),
    };
    let base_dir = parent_path(path);
    references
        .iter()
        .filter_map(|href| resolve_resource_href(&base_dir, href, archive))
        .collect()
}

fn has_property(item: &Element, property: &str) -> bool {
    item.attributes
        .get("properties")
        .is_some_and(|value| value.split_whitespace().any(|part| part == property))
}

fn spine_references(opf: &Element) -> HashSet<String> {
    opf.children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "spine" => Some(element),
            _ => None,
        })
        .flat_map(|spine| spine.children.iter())
        .filter_map(|node| match node {
            XMLNode::Element(itemref) if itemref.name == "itemref" => {
                itemref.attributes.get("idref").cloned()
            }
            _ => None,
        })
        .collect()
}

fn is_allowed_media_type(media_type: &str) -> bool {
    return matches!(
        media_type,
        "application/xhtml+xml"
            | "text/css"
            | "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/svg+xml"
            | "font/woff2"
            | "font/woff"
            | "font/ttf"
            | "font/otf"
    );
}

/// Content decides first (magic bytes, then text markers). The extension is only a
/// fallback, and only for text: binary data with an unknown signature is rejected
/// whatever its extension says.
fn sniff_media_type(path: &str, bytes: &[u8]) -> Option<String> {
    if let Some(kind) = sniff_magic_bytes(bytes) {
        return Some(kind.to_string());
    }
    let text = std::str::from_utf8(bytes).ok()?;
    sniff_text(text)
        .or_else(|| media_type_from_extension(path))
        .map(String::from)
}

fn sniff_magic_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\xFF\xD8\xFF") {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.starts_with(b"wOF2") {
        return Some("font/woff2");
    }
    if bytes.starts_with(b"wOFF") {
        return Some("font/woff");
    }
    if bytes.starts_with(b"\x00\x01\x00\x00") {
        return Some("font/ttf");
    }
    if bytes.starts_with(b"OTTO") {
        return Some("font/otf");
    }
    None
}

fn sniff_text(text: &str) -> Option<&'static str> {
    let lower = text
        .trim_start_matches('\u{feff}')
        .trim_start()
        .to_ascii_lowercase();
    if lower.contains("<svg") {
        Some("image/svg+xml")
    } else if lower.contains("<html") || lower.contains(":html") {
        Some("application/xhtml+xml")
    } else if looks_like_css(&lower) {
        Some("text/css")
    } else {
        None
    }
}

fn looks_like_css(lower: &str) -> bool {
    lower.starts_with("@charset")
        || lower.starts_with("@import")
        || lower.contains("url(")
        || (lower.contains('{') && (lower.contains("color:") || lower.contains("font-family:")))
}

fn media_type_from_extension(path: &str) -> Option<&'static str> {
    let extension = path
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase());
    match extension.as_deref() {
        Some("xhtml" | "html" | "htm") => Some("application/xhtml+xml"),
        Some("css") => Some("text/css"),
        Some("jpg" | "jpeg") => Some("image/jpeg"),
        Some("png") => Some("image/png"),
        Some("gif") => Some("image/gif"),
        Some("webp") => Some("image/webp"),
        Some("svg") => Some("image/svg+xml"),
        Some("woff2") => Some("font/woff2"),
        Some("woff") => Some("font/woff"),
        Some("ttf") => Some("font/ttf"),
        Some("otf") => Some("font/otf"),
        _ => None,
    }
}

/// For manifest hrefs, which must name a file exactly: any fragment or query is rejected.
fn resolve_href(base_dir: &str, href: &str, archive: &HashMap<String, Vec<u8>>) -> Option<String> {
    if href.trim().contains('#') || href.trim().contains('?') {
        return None;
    }
    resolve_path(base_dir, href, archive)
}

/// For links inside content, where `#fragment` and `?query` are legitimate and are stripped.
fn resolve_resource_href(
    base_dir: &str,
    href: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<String> {
    let href = href.trim();
    let href = href.split('#').next()?.split('?').next()?;
    resolve_path(base_dir, href, archive)
}

fn resolve_path(base_dir: &str, href: &str, archive: &HashMap<String, Vec<u8>>) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('/') || href.starts_with("//") || has_uri_scheme(href) {
        return None;
    }
    let decoded = percent_decode(href)?;
    if decoded.contains('\\') || decoded.contains('\0') {
        return None;
    }

    let mut components: Vec<&str> = if base_dir.is_empty() {
        Vec::new()
    } else {
        base_dir.split('/').collect()
    };
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            _ => components.push(part),
        }
    }
    if components.is_empty() {
        return None;
    }
    find_archive_entry(&components.join("/"), archive)
}

fn find_archive_entry(candidate: &str, archive: &HashMap<String, Vec<u8>>) -> Option<String> {
    if archive.contains_key(candidate) {
        return Some(candidate.to_string());
    }
    let mut matches = archive
        .keys()
        .filter(|path| path.eq_ignore_ascii_case(candidate));
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first.clone())
}

fn relative_href(base_dir: &str, target: &str) -> String {
    if base_dir.is_empty() {
        return target.to_string();
    }
    let base: Vec<&str> = base_dir.split('/').collect();
    let target: Vec<&str> = target.split('/').collect();
    let common = base
        .iter()
        .zip(target.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts = vec![".."; base.len() - common];
    parts.extend(target[common..].iter().copied());
    parts.join("/")
}

fn normalize_archive_path(path: &str) -> Option<String> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
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

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn has_uri_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            ch.is_ascii_alphabetic()
                || (index > 0 && (ch.is_ascii_digit() || matches!(ch, '+' | '.' | '-')))
        })
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push((hex_value(high)? << 4) | hex_value(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn extract_xhtml_references(bytes: &[u8]) -> Vec<String> {
    let Ok(root) = Element::parse(bytes) else {
        return Vec::new();
    };
    let mut references = Vec::new();
    collect_xml_references(&root, &mut references);
    references
}

fn collect_xml_references(element: &Element, references: &mut Vec<String>) {
    for (name, value) in &element.attributes {
        let is_resource =
            matches!(name.as_str(), "src" | "poster" | "data" | "href") || name.ends_with(":href");
        if is_resource {
            references.push(value.clone());
        }
    }
    for child in &element.children {
        if let XMLNode::Element(element) = child {
            collect_xml_references(element, references);
        }
    }
}

fn extract_css_references(bytes: &[u8]) -> Vec<String> {
    let Ok(css) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    let mut references = css_url_references(css);
    references.extend(css_import_references(css));
    references
}

fn css_url_references(css: &str) -> Vec<String> {
    let lower = css.to_ascii_lowercase();
    let mut references = Vec::new();
    let mut offset = 0;
    while let Some(relative) = lower[offset..].find("url(") {
        let start = offset + relative + 4;
        let Some(end_relative) = css[start..].find(')') else {
            break;
        };
        let end = start + end_relative;
        let value = css[start..end].trim().trim_matches(['\'', '"']).trim();
        if !value.is_empty() {
            references.push(value.to_string());
        }
        offset = end + 1;
        if offset >= css.len() {
            break;
        }
    }
    references
}

fn css_import_references(css: &str) -> Vec<String> {
    let lower = css.to_ascii_lowercase();
    let mut references = Vec::new();
    let mut offset = 0;
    while let Some(relative) = lower[offset..].find("@import") {
        let start = offset + relative + 7;
        let tail = css[start..].trim_start();
        if let Some(quote) = tail.chars().next().filter(|ch| *ch == '\'' || *ch == '"') {
            if let Some(end) = tail[1..].find(quote) {
                references.push(tail[1..end + 1].to_string());
            }
        }
        offset = start;
        if offset >= css.len() {
            break;
        }
    }
    references
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
