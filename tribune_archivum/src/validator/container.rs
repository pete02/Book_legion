
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

    let (selected, package_path) =
        select_rootfile(rootfiles, archive_files)?;

    let original_count = rootfiles
        .children
        .iter()
        .filter(|node| {
            matches!(node, XMLNode::Element(element) if element.name == "rootfile")
        })
        .count();

    let mut repaired = original_count != 1;

    // Remove every rootfile except the selected one.
    let mut index = 0;
    rootfiles.children.retain(|node| {
        let keep = match node {
            XMLNode::Element(element) if element.name == "rootfile" => {
                let keep = index == selected;
                index += 1;
                keep
            }
            _ => true,
        };

        keep
    });

    // Normalize the selected rootfile's path.
    if let Some(XMLNode::Element(rootfile)) = rootfiles
        .children
        .iter_mut()
        .find(|node| matches!(node, XMLNode::Element(element) if element.name == "rootfile"))
    {
        if rootfile.attributes.get("full-path") != Some(&package_path) {
            rootfile
                .attributes
                .insert("full-path".to_owned(), package_path.clone());
            repaired = true;
        }
    }

    let mut output = Vec::new();

    container
        .write(Cursor::new(&mut output))
        .map_err(|_| ContainerError::WriteFailed)?;

    Ok(ContainerResult {
        xml: output,
        package_path,
        repaired,
    })
}

fn parse_container(bytes: &[u8]) -> Result<Element, ContainerError> {
    Element::parse(Cursor::new(bytes))
        .map_err(|_| ContainerError::MalformedXml)
}

fn find_rootfiles_mut(container: &mut Element) -> Option<&mut Element> {
    container
        .children
        .iter_mut()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "rootfiles" => {
                Some(element)
            }
            _ => None,
        })
        .next()
}


fn select_rootfile(
    rootfiles: &Element,
    archive_files: &[String],
) -> Result<(usize, String), ContainerError> {
    let mut rootfile_index = 0;
    let mut found_candidate = false;

    for node in &rootfiles.children {
        let XMLNode::Element(rootfile) = node else {
            continue;
        };

        if rootfile.name != "rootfile" {
            continue;
        }

        let index = rootfile_index;
        rootfile_index += 1;

        if rootfile.attributes.get("media-type").map(String::as_str)
            != Some(ROOTFILE_MEDIA_TYPE)
        {
            continue;
        }

        let Some(full_path) = rootfile.attributes.get("full-path") else {
            continue;
        };

        let normalized = normalize_package_path(full_path)?;
        
        if !full_path.ends_with(".opf") {
            continue;
        }

        found_candidate = true;


        if archive_files.iter().any(|file| file == &normalized) {
            return Ok((index, normalized));
        }
    }

    if found_candidate {
        let path = rootfiles
            .children
            .iter()
            .filter_map(|node| match node {
                XMLNode::Element(element)
                    if element.name == "rootfile"
                        && element.attributes.get("media-type").map(String::as_str)
                            == Some(ROOTFILE_MEDIA_TYPE) =>
                {
                    element.attributes.get("full-path")
                }
                _ => None,
            })
            .find(|path| path.ends_with(".opf"))
            .cloned()
            .unwrap_or_default();

        Err(ContainerError::MissingPackage(path))
    } else {
        Err(ContainerError::MissingRootfile)
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
