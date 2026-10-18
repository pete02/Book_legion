use super::*;
use std::collections::HashMap;
use xmltree::{Element, XMLNode};

fn parse_opf(xml: &str) -> Element {
    Element::parse(xml.as_bytes()).expect("valid test OPF")
}

fn archive(entries: &[(&str, &[u8])]) -> HashMap<String, Vec<u8>> {
    entries.iter().map(|(path, bytes)| ((*path).to_string(), bytes.to_vec())).collect()
}

fn manifest_items(opf: &Element) -> Vec<&Element> {
    opf.children.iter().filter_map(|node| match node {
        XMLNode::Element(element) if element.name == "manifest" => Some(element),
        _ => None,
    }).flat_map(|manifest| manifest.children.iter()).filter_map(|node| match node {
        XMLNode::Element(item) if item.name == "item" => Some(item),
        _ => None,
    }).collect()
}

fn spine_refs(opf: &Element) -> Vec<String> {
    opf.children.iter().filter_map(|node| match node {
        XMLNode::Element(element) if element.name == "spine" => Some(element),
        _ => None,
    }).flat_map(|spine| spine.children.iter()).filter_map(|node| match node {
        XMLNode::Element(itemref) if itemref.name == "itemref" => itemref.attributes.get("idref").cloned(),
        _ => None,
    }).collect()
}

mod errors {
    use super::*;

    #[test]
    fn rejects_non_package_root() {
        let mut opf = parse_opf("<html/>");
        assert_eq!(repair_manifest(&mut opf, "Book.opf", &archive(&[])).unwrap_err(), ManifestError::MissingPackage);
    }

    #[test]
    fn rejects_missing_manifest() {
        let mut opf = parse_opf("<package><metadata/></package>");
        assert_eq!(repair_manifest(&mut opf, "Book.opf", &archive(&[])).unwrap_err(), ManifestError::MissingManifest);
    }

    #[test]
    fn rejects_invalid_opf_path() {
        let mut opf = parse_opf("<package><manifest/></package>");
        assert_eq!(repair_manifest(&mut opf, "../Book.opf", &archive(&[])).unwrap_err(), ManifestError::InvalidOpfPath);
    } 
}

mod item_normalization {
    use super::*;

    #[test]
    fn generates_empty_and_duplicate_ids_without_overwriting_the_first_valid_id() {
        let source = vec![
            parse_opf(r#"<item id="" href="chapter.xhtml" media-type="text/plain"/>"#),
            parse_opf(r#"<item id="same" href="other.xhtml" media-type="application/xhtml+xml"/>"#),
            parse_opf(r#"<item id="same" href="third.xhtml" media-type="application/xhtml+xml"/>"#),
        ];
        let files = archive(&[
            ("chapter.xhtml", b"<html/>"),
            ("other.xhtml", b"<html/>"),
            ("third.xhtml", b"<html/>"),
        ]);

        let (items, _, _, changed) = normalize_items(source, "", &files);
        let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids[0].starts_with("generated-item-"));
        assert_eq!(ids[1], "same");
        assert_ne!(ids[2], "same");
        assert!(changed);
    }

    #[test]
    fn removes_items_with_missing_files_or_unsupported_types() {
        let source = vec![
            parse_opf(r#"<item id="missing" href="missing.xhtml"/>"#),
            parse_opf(r#"<item id="script" href="script.js"/>"#),
        ];
        let files = archive(&[("script.js", b"alert(1)")]);
        let (items, _, _, changed) = normalize_items(source, "", &files);
        assert!(items.is_empty());
        assert!(changed);
    }

    #[test]
    fn normalizes_media_type_and_archive_path_case() {
        let source = vec![parse_opf(r#"<item id="chapter" href="text/CHAPTER.XHTML" media-type="text/plain"/>"#)];
        let files = archive(&[("Text/chapter.xhtml", b"<html/>" )]);
        let (items, _, _, changed) = normalize_items(source, "", &files);
        assert_eq!(items[0].element.attributes.get("href").unwrap(), "Text/chapter.xhtml");
        assert_eq!(items[0].element.attributes.get("media-type").unwrap(), "application/xhtml+xml");
        assert!(changed);
    }
}

mod href_resolution {
    use super::*;

    #[test]
    fn decodes_percent_encoded_paths() {
        let files = archive(&[("chapter one.xhtml", b"<html/>")]);
        assert_eq!(resolve_href("", "chapter%20one.xhtml", &files).as_deref(), Some("chapter one.xhtml"));
    }

    #[test]
    fn rejects_malformed_percent_encoding_and_root_escape() {
        let files = archive(&[("chapter.xhtml", b"<html/>")]);
        assert!(resolve_href("", "chapter%2.xhtml", &files).is_none());
        assert!(resolve_href("", "chapter%GG.xhtml", &files).is_none());
        assert!(resolve_href("", "../chapter.xhtml", &files).is_none());
    }

    #[test]
    fn resolves_parent_segments_that_stay_inside_archive() {
        let files = archive(&[("Text/chapter.xhtml", b"<html/>")]);
        assert_eq!(resolve_href("Text/Chapters", "../../Text/chapter.xhtml", &files).as_deref(), Some("Text/chapter.xhtml"));
    }

    #[test]
    fn content_references_ignore_fragments_and_queries() {
        let files = archive(&[("Text/chapter.xhtml", b"<html/>")]);
        assert_eq!(resolve_resource_href("Text", "chapter.xhtml#section-2", &files).as_deref(), Some("Text/chapter.xhtml"));
        assert_eq!(resolve_resource_href("Text", "chapter.xhtml?mode=1", &files).as_deref(), Some("Text/chapter.xhtml"));
    }

    #[test]
    fn rejects_remote_and_ambiguous_case_insensitive_paths() {
        let files = archive(&[("Text/A.xhtml", b"<html/>"), ("Text/a.xhtml", b"<html/>")]);
        assert!(resolve_href("", "https://example.invalid/a.xhtml", &files).is_none());
        assert!(resolve_href("", "text/a.xhtml", &files).is_none());
    }
}

mod resource_discovery {
    use super::*;

    const CHAPTER: &[u8] = br#"<html xmlns="http://www.w3.org/1999/xhtml"><head><link rel="stylesheet" href="style.css"/></head><body><img src="images/cover.png"/><a href="other.xhtml#part">next</a></body></html>"#;
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nexample";

    #[test]
    fn retains_spine_documents_and_their_referenced_resources() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="text/plain"/>
                </manifest>
            <spine>
                <itemref idref="chapter"/>
            </spine>
        </package>"#);

        let files = archive(&[
            ("Book.opf", b"opf"), ("chapter.xhtml", CHAPTER), ("other.xhtml", b"<html/>"),
            ("style.css", b"body { color: red; }"), ("images/cover.png", PNG),
            ("unused.bin", b"unused"), ("META-INF/container.xml", b"container"), ("mimetype", b"application/epub+zip"),
        ]);

        let report = repair_manifest(&mut opf, "Book.opf", &files).unwrap();
        for path in ["Book.opf", "chapter.xhtml", "other.xhtml", "style.css", "images/cover.png", "META-INF/container.xml", "mimetype"] {
            assert!(report.retained_paths.contains(path), "expected {path} to be retained");
        }
        assert!(report.removed_paths.contains("unused.bin"));
        let listed: HashSet<&str> = manifest_items(&opf).iter().filter_map(|item| item.attributes.get("href").map(String::as_str)).collect();
        assert!(listed.contains("images/cover.png"));
        assert!(listed.contains("style.css"));
    }

    #[test]
    fn merges_manifests_without_changing_spine_entries() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="one" href="one.xhtml"/>
                </manifest>
                <metadata/>
                <manifest>
                    <item id="two" href="two.xhtml"/>
                </manifest>
                <spine>
                    <itemref idref="one"/>
                    <itemref idref="two"/>
                    <itemref idref="missing"/>
                </spine>
            </package>"#);

        let before_spine = spine_refs(&opf);
        let files = archive(&[("Book.opf", b"opf"), ("one.xhtml", b"<html/>"), ("two.xhtml", b"<html/>")]);
        repair_manifest(&mut opf, "Book.opf", &files).unwrap();

        let manifest_count = opf.children.iter().filter(|node| matches!(node, XMLNode::Element(element) if element.name == "manifest")).count();
        assert_eq!(manifest_count, 1);
        assert_eq!(manifest_items(&opf).len(), 2);
        assert_eq!(spine_refs(&opf), before_spine);
    }

    #[test]
    fn drops_remote_and_missing_manifest_items_without_rewriting_spine() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="remote" href="https://example.invalid/book.xhtml"/>
                    <item id="missing" href="missing.xhtml"/>
                    <item id="good" href="chapter.xhtml"/>
                </manifest>
                <spine>
                    <itemref idref="remote"/>
                    <itemref idref="missing"/>
                    <itemref idref="good"/>
                </spine>
            </package>"#);

        let before_spine = spine_refs(&opf);
        let files = archive(&[("Book.opf", b"opf"), ("chapter.xhtml", b"<html/>")]);
        repair_manifest(&mut opf, "Book.opf", &files).unwrap();
        assert_eq!(manifest_items(&opf).len(), 1);
        assert_eq!(spine_refs(&opf), before_spine);
    }

    #[test]
    fn is_idempotent_after_repair() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine>
                    <itemref idref="chapter"/>
                </spine>
            </package>"#);
            
        let files = archive(&[("Book.opf", b"opf"), ("chapter.xhtml", br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="cover.png"/></body></html>"#), ("cover.png", PNG)]);

        let first = repair_manifest(&mut opf, "Book.opf", &files).unwrap();
        assert!(first.changed);
        let second = repair_manifest(&mut opf, "Book.opf", &files).unwrap();
        assert!(!second.changed);
        assert_eq!(first.retained_paths, second.retained_paths);
        assert_eq!(manifest_items(&opf).len(), 2);
    }
}
