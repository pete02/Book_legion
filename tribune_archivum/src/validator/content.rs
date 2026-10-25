use std::collections::HashMap;

use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use html5ever::{
    Attribute, ParseOpts, QualName, parse_document, tendril::TendrilSink,
    tree_builder::TreeBuilderOpts,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use xmltree::{Element, XMLNode};

const XHTML_MEDIA_TYPE: &str = "application/xhtml+xml";
const SVG_MEDIA_TYPE: &str = "image/svg+xml";
const XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const EPUB_NAMESPACE: &str = "http://www.idpf.org/2007/ops";
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

const DECLARED_PREFIXES: [(&str, &str); 2] = [("epub", EPUB_NAMESPACE), ("xlink", XLINK_NAMESPACE)];
const URL_ATTRIBUTES: [&str; 9] = [
    "href",
    "src",
    "action",
    "formaction",
    "data",
    "poster",
    "background",
    "cite",
    "longdesc",
];
const FORBIDDEN_CSS_TOKENS: [&str; 4] = ["@import", "expression(", "behavior:", "-moz-binding"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    MissingPackage,
    MissingManifest,
    InvalidOpfPath,
    WriteFailed,
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::MissingManifest => write!(f, "OPF has no manifest"),
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
            Self::WriteFailed => write!(f, "failed to serialize content document"),
        }
    }
}

impl std::error::Error for ContentError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentReport {
    pub changed: bool,
    pub transcoded_documents: Vec<String>,
    pub reparsed_documents: Vec<String>,
    pub empty_documents: Vec<String>,
    pub removed_images: Vec<String>,
    pub removed_elements: usize,
    pub removed_attributes: usize,
    pub unwrapped_elements: usize,
    pub resolved_switches: usize,
}

impl ContentReport {
    fn add_repairs(&mut self, repairs: &Repairs) {
        self.removed_elements += repairs.removed_elements;
        self.removed_attributes += repairs.removed_attributes;
        self.unwrapped_elements += repairs.unwrapped_elements;
        self.resolved_switches += repairs.resolved_switches;
    }
}

#[derive(Debug, Default)]
struct Repairs {
    removed_elements: usize,
    removed_attributes: usize,
    unwrapped_elements: usize,
    resolved_switches: usize,
}

impl Repairs {
    fn any(&self) -> bool {
        self.removed_elements
            + self.removed_attributes
            + self.unwrapped_elements
            + self.resolved_switches
            > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DocumentKind {
    Xhtml,
    Svg,
}

enum Fate {
    Keep,
    Remove,
    Unwrap,
    UseDefaultBranch,
}

enum SvgFate {
    Unchanged,
    Rewritten(Vec<u8>),
    Unusable,
}

struct ParsedDocument {
    root: Element,
    transcoded: bool,
    reparsed_as_html: bool,
}

struct XhtmlOutcome {
    rewritten: Option<Vec<u8>>,
    repairs: Repairs,
    transcoded: bool,
    reparsed_as_html: bool,
    empty: bool,
}

/// Sanitizes every XHTML manifest item (X1-X10) and every standalone SVG image (X7).
///
/// A document that is already canonical keeps its exact bytes, because reading
/// positions depend on stable text. Documents are processed in path order so the
/// output is deterministic. An SVG that cannot be parsed cannot be shown safe,
/// so it is removed from the archive and the manifest.
pub fn repair_content_documents(
    opf: &mut Element,
    opf_path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
) -> Result<ContentReport, ContentError> {
    if opf.name != "package" {
        return Err(ContentError::MissingPackage);
    }

    let opf_path = normalize_path(opf_path).ok_or(ContentError::InvalidOpfPath)?;
    let opf_dir = parent_path(&opf_path);
    let documents = manifest_documents(opf, &opf_dir, archive)?;
    let mut report = ContentReport::default();

    for (path, kind) in documents {
        match kind {
            DocumentKind::Xhtml => repair_xhtml_entry(&path, archive, &mut report)?,
            DocumentKind::Svg => repair_svg_entry(&path, archive, &mut report)?,
        }
    }

    if !report.removed_images.is_empty() {
        remove_manifest_items(opf, &opf_dir, &report.removed_images);
        report.changed = true;
    }

    Ok(report)
}

fn manifest_documents(
    opf: &Element,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Result<Vec<(String, DocumentKind)>, ContentError> {
    let manifest = child_element(opf, "manifest").ok_or(ContentError::MissingManifest)?;

    let mut documents: Vec<(String, DocumentKind)> = manifest
        .children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(item) if item.name == "item" => {
                document_entry(item, opf_dir, archive)
            }
            _ => None,
        })
        .collect();

    documents.sort();
    documents.dedup();
    Ok(documents)
}

fn document_entry(
    item: &Element,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<(String, DocumentKind)> {
    let kind = match item.attributes.get("media-type")?.as_str() {
        XHTML_MEDIA_TYPE => DocumentKind::Xhtml,
        SVG_MEDIA_TYPE => DocumentKind::Svg,
        _ => return None,
    };
    let path = resolve_path(opf_dir, item.attributes.get("href")?)?;

    archive.contains_key(&path).then_some((path, kind))
}

fn repair_xhtml_entry(
    path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
    report: &mut ContentReport,
) -> Result<(), ContentError> {
    let Some(bytes) = archive.get(path) else {
        return Ok(());
    };
    let outcome = repair_xhtml(bytes)?;

    report.add_repairs(&outcome.repairs);
    if outcome.transcoded {
        report.transcoded_documents.push(path.to_owned());
    }
    if outcome.reparsed_as_html {
        report.reparsed_documents.push(path.to_owned());
    }
    if outcome.empty {
        report.empty_documents.push(path.to_owned());
    }
    if let Some(rewritten) = outcome.rewritten {
        archive.insert(path.to_owned(), rewritten);
        report.changed = true;
    }

    Ok(())
}

fn repair_svg_entry(
    path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
    report: &mut ContentReport,
) -> Result<(), ContentError> {
    let Some(bytes) = archive.get(path) else {
        return Ok(());
    };
    let mut repairs = Repairs::default();
    let fate = repair_svg(bytes, &mut repairs)?;

    report.add_repairs(&repairs);
    match fate {
        SvgFate::Unchanged => {}
        SvgFate::Rewritten(rewritten) => {
            archive.insert(path.to_owned(), rewritten);
            report.changed = true;
        }
        SvgFate::Unusable => {
            archive.remove(path);
            report.removed_images.push(path.to_owned());
        }
    }

    Ok(())
}

fn repair_xhtml(bytes: &[u8]) -> Result<XhtmlOutcome, ContentError> {
    let ParsedDocument {
        mut root,
        transcoded,
        reparsed_as_html,
    } = parse_xhtml(bytes);
    let mut repairs = Repairs::default();

    let structure_repaired = ensure_xhtml_root(&mut root);
    sanitize_element(&mut root, false, &mut repairs);
    if reparsed_as_html {
        declare_used_prefixes(&mut root);
    }

    let empty = !child_element(&root, "body").is_some_and(has_content);
    let needs_rewrite = transcoded || reparsed_as_html || structure_repaired || repairs.any();
    let rewritten = if needs_rewrite {
        Some(serialize(&root)?)
    } else {
        None
    };

    Ok(XhtmlOutcome {
        rewritten,
        repairs,
        transcoded,
        reparsed_as_html,
        empty,
    })
}

fn repair_svg(bytes: &[u8], repairs: &mut Repairs) -> Result<SvgFate, ContentError> {
    let (text, transcoded) = decode_text(bytes);
    let Ok(mut root) = Element::parse(strip_xml_declaration(&text).as_bytes()) else {
        return Ok(SvgFate::Unusable);
    };

    if local_name(&root.name) != "svg" {
        return Ok(SvgFate::Unusable);
    }

    sanitize_element(&mut root, true, repairs);

    if !transcoded && !repairs.any() {
        return Ok(SvgFate::Unchanged);
    }
    Ok(SvgFate::Rewritten(serialize(&root)?))
}

fn parse_xhtml(bytes: &[u8]) -> ParsedDocument {
    let (text, transcoded) = decode_text(bytes);
    let markup = strip_xml_declaration(&text);

    match Element::parse(markup.as_bytes()) {
        Ok(root) if local_name(&root.name) == "html" => ParsedDocument {
            root,
            transcoded,
            reparsed_as_html: false,
        },
        _ => ParsedDocument {
            root: parse_as_html(markup),
            transcoded,
            reparsed_as_html: true,
        },
    }
}

/// Returns the text and whether it differed from valid UTF-8 without a BOM.
///
/// Bytes that are valid UTF-8 are always read as UTF-8, whatever the XML
/// declaration claims, because mislabelled files are more common than
/// UTF-8-looking Latin-1. The declaration is only consulted for invalid bytes.
fn decode_text(bytes: &[u8]) -> (String, bool) {
    if let Some((encoding, bom_length)) = Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom_length..]);
        return (text.into_owned(), true);
    }

    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_owned(), false),
        Err(_) => {
            let encoding = declared_encoding(bytes).unwrap_or(WINDOWS_1252);
            let (text, _) = encoding.decode_without_bom_handling(bytes);
            (text.into_owned(), true)
        }
    }
}

fn declared_encoding(bytes: &[u8]) -> Option<&'static Encoding> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
    let declaration = head
        .trim_start()
        .strip_prefix("<?xml")?
        .split("?>")
        .next()?;
    let label = declaration
        .split("encoding=")
        .nth(1)?
        .trim_start_matches(['"', '\''])
        .split(['"', '\''])
        .next()?;

    Encoding::for_label(label.trim().as_bytes()).filter(|encoding| *encoding != UTF_8)
}

fn strip_xml_declaration(text: &str) -> &str {
    let trimmed = text.trim_start();
    trimmed
        .strip_prefix("<?xml")
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .and_then(|rest| rest.split_once("?>"))
        .map_or(trimmed, |(_, document)| document)
}

/// `scripting_enabled` is off so `<noscript>` content is parsed as elements;
/// otherwise its markup would be serialized as escaped text.
fn parse_as_html(markup: &str) -> Element {
    let options = ParseOpts {
        tree_builder: TreeBuilderOpts {
            scripting_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), options).one(markup);

    let top_level = dom.document.children.borrow();
    let root = top_level
        .iter()
        .flat_map(|node| convert_node(node, ""))
        .find_map(|node| match node {
            XMLNode::Element(root) => Some(root),
            _ => None,
        })
        .unwrap_or_else(|| Element::new("html"));
    root
}

fn convert_node(node: &Handle, parent_namespace: &str) -> Vec<XMLNode> {
    match &node.data {
        NodeData::Text { contents } => vec![XMLNode::Text(xml_safe(&contents.borrow()))],
        NodeData::Element { name, attrs, .. } => {
            convert_element(node, name, &attrs.borrow(), parent_namespace)
        }
        _ => Vec::new(),
    }
}

fn convert_element(
    node: &Handle,
    name: &QualName,
    attributes: &[Attribute],
    parent_namespace: &str,
) -> Vec<XMLNode> {
    let namespace: &str = &name.ns;
    let (prefix, local) = split_qualified(&name.local);
    let convert_children = |child_namespace: &str| -> Vec<XMLNode> {
        node.children
            .borrow()
            .iter()
            .flat_map(|child| convert_node(child, child_namespace))
            .collect()
    };

    if !is_xml_name(local) || !prefix.is_none_or(is_known_prefix) {
        return convert_children(parent_namespace);
    }

    let mut element = Element::new(local);
    if prefix == Some("epub") {
        element.prefix = Some("epub".to_owned());
        element.namespace = Some(EPUB_NAMESPACE.to_owned());
    }
    for attribute in attributes {
        if let Some((key, value)) = convert_attribute(attribute) {
            element.attributes.insert(key, value);
        }
    }
    if namespace != parent_namespace {
        element
            .attributes
            .insert("xmlns".to_owned(), namespace.to_owned());
    }
    element.children = convert_children(namespace);

    vec![XMLNode::Element(element)]
}

fn convert_attribute(attribute: &Attribute) -> Option<(String, String)> {
    let key = match &attribute.name.prefix {
        Some(prefix) => format!("{prefix}:{}", attribute.name.local),
        None => attribute.name.local.to_string(),
    };
    let (prefix, local) = split_qualified(&key);
    let is_declaration = key == "xmlns" || prefix == Some("xmlns");
    let is_serializable = is_xml_name(local) && prefix.is_none_or(is_known_prefix);

    (is_serializable && !is_declaration).then(|| (key.clone(), xml_safe(&attribute.value)))
}

fn split_qualified(name: &str) -> (Option<&str>, &str) {
    match name.split_once(':') {
        Some((prefix, local)) => (Some(prefix), local),
        None => (None, name),
    }
}

fn is_known_prefix(prefix: &str) -> bool {
    prefix == "xml" || DECLARED_PREFIXES.iter().any(|(known, _)| *known == prefix)
}

fn is_xml_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn xml_safe(text: &str) -> String {
    text.chars()
        .filter(|ch| {
            matches!(
                ch,
                '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}'
                    | '\u{10000}'..='\u{10FFFF}'
            )
        })
        .collect()
}

fn ensure_xhtml_root(root: &mut Element) -> bool {
    let namespace_declared = root.namespace.as_deref() == Some(XHTML_NAMESPACE)
        || root.attributes.get("xmlns").map(String::as_str) == Some(XHTML_NAMESPACE);

    if !namespace_declared {
        root.attributes
            .insert("xmlns".to_owned(), XHTML_NAMESPACE.to_owned());
    }

    let body_added = ensure_body(root);
    !namespace_declared || body_added
}

fn ensure_body(root: &mut Element) -> bool {
    if child_element(root, "body").is_some() {
        return false;
    }

    let (head, loose): (Vec<XMLNode>, Vec<XMLNode>) = std::mem::take(&mut root.children)
        .into_iter()
        .partition(is_head);
    let mut body = Element::new("body");
    body.children = loose;

    root.children = head;
    root.children.push(XMLNode::Element(body));
    true
}

fn is_head(node: &XMLNode) -> bool {
    matches!(node, XMLNode::Element(element) if local_name(&element.name) == "head")
}

fn declare_used_prefixes(root: &mut Element) {
    for (prefix, namespace) in DECLARED_PREFIXES {
        if uses_prefix(root, prefix) {
            root.attributes
                .insert(format!("xmlns:{prefix}"), namespace.to_owned());
        }
    }
}

fn uses_prefix(element: &Element, prefix: &str) -> bool {
    element.prefix.as_deref() == Some(prefix)
        || element.attributes.keys().any(|key| {
            key.strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with(':'))
        })
        || element
            .children
            .iter()
            .any(|node| matches!(node, XMLNode::Element(child) if uses_prefix(child, prefix)))
}

fn sanitize_element(element: &mut Element, in_svg: bool, repairs: &mut Repairs) {
    let before = element.attributes.len();
    element
        .attributes
        .retain(|name, value| !is_unsafe_attribute(name, value));
    repairs.removed_attributes += before - element.attributes.len();

    let children = std::mem::take(&mut element.children);
    element.children = sanitize_nodes(children, in_svg, repairs);
}

fn sanitize_nodes(nodes: Vec<XMLNode>, in_svg: bool, repairs: &mut Repairs) -> Vec<XMLNode> {
    let mut sanitized = Vec::new();

    for node in nodes {
        let XMLNode::Element(mut child) = node else {
            sanitized.push(node);
            continue;
        };
        let child_in_svg = in_svg || local_name(&child.name) == "svg";

        match classify(&child, child_in_svg) {
            Fate::Keep => {
                sanitize_element(&mut child, child_in_svg, repairs);
                sanitized.push(XMLNode::Element(child));
            }
            Fate::Remove => repairs.removed_elements += 1,
            Fate::Unwrap => {
                repairs.unwrapped_elements += 1;
                let children = unwrapped_children(child);
                sanitized.extend(sanitize_nodes(children, child_in_svg, repairs));
            }
            Fate::UseDefaultBranch => {
                repairs.resolved_switches += 1;
                let children = default_branch(child);
                sanitized.extend(sanitize_nodes(children, child_in_svg, repairs));
            }
        }
    }

    sanitized
}

/// `form`, `audio` and `video` are unwrapped rather than removed so the text
/// inside them keeps its reading positions.
fn classify(element: &Element, in_svg: bool) -> Fate {
    let name = local_name(&element.name).to_ascii_lowercase();

    if is_epub_element(element, "switch") {
        Fate::UseDefaultBranch
    } else if is_forbidden(&name, element, in_svg) {
        Fate::Remove
    } else if matches!(name.as_str(), "form" | "audio" | "video") {
        Fate::Unwrap
    } else {
        Fate::Keep
    }
}

fn is_forbidden(name: &str, element: &Element, in_svg: bool) -> bool {
    match name {
        "script" | "iframe" | "object" | "embed" | "applet" | "base" | "foreignobject" => true,
        "meta" => element
            .attributes
            .keys()
            .any(|attribute| attribute.eq_ignore_ascii_case("http-equiv")),
        "style" => is_unsafe_css(&element_text(element)),
        _ => in_svg && has_external_reference(element),
    }
}

fn is_epub_element(element: &Element, name: &str) -> bool {
    local_name(&element.name) == name
        && (element.prefix.as_deref() == Some("epub")
            || element.namespace.as_deref() == Some(EPUB_NAMESPACE))
}

fn default_branch(switch: Element) -> Vec<XMLNode> {
    switch
        .children
        .into_iter()
        .find_map(|node| match node {
            XMLNode::Element(branch) if is_epub_element(&branch, "default") => {
                Some(branch.children)
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn unwrapped_children(element: Element) -> Vec<XMLNode> {
    let is_media = matches!(local_name(&element.name), "audio" | "video");

    element
        .children
        .into_iter()
        .filter(|node| !(is_media && is_media_source(node)))
        .collect()
}

fn is_media_source(node: &XMLNode) -> bool {
    matches!(
        node,
        XMLNode::Element(element) if matches!(local_name(&element.name), "source" | "track")
    )
}

fn is_unsafe_attribute(name: &str, value: &str) -> bool {
    let name = local_name(name).to_ascii_lowercase();
    let is_event_handler = name.len() > 2 && name.starts_with("on");

    is_event_handler
        || name == "srcdoc"
        || (URL_ATTRIBUTES.contains(&name.as_str()) && is_javascript_url(value))
        || (name == "style" && is_unsafe_css(value))
}

fn is_javascript_url(value: &str) -> bool {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_control())
        .collect::<String>()
        .to_ascii_lowercase()
        .starts_with("javascript:")
}

fn has_external_reference(element: &Element) -> bool {
    element
        .attributes
        .iter()
        .any(|(name, value)| local_name(name) == "href" && is_external_url(value))
}

fn is_external_url(url: &str) -> bool {
    let url = url.trim().to_ascii_lowercase();
    url.starts_with("//") || (has_uri_scheme(&url) && !url.starts_with("data:image/"))
}

fn is_unsafe_css(css: &str) -> bool {
    let normalized = strip_css_comments(&decode_css_escapes(css))
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();

    FORBIDDEN_CSS_TOKENS
        .iter()
        .any(|token| normalized.contains(token))
        || css_urls(&normalized).into_iter().any(is_external_url)
}

/// Escapes such as `\65 xpression(` are decoded first so they cannot hide a
/// forbidden token from the checks.
fn decode_css_escapes(css: &str) -> String {
    let mut decoded = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        let hex: String = std::iter::from_fn(|| chars.next_if(char::is_ascii_hexdigit))
            .take(6)
            .collect();

        if hex.is_empty() {
            decoded.extend(chars.next());
            continue;
        }

        let code_point = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
        decoded.push(code_point.unwrap_or('\u{FFFD}'));
        chars.next_if(|next| next.is_whitespace());
    }

    decoded
}

fn strip_css_comments(css: &str) -> String {
    let mut stripped = String::with_capacity(css.len());
    let mut rest = css;

    while let Some(start) = rest.find("/*") {
        stripped.push_str(&rest[..start]);
        let comment = &rest[start + 2..];
        rest = comment.find("*/").map_or("", |end| &comment[end + 2..]);
    }

    stripped.push_str(rest);
    stripped
}

fn css_urls(css: &str) -> Vec<&str> {
    let mut urls = Vec::new();
    let mut rest = css;

    while let Some(start) = rest.find("url(") {
        let after = &rest[start + 4..];
        let end = after.find(')').unwrap_or(after.len());
        urls.push(after[..end].trim_matches(['\'', '"']));
        rest = &after[end..];
    }

    urls
}

fn has_content(body: &Element) -> bool {
    body.children.iter().any(|node| match node {
        XMLNode::Text(text) | XMLNode::CData(text) => !text.trim().is_empty(),
        XMLNode::Element(child) => match local_name(&child.name) {
            "img" | "image" | "svg" | "picture" => true,
            "style" => false,
            _ => has_content(child),
        },
        _ => false,
    })
}

fn element_text(element: &Element) -> String {
    element
        .children
        .iter()
        .map(|node| match node {
            XMLNode::Text(text) | XMLNode::CData(text) => text.clone(),
            XMLNode::Element(child) => element_text(child),
            _ => String::new(),
        })
        .collect()
}

fn child_element<'a>(parent: &'a Element, name: &str) -> Option<&'a Element> {
    parent.children.iter().find_map(|node| match node {
        XMLNode::Element(element) if local_name(&element.name) == name => Some(element),
        _ => None,
    })
}

fn remove_manifest_items(opf: &mut Element, opf_dir: &str, removed_paths: &[String]) {
    for node in &mut opf.children {
        let XMLNode::Element(manifest) = node else {
            continue;
        };
        if manifest.name != "manifest" {
            continue;
        }
        manifest
            .children
            .retain(|child| !is_item_at_any_path(child, opf_dir, removed_paths));
    }
}

fn is_item_at_any_path(node: &XMLNode, opf_dir: &str, paths: &[String]) -> bool {
    let XMLNode::Element(item) = node else {
        return false;
    };

    item.name == "item"
        && item
            .attributes
            .get("href")
            .and_then(|href| resolve_path(opf_dir, href))
            .is_some_and(|path| paths.contains(&path))
}

fn serialize(root: &Element) -> Result<Vec<u8>, ContentError> {
    let mut bytes = Vec::new();

    root.write(&mut bytes)
        .map_err(|_| ContentError::WriteFailed)?;

    Ok(bytes)
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
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
    if href.starts_with('/')
        || href.contains('\\')
        || href.contains('\0')
        || href.contains(':')
    {
        return None;
    }

    let path = href.split(['?', '#']).next()?;
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();

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

#[cfg(test)]
#[path = "content_tests.rs"]
mod content_tests;