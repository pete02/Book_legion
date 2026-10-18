use std::collections::HashSet;

use xmltree::{Element, XMLNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpineError {
    MissingSpine,
    EmptySpine,
}

impl std::fmt::Display for SpineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSpine => write!(f, "OPF has no spine"),
            Self::EmptySpine => write!(f, "spine has no itemrefs"),
        }
    }
}

impl std::error::Error for SpineError {}

/// What was repaired, as the `idref` of each affected itemref in document order.
/// An itemref without an `idref` attribute is reported as an empty string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpineReport {
    pub dangling_idrefs: Vec<String>,
    pub duplicate_idrefs: Vec<String>,
    pub non_linear_idrefs: Vec<String>,
    pub changed: bool,
}

impl SpineReport {
    fn new(
        dangling_idrefs: Vec<String>,
        duplicate_idrefs: Vec<String>,
        non_linear_idrefs: Vec<String>,
    ) -> Self {
        let changed = !dangling_idrefs.is_empty()
            || !duplicate_idrefs.is_empty()
            || !non_linear_idrefs.is_empty();
        Self {
            dangling_idrefs,
            duplicate_idrefs,
            non_linear_idrefs,
            changed,
        }
    }
}

/// Repairs the spine so that spine item N is chapter N.
///
/// S2, S4 and S3 are repaired in that order, and S1 is checked last, so a spine
/// whose itemrefs all dangle is rejected as empty. Only the spine and the manifest
/// ids are consulted; the nav document plays no part in chapter order.
///
/// On error the OPF may be partly modified and should be discarded.
pub fn repair_spine(opf: &mut Element) -> Result<SpineReport, SpineError> {
    let manifest_ids = manifest_item_ids(opf);
    let spine = find_spine_mut(opf).ok_or(SpineError::MissingSpine)?;

    let dangling_idrefs = drop_dangling_itemrefs(spine, &manifest_ids);
    let duplicate_idrefs = drop_duplicate_itemrefs(spine);
    let non_linear_idrefs = clear_non_linear(spine);

    if itemref_count(spine) == 0 {
        return Err(SpineError::EmptySpine);
    }

    Ok(SpineReport::new(
        dangling_idrefs,
        duplicate_idrefs,
        non_linear_idrefs,
    ))
}

fn manifest_item_ids(opf: &Element) -> HashSet<String> {
    child_elements(opf, "manifest")
        .flat_map(|manifest| child_elements(manifest, "item"))
        .filter_map(|item| item.attributes.get("id").cloned())
        .collect()
}

fn find_spine_mut(opf: &mut Element) -> Option<&mut Element> {
    opf.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element) if element.name == "spine" => Some(element),
        _ => None,
    })
}

fn drop_dangling_itemrefs(spine: &mut Element, manifest_ids: &HashSet<String>) -> Vec<String> {
    remove_itemrefs(spine, |itemref| !manifest_ids.contains(idref(itemref)))
}

fn drop_duplicate_itemrefs(spine: &mut Element) -> Vec<String> {
    let mut seen = HashSet::new();
    remove_itemrefs(spine, |itemref| !seen.insert(idref(itemref).to_string()))
}

fn clear_non_linear(spine: &mut Element) -> Vec<String> {
    let mut cleared = Vec::new();
    for node in &mut spine.children {
        let XMLNode::Element(itemref) = node else {
            continue;
        };
        if is_itemref(itemref) && is_non_linear(itemref) {
            itemref.attributes.remove("linear");
            cleared.push(idref(itemref).to_string());
        }
    }
    cleared
}

fn remove_itemrefs(
    spine: &mut Element,
    mut should_remove: impl FnMut(&Element) -> bool,
) -> Vec<String> {
    let mut removed = Vec::new();
    spine.children.retain(|node| {
        let XMLNode::Element(itemref) = node else {
            return true;
        };
        if !is_itemref(itemref) || !should_remove(itemref) {
            return true;
        }
        removed.push(idref(itemref).to_string());
        false
    });
    removed
}

fn itemref_count(spine: &Element) -> usize {
    child_elements(spine, "itemref").count()
}

fn is_itemref(element: &Element) -> bool {
    element.name == "itemref"
}

fn is_non_linear(itemref: &Element) -> bool {
    itemref.attributes.get("linear").map(String::as_str) == Some("no")
}

fn idref(itemref: &Element) -> &str {
    itemref
        .attributes
        .get("idref")
        .map(String::as_str)
        .unwrap_or("")
}

fn child_elements<'a>(parent: &'a Element, name: &'a str) -> impl Iterator<Item = &'a Element> {
    parent.children.iter().filter_map(move |node| match node {
        XMLNode::Element(element) if element.name == name => Some(element),
        _ => None,
    })
}

#[cfg(test)]
#[path = "spine_tests.rs"]
mod spine_tests;
