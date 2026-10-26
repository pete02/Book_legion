use std::collections::{HashMap, HashSet};

use xmltree::{Element, XMLNode};

const XHTML_MEDIA_TYPE: &str = "application/xhtml+xml";
const SVG_MEDIA_TYPE: &str = "image/svg+xml";
const CSS_MEDIA_TYPE: &str = "text/css";

const ALLOWED_DATA_URI_PREFIXES: [&str; 5] = [
    "data:image/png",
    "data:image/jpeg",
    "data:image/gif",
    "data:image/webp",
    "data:image/svg+xml",
];
const EXTERNAL_LINK_SCHEMES: [&str; 3] = ["http", "https", "mailto"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    MissingPackage,
    MissingManifest,
    MissingSpine,
    InvalidOpfPath,
    MalformedDocument(String),
    WriteFailed,
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::MissingManifest => write!(f, "OPF has no manifest"),
            Self::MissingSpine => write!(f, "OPF has no spine"),
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
            Self::MalformedDocument(path) => write!(f, "document is not well-formed: {path}"),
            Self::WriteFailed => write!(f, "failed to serialize document"),
        }
    }
}

impl std::error::Error for ReferenceError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceReport {
    pub changed: bool,
    pub rewritten_references: usize,
    pub removed_resource_references: usize,
    pub removed_link_targets: usize,
    pub removed_srcsets: usize,
}

impl ReferenceReport {
    fn edits(&self) -> usize {
        self.rewritten_references
            + self.removed_resource_references
            + self.removed_link_targets
            + self.removed_srcsets
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expected {
    Image,
    Stylesheet,
    SvgImage,
}

impl Expected {
    fn accepts(self, media_type: &str) -> bool {
        match self {
            Self::Image => media_type.starts_with("image/"),
            Self::Stylesheet => media_type == CSS_MEDIA_TYPE,
            Self::SvgImage => media_type == SVG_MEDIA_TYPE,
        }
    }
}

enum Fate {
    Keep,
    Drop,
    ReplaceWithText(String),
}

struct Package {
    media_types: HashMap<String, String>,
    spine_paths: HashSet<String>,
}

struct Context<'a> {
    document_dir: &'a str,
    package: &'a Package,
}

/// Makes every reference in the XHTML and SVG documents resolve to a manifest item (R1-R6).
///
/// Kept references are rewritten to one canonical form: the shortest relative
/// path from the referencing document, percent-encoded, with the fragment kept
/// and any query dropped. The reader can then decode it and look the path up
/// by exact string match. A resource reference that cannot be kept is removed
/// together with its element, except that an `<img>` leaves its alt text behind.
/// A link that cannot be kept loses its `href` and keeps its text.
///
/// Links must target a spine item, so a link to a manifest image or stylesheet
/// is removed too. CSS `url()` references are not touched here.
pub fn repair_references(
    opf: &Element,
    opf_path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
) -> Result<ReferenceReport, ReferenceError> {
    if opf.name != "package" {
        return Err(ReferenceError::MissingPackage);
    }

    let opf_path = normalize_path(opf_path).ok_or(ReferenceError::InvalidOpfPath)?;
    let package = read_package(opf, &parent_path(&opf_path))?;
    let mut report = ReferenceReport::default();

    let mut documents: Vec<&String> = package
        .media_types
        .iter()
        .filter(|(path, media_type)| {
            matches!(media_type.as_str(), XHTML_MEDIA_TYPE | SVG_MEDIA_TYPE)
                && archive.contains_key(*path)
        })
        .map(|(path, _)| path)
        .collect();
    documents.sort();

    for path in documents {
        repair_document(path, archive, &package, &mut report)?;
    }

    Ok(report)
}

fn read_package(opf: &Element, opf_dir: &str) -> Result<Package, ReferenceError> {
    let manifest = child_element(opf, "manifest").ok_or(ReferenceError::MissingManifest)?;
    let spine = child_element(opf, "spine").ok_or(ReferenceError::MissingSpine)?;
    let mut media_types = HashMap::new();
    let mut paths_by_id = HashMap::new();

    for item in child_elements(manifest, "item") {
        let (Some(href), Some(media_type)) =
            (item.attributes.get("href"), item.attributes.get("media-type"))
        else {
            continue;
        };
        let Some(path) = resolve_path(opf_dir, href) else {
            continue;
        };
        if let Some(id) = item.attributes.get("id") {
            paths_by_id.insert(id.clone(), path.clone());
        }
        media_types.insert(path, media_type.clone());
    }

    let spine_paths = child_elements(spine, "itemref")
        .filter_map(|itemref| itemref.attributes.get("idref"))
        .filter_map(|idref| paths_by_id.get(idref).cloned())
        .collect();

    Ok(Package {
        media_types,
        spine_paths,
    })
}

fn repair_document(
    path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
    package: &Package,
    report: &mut ReferenceReport,
) -> Result<(), ReferenceError> {
    let Some(bytes) = archive.get(path) else {
        return Ok(());
    };
    let mut root = Element::parse(bytes.as_slice())
        .map_err(|_| ReferenceError::MalformedDocument(path.to_owned()))?;
    let document_dir = parent_path(path);
    let context = Context {
        document_dir: &document_dir,
        package,
    };
    let edits_before = report.edits();

    let children = std::mem::take(&mut root.children);
    root.children = rewrite_nodes(children, &context, report);

    if report.edits() == edits_before {
        return Ok(());
    }

    let mut rewritten = Vec::new();
    root.write(&mut rewritten)
        .map_err(|_| ReferenceError::WriteFailed)?;
    archive.insert(path.to_owned(), rewritten);
    report.changed = true;
    Ok(())
}

fn rewrite_nodes(
    nodes: Vec<XMLNode>,
    context: &Context,
    report: &mut ReferenceReport,
) -> Vec<XMLNode> {
    let mut rewritten = Vec::new();

    for node in nodes {
        let XMLNode::Element(mut element) = node else {
            rewritten.push(node);
            continue;
        };

        match rewrite_element(&mut element, context, report) {
            Fate::Keep => {
                let children = std::mem::take(&mut element.children);
                element.children = rewrite_nodes(children, context, report);
                rewritten.push(XMLNode::Element(element));
            }
            Fate::Drop => {}
            Fate::ReplaceWithText(text) => rewritten.push(XMLNode::Text(text)),
        }
    }

    rewritten
}

fn rewrite_element(
    element: &mut Element,
    context: &Context,
    report: &mut ReferenceReport,
) -> Fate {
    let name = local_name(&element.name).to_ascii_lowercase();

    match name.as_str() {
        "img" | "source" => {
            report.removed_srcsets += usize::from(element.attributes.remove("srcset").is_some());
            rewrite_resource(element, "src", Expected::Image, context, report)
        }
        "link" => {
            report.removed_srcsets +=
                usize::from(element.attributes.remove("imagesrcset").is_some());
            rewrite_resource(element, "href", Expected::Stylesheet, context, report)
        }
        "image" => rewrite_resource(element, "href", Expected::Image, context, report),
        "use" => rewrite_resource(element, "href", Expected::SvgImage, context, report),
        "a" | "area" => {
            rewrite_link(element, context, report);
            Fate::Keep
        }
        _ => Fate::Keep,
    }
}

fn rewrite_resource(
    element: &mut Element,
    attribute: &str,
    expected: Expected,
    context: &Context,
    report: &mut ReferenceReport,
) -> Fate {
    let Some(key) = attribute_key(element, attribute) else {
        return strip_resource(element, report);
    };
    let value = element.attributes[&key].clone();

    match canonical_resource(&value, expected, context) {
        Some(canonical) => {
            if canonical != value {
                element.attributes.insert(key, canonical);
                report.rewritten_references += 1;
            }
            Fate::Keep
        }
        None => strip_resource(element, report),
    }
}

fn strip_resource(element: &Element, report: &mut ReferenceReport) -> Fate {
    report.removed_resource_references += 1;

    let is_image = local_name(&element.name).eq_ignore_ascii_case("img");
    let alt = element
        .attributes
        .get("alt")
        .map(|alt| alt.trim())
        .filter(|alt| is_image && !alt.is_empty());

    alt.map_or(Fate::Drop, |text| Fate::ReplaceWithText(text.to_owned()))
}

fn rewrite_link(element: &mut Element, context: &Context, report: &mut ReferenceReport) {
    let Some(key) = attribute_key(element, "href") else {
        return;
    };
    let value = element.attributes[&key].clone();

    match canonical_link(&value, context) {
        Some(canonical) => {
            if canonical != value {
                element.attributes.insert(key, canonical);
                report.rewritten_references += 1;
            }
        }
        None => {
            element.attributes.remove(&key);
            report.removed_link_targets += 1;
        }
    }
}

fn canonical_resource(value: &str, expected: Expected, context: &Context) -> Option<String> {
    let value = value.trim();

    if value.starts_with('#') || (expected == Expected::Image && is_allowed_data_uri(value)) {
        return Some(value.to_owned());
    }

    let (path, suffix) = resolve_target(value, context)?;
    let media_type = context.package.media_types.get(&path)?;

    expected
        .accepts(media_type)
        .then(|| canonical_href(context.document_dir, &path, &suffix))
}

fn canonical_link(value: &str, context: &Context) -> Option<String> {
    let value = value.trim();

    if value.starts_with('#') {
        return Some(value.to_owned());
    }

    if let Some(scheme) = uri_scheme(value) {
        let is_external_link = EXTERNAL_LINK_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str());
        return is_external_link.then(|| value.to_owned());
    }

    let (path, suffix) = resolve_target(value, context)?;

    context
        .package
        .spine_paths
        .contains(&path)
        .then(|| canonical_href(context.document_dir, &path, &suffix))
}

fn is_allowed_data_uri(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ALLOWED_DATA_URI_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

/// Returns the manifest path a local href points at, and its `#fragment` (or an empty string).
fn resolve_target(href: &str, context: &Context) -> Option<(String, String)> {
    if href.is_empty() || href.starts_with('/') || uri_scheme(href).is_some() {
        return None;
    }

    let (location, fragment) = match href.split_once('#') {
        Some((location, fragment)) => (location, format!("#{fragment}")),
        None => (href, String::new()),
    };
    let path = lookup_manifest_path(location.split('?').next()?, context)?;

    Some((path, fragment))
}

fn lookup_manifest_path(location: &str, context: &Context) -> Option<String> {
    let decoded = percent_decode(location)?;
    if decoded.is_empty() || decoded.contains('\\') || decoded.contains('\0') {
        return None;
    }

    let mut parts: Vec<&str> = context
        .document_dir
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }

    find_manifest_path(&parts.join("/"), context.package)
}

fn find_manifest_path(candidate: &str, package: &Package) -> Option<String> {
    if package.media_types.contains_key(candidate) {
        return Some(candidate.to_owned());
    }

    let mut matches = package
        .media_types
        .keys()
        .filter(|path| path.eq_ignore_ascii_case(candidate));
    let first = matches.next()?;

    matches.next().is_none().then(|| first.clone())
}

fn canonical_href(document_dir: &str, path: &str, suffix: &str) -> String {
    format!("{}{suffix}", encode_path(&relative_href(document_dir, path)))
}

fn encode_path(path: &str) -> String {
    path.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn attribute_key(element: &Element, local: &str) -> Option<String> {
    element
        .attributes
        .keys()
        .find(|key| local_name(key) == local)
        .cloned()
}

fn uri_scheme(value: &str) -> Option<&str> {
    let (scheme, _) = value.split_once(':')?;
    let is_scheme = !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            ch.is_ascii_alphabetic()
                || (index > 0 && (ch.is_ascii_digit() || matches!(ch, '+' | '.' | '-')))
        });

    is_scheme.then_some(scheme)
}

fn child_elements<'a>(
    parent: &'a Element,
    name: &'a str,
) -> impl Iterator<Item = &'a Element> {
    parent.children.iter().filter_map(move |node| match node {
        XMLNode::Element(element) if local_name(&element.name) == name => Some(element),
        _ => None,
    })
}

fn child_element<'a>(parent: &'a Element, name: &'a str) -> Option<&'a Element> {
    child_elements(parent, name).next()
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_value(*bytes.get(index + 1)?)?;
            let low = hex_value(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
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

fn normalize_path(path: &str) -> Option<String> {
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

    (!parts.is_empty()).then(|| parts.join("/"))
}

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default()
}

fn resolve_path(base: &str, href: &str) -> Option<String> {
    if href.starts_with('/') || href.contains('\\') || href.contains('\0') || href.contains(':') {
        return None;
    }

    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for part in href.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }

    (!parts.is_empty()).then(|| parts.join("/"))
}

fn relative_href(base_dir: &str, target: &str) -> String {
    let base: Vec<&str> = base_dir.split('/').filter(|part| !part.is_empty()).collect();
    let target: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    let common = base
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();

    let mut parts = vec![".."; base.len() - common];
    parts.extend_from_slice(&target[common..]);
    parts.join("/")
}

#[cfg(test)]
#[path = "references_tests.rs"]
mod references_tests;