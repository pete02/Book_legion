
use super::*;
use xmltree::{Element, XMLNode};

const MANIFEST_ITEM: &str =
    r#"<item id="chapter" href="chapter.xhtml"
           media-type="application/xhtml+xml"/>"#;

const SPINE_ITEM: &str = r#"<itemref idref="chapter"/>"#;

fn opf(manifest: &str, spine: &str, guide: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0"?>
        <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
            <metadata/>
            {manifest}
            {spine}
            {guide}
        </package>"#
    )
    .into_bytes()
}

fn archive_files() -> Vec<String> {
    vec![
        "Books/book.opf".to_owned(),
        "Books/cover.xhtml".to_owned(),
        "Books/chapter.xhtml".to_owned(),
    ]
}

fn verify(xml: &[u8]) -> Result<OpfResult, OpfError> {
    verify_opf(xml, "Books/book.opf", &archive_files())
}

fn parse(xml: &[u8]) -> Element {
    Element::parse(Cursor::new(xml)).unwrap()
}

fn elements<'a>(parent: &'a Element, name: &str) -> Vec<&'a Element> {
    parent.children.iter().filter_map(|node| match node {
        XMLNode::Element(element) if element.name == name => Some(element),
        _ => None,
    }).collect()
}

#[test]
fn accepts_valid_package() {
    let xml = opf(
        &format!("<manifest>{MANIFEST_ITEM}</manifest>"),
        &format!("<spine>{SPINE_ITEM}</spine>"),
        "",
    );

    let result = verify(&xml).unwrap();

    assert_eq!(result.opf_path, "Books/book.opf");
    assert_eq!(result.cover_path, None);
    assert!(!result.repaired);
}

#[test]
fn rejects_malformed_xml() {
    assert_eq!(
        verify(b"<package><manifest>"),
        Err(OpfError::MalformedXml)
    );
}

#[test]
fn rejects_wrong_root_element() {
    assert_eq!(
        verify(b"<container/>"),
        Err(OpfError::InvalidPackageRoot)
    );
}

#[test]
fn rejects_missing_manifest() {
    let xml = opf(
        "",
        "<spine/>",
        "",
    );

    assert_eq!(verify(&xml), Err(OpfError::MissingManifest));
}

#[test]
fn rejects_missing_spine() {
    let xml = opf(
        "<manifest/>",
        "",
        "",
    );

    assert_eq!(verify(&xml), Err(OpfError::MissingSpine));
}

#[test]
fn merges_manifests() {
    let xml = opf(
        &format!(
            "<manifest>{MANIFEST_ITEM}</manifest>
             <manifest>
                 <item id=\"cover\" href=\"cover.xhtml\"
                       media-type=\"application/xhtml+xml\"/>
             </manifest>"
        ),
        "<spine/>",
        "",
    );

    let result = verify(&xml).unwrap();
    let package = parse(&result.xml);

    assert_eq!(elements(&package, "manifest").len(), 1);
    assert_eq!(elements(elements(&package, "manifest")[0], "item").len(), 2);
    assert!(result.repaired);
}

#[test]
fn removes_duplicate_manifest_items_from_later_manifests() {
    let xml = opf(
        &format!(
            "<manifest>{MANIFEST_ITEM}</manifest>
             <manifest>{MANIFEST_ITEM}</manifest>"
        ),
        "<spine/>",
        "",
    );

    let result = verify(&xml).unwrap();
    let package = parse(&result.xml);

    assert_eq!(elements(&package, "manifest").len(), 1);
    assert_eq!(elements(elements(&package, "manifest")[0], "item").len(), 1);
}

#[test]
fn rejects_conflicting_manifest_ids() {
    let xml = opf(
        r#"<manifest>
            <item id="chapter" href="a.xhtml" media-type="application/xhtml+xml"/>
        </manifest>
        <manifest>
            <item id="chapter" href="b.xhtml" media-type="application/xhtml+xml"/>
        </manifest>"#,
        "<spine/>",
        "",
    );

    assert_eq!(
        verify(&xml),
        Err(OpfError::ConflictingManifestItem("chapter".to_owned()))
    );
}

#[test]
fn merges_spines_and_preserves_order() {
    let xml = opf(
        "<manifest/>",
        r#"<spine><itemref idref="first"/></spine>
           <spine><itemref idref="second"/></spine>"#,
        "",
    );

    let result = verify(&xml).unwrap();
    let package = parse(&result.xml);
    let spines = elements(&package, "spine");

    assert_eq!(spines.len(), 1);

    let ids: Vec<_> = elements(spines[0], "itemref")
        .iter()
        .filter_map(|item| item.attributes.get("idref"))
        .map(String::as_str)
        .collect();

    assert_eq!(ids, vec!["first", "second"]);
}

#[test]
fn locates_cover_before_stripping_guide() {
    let xml = opf(
        "<manifest/>",
        "<spine/>",
        r#"<guide>
            <reference type="cover" href="cover.xhtml"/>
        </guide>"#,
    );

    let result = verify(&xml).unwrap();

    assert_eq!(result.cover_path.as_deref(), Some("Books/cover.xhtml"));

    let package = parse(&result.xml);
    assert!(elements(&package, "guide").is_empty());
    assert!(result.repaired);
}

#[test]
fn resolves_cover_relative_to_opf_location() {
    let xml = opf(
        "<manifest/>",
        "<spine/>",
        r#"<guide>
            <reference type="cover" href="../Books/cover.xhtml"/>
        </guide>"#,
    );

    let result = verify(&xml).unwrap();

    assert_eq!(result.cover_path.as_deref(), Some("Books/cover.xhtml"));
}

#[test]
fn leaves_guide_absent_when_no_cover_is_declared() {
    let xml = opf(
        "<manifest/>",
        "<spine/>",
        "<guide/>",
    );

    let result = verify(&xml).unwrap();

    assert_eq!(result.cover_path, None);
    assert!(result.repaired);
}
