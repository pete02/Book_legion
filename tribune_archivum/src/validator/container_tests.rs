
use super::*;
use xmltree::{Element, XMLNode};

const MEDIA_TYPE: &str = "application/oebps-package+xml";

fn archive_files() -> Vec<String> {
    return vec![
        "EPUB/content.opf".to_owned(),
        "EPUB/other.opf".to_owned(),
    ]
}

fn rootfile(path: &str, media_type: &str) -> String {
    return format!(
        r#"<rootfile full-path="{path}" media-type="{media_type}"/>"#
    )
}

fn document(rootfiles: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0"?>
        <container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"
                   version="1.0">
            <rootfiles>{rootfiles}</rootfiles>
        </container>"#
    )
    .into_bytes()
}

fn verify(xml: &[u8]) -> Result<ContainerResult, ContainerError> {
    verify_container(xml, &archive_files())
}

fn parsed_rootfiles(xml: &[u8]) -> Element {
    let container = Element::parse(xml).unwrap();

    container
        .children
        .iter()
        .find_map(|node| match node {
            XMLNode::Element(element) if element.name == "rootfiles" => {
                Some(element.clone())
            }
            _ => None,
        })
        .unwrap()
}

fn rootfile_elements(rootfiles: &Element) -> Vec<&Element> {
    rootfiles
        .children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "rootfile" => {
                Some(element)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn accepts_single_existing_rootfile() {
    let xml = document(&rootfile("EPUB/content.opf", MEDIA_TYPE));

    let result = verify(&xml).unwrap();

    assert_eq!(result.package_path, "EPUB/content.opf");
    assert!(!result.repaired);

    let rootfiles = parsed_rootfiles(&result.xml);
    assert_eq!(rootfile_elements(&rootfiles).len(), 1);
}

#[test]
fn keeps_first_matching_existing_rootfile() {
    let xml = document(&format!(
        "{}{}{}",
        rootfile("EPUB/missing.opf", MEDIA_TYPE),
        rootfile("EPUB/content.opf", MEDIA_TYPE),
        rootfile("EPUB/other.opf", MEDIA_TYPE),
    ));

    let result = verify(&xml).unwrap();

    assert_eq!(result.package_path, "EPUB/content.opf");
    assert!(result.repaired);

    let rootfiles = parsed_rootfiles(&result.xml);
    let retained = rootfile_elements(&rootfiles);

    assert_eq!(retained.len(), 1);
    assert_eq!(
        retained[0].attributes.get("full-path").unwrap(),
        "EPUB/content.opf"
    );
}

#[test]
fn normalizes_dot_segments_and_repeated_slashes() {
    let xml = document(&rootfile("./EPUB//content.opf", MEDIA_TYPE));

    let result = verify(&xml).unwrap();

    assert_eq!(result.package_path, "EPUB/content.opf");
    assert!(result.repaired);
}

#[test]
fn rejects_malformed_xml() {
    let result = verify(b"<container><rootfiles>");

    assert_eq!(result, Err(ContainerError::MalformedXml));
}

#[test]
fn rejects_parent_directory_traversal() {
    let xml = document(&rootfile("../EPUB/content.opf", MEDIA_TYPE));

    assert_eq!(
        verify(&xml),
        Err(ContainerError::InvalidRootfilePath)
    );
}

#[test]
fn rejects_absolute_paths() {
    let xml = document(&rootfile("/EPUB/content.opf", MEDIA_TYPE));

    assert_eq!(
        verify(&xml),
        Err(ContainerError::InvalidRootfilePath)
    );
}

#[test]
fn rejects_percent_encoding() {
    let xml = document(&rootfile("EPUB/content%2Eopf", MEDIA_TYPE));

    assert_eq!(
        verify(&xml),
        Err(ContainerError::InvalidRootfilePath)
    );
}

#[test]
fn rejects_missing_package_file() {
    let xml = document(&rootfile("EPUB/missing.opf", MEDIA_TYPE));

    assert_eq!(
        verify(&xml),
        Err(ContainerError::MissingPackage(
            "EPUB/missing.opf".to_owned()
        ))
    );
}

#[test]
fn rejects_wrong_media_type() {
    let xml = document(&rootfile(
        "EPUB/content.opf",
        "text/plain",
    ));

    assert_eq!(verify(&xml), Err(ContainerError::MissingRootfile));
}

#[test]
fn rejects_rootfile_without_opf_suffix() {
    let xml = document(&rootfile("EPUB/content.xml", MEDIA_TYPE));

    assert_eq!(verify(&xml), Err(ContainerError::MissingRootfile));
}

#[test]
fn preserves_unrelated_rootfiles_children() {
    let xml = document(&format!(
        "<metadata>keep me</metadata>{}",
        rootfile("EPUB/content.opf", MEDIA_TYPE),
    ));

    let result = verify(&xml).unwrap();
    let rootfiles = parsed_rootfiles(&result.xml);

    assert!(rootfiles.children.iter().any(|node| {
        return matches!(node, XMLNode::Element(element)
            if element.name == "metadata")
    }));
}
