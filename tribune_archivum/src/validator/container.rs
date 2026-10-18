use std::io::Cursor;

use xmltree::{Element, XMLNode};

const ROOTFILE_MEDIA_TYPE: &str = "application/oebps-package+xml";

#[derive(Debug, PartialEq)]
pub enum ContainerError {
    MalformedXml,
    MissingRootfile,
    InvalidRootfilePath,
    MissingPackage(String),
    WriteFailed,
}

#[derive(Debug, PartialEq)]
pub struct ContainerResult {
    pub xml: Vec<u8>,
    pub package_path: String,
    pub repaired: bool,
}

/// Validates and repairs META-INF/container.xml.
///
/// C1: malformed XML is rejected.
/// C2: the first matching, existing package rootfile is retained.
/// C3: its path is normalized and must refer to an archive entry.
pub fn verify_container(
    bytes: &[u8],
    archive_files: &[String],
) -> Result<ContainerResult, ContainerError> {
    let mut container = parse_container(bytes)?;

    let rootfiles = find_rootfiles_mut(&mut container)
        .ok_or(ContainerError::MissingRootfile)?;

    let (selected, package_path) = select_rootfile(rootfiles, archive_files)?;

    let had_other_rootfiles = rootfile_elements(rootfiles).count() != 1;
    keep_only_rootfile(rootfiles, selected);
    let path_changed = set_full_path(rootfiles, &package_path);

    Ok(ContainerResult {
        xml: serialize(&container)?,
        package_path,
        repaired: had_other_rootfiles || path_changed,
    })
}

fn parse_container(bytes: &[u8]) -> Result<Element, ContainerError> {
    Element::parse(Cursor::new(bytes)).map_err(|_| ContainerError::MalformedXml)
}

fn serialize(container: &Element) -> Result<Vec<u8>, ContainerError> {
    let mut output = Vec::new();
    container
        .write(Cursor::new(&mut output))
        .map_err(|_| ContainerError::WriteFailed)?;
    Ok(output)
}

fn find_rootfiles_mut(container: &mut Element) -> Option<&mut Element> {
    container.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(element) if element.name == "rootfiles" => Some(element),
        _ => None,
    })
}

fn is_rootfile(node: &XMLNode) -> bool {
    matches!(node, XMLNode::Element(element) if element.name == "rootfile")
}

fn rootfile_elements(rootfiles: &Element) -> impl Iterator<Item = &Element> {
    rootfiles
        .children
        .iter()
        .filter(|node| is_rootfile(node))
        .filter_map(|node| match node {
            XMLNode::Element(element) => Some(element),
            _ => None,
        })
}

fn keep_only_rootfile(rootfiles: &mut Element, selected: usize) {
    let mut index = 0;
    rootfiles.children.retain(|node| {
        if !is_rootfile(node) {
            return true;
        }
        let keep = index == selected;
        index += 1;
        keep
    });
}

/// Returns true if the attribute had to be changed.
fn set_full_path(rootfiles: &mut Element, package_path: &str) -> bool {
    let Some(XMLNode::Element(rootfile)) = rootfiles
        .children
        .iter_mut()
        .find(|node| is_rootfile(node))
    else {
        return false;
    };

    if rootfile.attributes.get("full-path").map(String::as_str) == Some(package_path) {
        return false;
    }

    rootfile
        .attributes
        .insert("full-path".to_owned(), package_path.to_owned());
    true
}

/// The `full-path` of a rootfile declared as an OPF package, if it has one.
fn package_rootfile_path(rootfile: &Element) -> Option<&String> {
    if rootfile.attributes.get("media-type").map(String::as_str) != Some(ROOTFILE_MEDIA_TYPE) {
        return None;
    }
    rootfile.attributes.get("full-path")
}

fn select_rootfile(
    rootfiles: &Element,
    archive_files: &[String],
) -> Result<(usize, String), ContainerError> {
    let mut first_candidate = None;

    for (index, rootfile) in rootfile_elements(rootfiles).enumerate() {
        let Some(full_path) = package_rootfile_path(rootfile) else {
            continue;
        };

        let normalized = normalize_package_path(full_path)?;

        if !full_path.ends_with(".opf") {
            continue;
        }

        first_candidate.get_or_insert(full_path);

        if archive_files.contains(&normalized) {
            return Ok((index, normalized));
        }
    }

    match first_candidate {
        Some(path) => Err(ContainerError::MissingPackage(path.clone())),
        None => Err(ContainerError::MissingRootfile),
    }
}

/// Normalizes harmless path variations, but rejects traversal,
/// absolute paths, backslashes, and percent-encoding.
fn normalize_package_path(path: &str) -> Result<String, ContainerError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('%')
        || path.chars().any(char::is_control)
    {
        return Err(ContainerError::InvalidRootfilePath);
    }

    let mut components = Vec::new();

    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => return Err(ContainerError::InvalidRootfilePath),
            component => components.push(component),
        }
    }

    if components.is_empty() {
        return Err(ContainerError::InvalidRootfilePath);
    }

    Ok(components.join("/"))
}

#[cfg(test)]
#[path = "container_tests.rs"]
mod container_tests;