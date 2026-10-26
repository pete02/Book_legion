use std::collections::HashSet;

use sha2::{Digest, Sha256};
use xmltree::{Element, XMLNode};

use super::references::normalize_path;

pub const PROFILE_VERSION: u32 = 1;

const DC_NAMESPACE: &str = "http://purl.org/dc/elements/1.1/";
const UNTITLED: &str = "Untitled";
const UNKNOWN_LANGUAGE: &str = "und";
const IDENTIFIER_ID: &str = "pub-id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    MissingPackage,
    MissingSpine,
    InvalidOpfPath,
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::MissingSpine => write!(f, "OPF has no spine"),
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
        }
    }
}

impl std::error::Error for MetadataError {}

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub language: String,
    pub identifier: String,
    pub series: Option<String>,
    pub series_index: Option<f64>,
    pub file_hash: String,
    pub chapter_count: usize,
    pub profile_version: u32,
    pub opf_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetadataReport {
    pub changed: bool,
    pub inserted_title: bool,
    pub repaired_language: bool,
    pub repaired_identifier: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataResult {
    pub metadata: LibraryMetadata,
    pub report: MetadataReport,
}

struct IdentifierChoice {
    id: String,
    value: String,
}

struct SeriesInfo {
    name: String,
    index: Option<f64>,
}

/// Guarantees the OPF has a title, a language and a unique identifier (L1, L3, L4)
/// and returns everything the library keeps about the book (L1-L9).
///
/// `original` is the untouched upload, whose SHA-256 is the `file_hash`. Run this
/// last, so that the chapter count is the count of the final spine. A missing
/// language becomes `und`, a missing title `Untitled`, and a book with no usable
/// identifier gets `urn:sha256:<file_hash>`, so the same upload always maps to the
/// same identifier.
pub fn repair_metadata(
    opf: &mut Element,
    opf_path: &str,
    original: &[u8],
) -> Result<MetadataResult, MetadataError> {
    if opf.name != "package" {
        return Err(MetadataError::MissingPackage);
    }

    let opf_path = normalize_path(opf_path).ok_or(MetadataError::InvalidOpfPath)?;
    let chapter_count = count_chapters(opf)?;
    let file_hash = sha256_hex(original);
    let declared_identifier = opf.attributes.get("unique-identifier").cloned();
    let mut report = MetadataReport::default();

    report.changed |= ensure_metadata(opf);
    let metadata = child_element_mut(opf, "metadata").expect("metadata was ensured");

    let title = repair_title(metadata, &mut report);
    let language = repair_language(metadata, &mut report);
    let identifier = repair_identifier(
        metadata,
        declared_identifier.as_deref(),
        &file_hash,
        &mut report,
    );
    let authors = find_authors(metadata);
    let series = find_series(metadata);

    if declared_identifier.as_deref() != Some(identifier.id.as_str()) {
        opf.attributes
            .insert("unique-identifier".to_owned(), identifier.id);
    }

    let metadata = LibraryMetadata {
        title,
        authors,
        language,
        identifier: identifier.value,
        series_index: series.as_ref().and_then(|series| series.index),
        series: series.map(|series| series.name),
        file_hash,
        chapter_count,
        profile_version: PROFILE_VERSION,
        opf_path,
    };

    Ok(MetadataResult { metadata, report })
}

fn count_chapters(opf: &Element) -> Result<usize, MetadataError> {
    let spine = child_element(opf, "spine").ok_or(MetadataError::MissingSpine)?;

    Ok(child_elements(spine, "itemref").count())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn ensure_metadata(opf: &mut Element) -> bool {
    if child_element(opf, "metadata").is_some() {
        return false;
    }

    opf.children
        .insert(0, XMLNode::Element(Element::new("metadata")));
    true
}

fn repair_title(metadata: &mut Element, report: &mut MetadataReport) -> String {
    let before = metadata.children.len();
    metadata
        .children
        .retain(|node| !is_blank_dc_element(node, "title"));
    report.changed |= metadata.children.len() != before;

    if let Some(title) = dc_elements(metadata, "title").next() {
        return collapse_whitespace(&element_text(title));
    }

    let title = new_dc_element(metadata, "title", UNTITLED);
    metadata.children.push(XMLNode::Element(title));
    report.inserted_title = true;
    report.changed = true;
    UNTITLED.to_owned()
}

fn repair_language(metadata: &mut Element, report: &mut MetadataReport) -> String {
    let current = dc_elements(metadata, "language").next().map(element_text);
    let language = current
        .as_deref()
        .and_then(normalize_language)
        .unwrap_or_else(|| UNKNOWN_LANGUAGE.to_owned());

    match current {
        None => {
            let element = new_dc_element(metadata, "language", &language);
            metadata.children.push(XMLNode::Element(element));
        }
        Some(current) if current.trim() != language => {
            set_first_dc_text(metadata, "language", &language);
        }
        Some(_) => return language,
    }

    report.repaired_language = true;
    report.changed = true;
    language
}

/// Only the shape of the tag is checked and normalized (`en_us` becomes `en-US`);
/// whether the subtags are registered is not.
fn normalize_language(raw: &str) -> Option<String> {
    let tag = raw.trim().replace('_', "-");
    let mut subtags = tag.split('-');
    let primary = subtags.next()?;

    if !(2..=3).contains(&primary.len()) || !primary.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    let mut normalized = vec![primary.to_ascii_lowercase()];
    for subtag in subtags {
        let is_valid = (1..=8).contains(&subtag.len())
            && subtag.chars().all(|ch| ch.is_ascii_alphanumeric());
        if !is_valid {
            return None;
        }
        normalized.push(with_subtag_case(subtag));
    }

    Some(normalized.join("-"))
}

fn with_subtag_case(subtag: &str) -> String {
    let is_alphabetic = subtag.chars().all(|ch| ch.is_ascii_alphabetic());

    match subtag.len() {
        4 if is_alphabetic => {
            let (first, rest) = subtag.split_at(1);
            format!("{}{}", first.to_ascii_uppercase(), rest.to_ascii_lowercase())
        }
        2 if is_alphabetic => subtag.to_ascii_uppercase(),
        _ => subtag.to_ascii_lowercase(),
    }
}

fn repair_identifier(
    metadata: &mut Element,
    declared_id: Option<&str>,
    file_hash: &str,
    report: &mut MetadataReport,
) -> IdentifierChoice {
    let candidates: Vec<(Option<String>, String)> = dc_elements(metadata, "identifier")
        .map(|element| {
            (
                element.attributes.get("id").cloned(),
                element_text(element).trim().to_owned(),
            )
        })
        .filter(|(_, value)| !value.is_empty())
        .collect();

    let declared = declared_id.and_then(|id| {
        candidates
            .iter()
            .find(|(candidate, _)| candidate.as_deref() == Some(id))
    });
    if let Some((Some(id), value)) = declared {
        return IdentifierChoice {
            id: id.clone(),
            value: value.clone(),
        };
    }

    report.repaired_identifier = true;
    report.changed = true;

    match candidates.into_iter().next() {
        Some((Some(id), value)) => IdentifierChoice { id, value },
        Some((None, value)) => {
            let id = assign_identifier_id(metadata, &value);
            IdentifierChoice { id, value }
        }
        None => synthesize_identifier(metadata, file_hash),
    }
}

fn assign_identifier_id(metadata: &mut Element, value: &str) -> String {
    let id = unused_id(metadata);

    let unnamed = metadata.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element)
            if is_dc_element(&element, "identifier")
                && !element.attributes.contains_key("id")
                && element_text(&element).trim() == value =>
        {
            Some(element)
        }
        _ => None,
    });
    if let Some(element) = unnamed {
        element.attributes.insert("id".to_owned(), id.clone());
    }

    id
}

fn synthesize_identifier(metadata: &mut Element, file_hash: &str) -> IdentifierChoice {
    let id = unused_id(metadata);
    let value = format!("urn:sha256:{file_hash}");

    let mut element = new_dc_element(metadata, "identifier", &value);
    element.attributes.insert("id".to_owned(), id.clone());
    metadata.children.push(XMLNode::Element(element));

    IdentifierChoice { id, value }
}

fn unused_id(metadata: &Element) -> String {
    let used: HashSet<&str> = metadata
        .children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) => element.attributes.get("id").map(String::as_str),
            _ => None,
        })
        .collect();

    std::iter::once(IDENTIFIER_ID.to_owned())
        .chain((2..).map(|suffix| format!("{IDENTIFIER_ID}-{suffix}")))
        .find(|candidate| !used.contains(candidate.as_str()))
        .expect("the candidate sequence is infinite")
}

fn find_authors(metadata: &Element) -> Vec<String> {
    dc_elements(metadata, "creator")
        .map(|element| collapse_whitespace(&element_text(element)))
        .filter(|author| !author.is_empty())
        .collect()
}

fn find_series(metadata: &Element) -> Option<SeriesInfo> {
    collection_series(metadata).or_else(|| calibre_series(metadata))
}

fn collection_series(metadata: &Element) -> Option<SeriesInfo> {
    let metas: Vec<&Element> = child_elements(metadata, "meta").collect();

    metas.iter().find_map(|meta| {
        if meta.attributes.get("property").map(String::as_str) != Some("belongs-to-collection") {
            return None;
        }

        let name = collapse_whitespace(&element_text(meta));
        let refinement = |property: &str| refinement_value(&metas, meta, property);
        let is_series = refinement("collection-type").is_none_or(|kind| kind == "series");

        (!name.is_empty() && is_series).then(|| SeriesInfo {
            name,
            index: refinement("group-position").and_then(|position| position.parse().ok()),
        })
    })
}

fn refinement_value(metas: &[&Element], collection: &Element, property: &str) -> Option<String> {
    let target = format!("#{}", collection.attributes.get("id")?);

    metas
        .iter()
        .find(|meta| {
            meta.attributes.get("refines") == Some(&target)
                && meta.attributes.get("property").map(String::as_str) == Some(property)
        })
        .map(|meta| element_text(meta).trim().to_owned())
}

fn calibre_series(metadata: &Element) -> Option<SeriesInfo> {
    let content = |name: &str| {
        child_elements(metadata, "meta")
            .find(|meta| meta.attributes.get("name").map(String::as_str) == Some(name))
            .and_then(|meta| meta.attributes.get("content"))
            .map(|content| content.trim().to_owned())
    };
    let name = content("calibre:series").filter(|name| !name.is_empty())?;

    Some(SeriesInfo {
        name,
        index: content("calibre:series_index").and_then(|index| index.parse().ok()),
    })
}

fn new_dc_element(metadata: &mut Element, name: &str, text: &str) -> Element {
    let is_declared = metadata.attributes.contains_key("xmlns:dc")
        || metadata
            .namespaces
            .as_ref()
            .is_some_and(|namespaces| namespaces.get("dc").is_some());
    if !is_declared {
        metadata
            .attributes
            .insert("xmlns:dc".to_owned(), DC_NAMESPACE.to_owned());
    }

    let mut element = Element::new(&format!("dc:{name}"));
    element.children.push(XMLNode::Text(text.to_owned()));
    element
}

fn set_first_dc_text(metadata: &mut Element, name: &str, text: &str) {
    let first = metadata.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element) if is_dc_element(&element, name) => Some(element),
        _ => None,
    });

    if let Some(element) = first {
        element.children = vec![XMLNode::Text(text.to_owned())];
    }
}

fn is_dc_element(element: &Element, name: &str) -> bool {
    local_name(&element.name) == name
        && (element.prefix.as_deref() == Some("dc")
            || element.namespace.as_deref() == Some(DC_NAMESPACE)
            || element.name.starts_with("dc:"))
}

fn is_blank_dc_element(node: &XMLNode, name: &str) -> bool {
    matches!(
        node,
        XMLNode::Element(element)
            if is_dc_element(element, name) && element_text(element).trim().is_empty()
    )
}

fn dc_elements<'a>(
    metadata: &'a Element,
    name: &'a str,
) -> impl Iterator<Item = &'a Element> {
    metadata.children.iter().filter_map(move |node| match node {
        XMLNode::Element(element) if is_dc_element(&element, name) => Some(element),
        _ => None,
    })
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

fn child_element_mut<'a>(parent: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    parent.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element) if local_name(&element.name) == name => Some(element),
        _ => None,
    })
}

fn element_text(element: &Element) -> String {
    element
        .children
        .iter()
        .map(|node| match node {
            XMLNode::Text(text) | XMLNode::CData(text) => text.clone(),
            XMLNode::Element(child) => element_text(&child),
            _ => String::new(),
        })
        .collect()
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;