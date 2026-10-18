
use std::{
    collections::HashMap,
    io::Cursor,
    path::{Component, Path, PathBuf},
};

use xmltree::{Element, XMLNode};

#[derive(Debug, PartialEq)]
pub enum OpfError {
    MalformedXml,
    InvalidPackageRoot,
    MissingManifest,
    MissingSpine,
    ConflictingManifestItem(String),
    InvalidCoverPath,
    WriteFailed,
}

#[derive(Debug, PartialEq)]
pub struct OpfResult {
    pub xml: Vec<u8>,
    pub opf_path: String,
    pub cover_path: Option<String>,
    pub repaired: bool,
}

pub fn verify_opf(
    bytes: &[u8],
    opf_path: &str,
    archive_files: &[String],
) -> Result<OpfResult, OpfError> {
    let mut package = parse_opf(bytes)?;

    if package.name != "package" {
        return Err(OpfError::InvalidPackageRoot);
    }

    let cover_path = find_guide_cover(&package, opf_path, archive_files)?;

    let repaired = merge_manifests(&mut package)?
        | merge_spines(&mut package)?
        | strip_guides(&mut package);

    let mut xml = Vec::new();

    package
        .write(Cursor::new(&mut xml))
        .map_err(|_| OpfError::WriteFailed)?;

    Ok(OpfResult {
        xml,
        opf_path: opf_path.to_owned(),
        cover_path,
        repaired,
    })
}

fn parse_opf(bytes: &[u8]) -> Result<Element, OpfError> {
    Element::parse(Cursor::new(bytes))
        .map_err(|_| OpfError::MalformedXml)
}

fn direct_elements<'a>(
    parent: &'a Element,
    name: &'static str,
) -> impl Iterator<Item = &'a Element> {
    parent.children.iter().filter_map(move |node| match node {
        XMLNode::Element(element) if element.name == name => Some(element),
        _ => None,
    })
}

fn direct_elements_mut<'a>(
    parent: &'a mut Element,
    name: &'static str,
) -> impl Iterator<Item = &'a mut Element> {
    parent.children.iter_mut().filter_map(move |node| match node {
        XMLNode::Element(element) if element.name == name => Some(element),
        _ => None,
    })
}

fn merge_manifests(package: &mut Element) -> Result<bool, OpfError> {
    let count = direct_elements(package, "manifest").count();

    if count == 0 {
        return Err(OpfError::MissingManifest);
    }

    if count == 1 {
        return Ok(false);
    }

    let mut manifests = Vec::new();
    let mut retained = None;

    for node in std::mem::take(&mut package.children) {
        match node {
            XMLNode::Element(element) if element.name == "manifest" => {
                if let Some(first) = &mut retained {
                    append_manifest_items(first, element)?;
                } else {
                    retained = Some(element);
                }
            }
            other => manifests.push(other),
        }
    }

    let manifest = retained.expect("manifest count was checked");
    insert_at_original_position(&mut manifests, manifest, "manifest");

    package.children = manifests;
    Ok(true)
}

fn append_manifest_items(
    target: &mut Element,
    source: Element,
) -> Result<(), OpfError> {
    let mut ids: HashMap<String, HashMap<String, String>> =
        direct_elements(target, "item")
            .filter_map(|item| {
                item.attributes
                    .get("id")
                    .map(|id| (id.clone(), item.attributes.clone()))
            })
            .collect();

    for node in source.children {
        let XMLNode::Element(item) = &node else {
            target.children.push(node);
            continue;
        };

        if item.name != "item" {
            target.children.push(node);
            continue;
        }

        let Some(id) = item.attributes.get("id") else {
            target.children.push(node);
            continue;
        };

        if let Some(existing) = ids.get(id) {
            if existing != &item.attributes {
                return Err(OpfError::ConflictingManifestItem(id.clone()));
            }

            continue;
        }

        ids.insert(id.clone(), item.attributes.clone());
        target.children.push(node);
    }

    Ok(())
}

fn merge_spines(package: &mut Element) -> Result<bool, OpfError> {
    let count = direct_elements(package, "spine").count();

    if count == 0 {
        return Err(OpfError::MissingSpine);
    }

    if count == 1 {
        return Ok(false);
    }

    let mut children = Vec::new();
    let mut retained: Option<Element> = None;

    for node in std::mem::take(&mut package.children) {
        match node {
            XMLNode::Element(element) if element.name == "spine" => {
                if let Some(first) = &mut retained {
                    append_spine_items(first, element);
                } else {
                    retained = Some(element);
                }
            }
            other => children.push(other),
        }
    }

    let spine = retained.expect("spine count was checked");
    insert_at_original_position(&mut children, spine, "spine");

    package.children = children;
    Ok(true)
}

fn append_spine_items(target: &mut Element, source: Element) {
    let mut idrefs: Vec<String> = direct_elements(target, "itemref")
        .filter_map(|item| item.attributes.get("idref").cloned())
        .collect();

    for node in source.children {
        let XMLNode::Element(item) = &node else {
            target.children.push(node);
            continue;
        };

        if item.name != "itemref" {
            target.children.push(node);
            continue;
        }

        if let Some(idref) = item.attributes.get("idref") {
            if idrefs.contains(idref) {
                continue;
            }

            idrefs.push(idref.clone());
        }

        target.children.push(node);
    }
}

fn insert_at_original_position(
    children: &mut Vec<XMLNode>,
    element: Element,
    name: &str,
) {
    let position = children
        .iter()
        .position(|node| {
            matches!(node, XMLNode::Element(existing) if existing.name == name)
        })
        .unwrap_or(children.len());

    children.insert(position, XMLNode::Element(element));
}

fn strip_guides(package: &mut Element) -> bool {
    let original_len = package.children.len();

    package.children.retain(|node| {
        !matches!(node, XMLNode::Element(element) if element.name == "guide")
    });

    package.children.len() != original_len
}

fn find_guide_cover(
    package: &Element,
    opf_path: &str,
    archive_files: &[String],
) -> Result<Option<String>, OpfError> {
    let Some(guide) = direct_elements(package, "guide").next() else {
        return Ok(None);
    };

    for reference in direct_elements(guide, "reference") {
        if reference.attributes.get("type").map(String::as_str) != Some("cover") {
            continue;
        }

        let Some(href) = reference.attributes.get("href") else {
            continue;
        };

        let path = href.split(['#', '?']).next().unwrap_or_default();
        let resolved = resolve_relative_path(opf_path, path)?;

        if archive_files.iter().any(|file| file == &resolved) {
            return Ok(Some(resolved));
        }
    }

    Ok(None)
}

fn resolve_relative_path(
    opf_path: &str,
    href: &str,
) -> Result<String, OpfError> {
    if href.is_empty()
        || href.starts_with('/')
        || href.contains('\\')
        || href.contains('%')
        || href.chars().any(char::is_control)
    {
        return Err(OpfError::InvalidCoverPath);
    }

    let parent = Path::new(opf_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    let mut resolved = PathBuf::from(parent);

    for component in Path::new(href).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir => {
                if !resolved.pop() {
                    return Err(OpfError::InvalidCoverPath);
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(OpfError::InvalidCoverPath);
            }
        }
    }

    let result = resolved
        .to_str()
        .ok_or(OpfError::InvalidCoverPath)?
        .to_owned();

    if result.is_empty() {
        return Err(OpfError::InvalidCoverPath);
    }

    Ok(result)
}

#[cfg(test)]
#[path = "opf_tests.rs"]
mod opf_tests;
