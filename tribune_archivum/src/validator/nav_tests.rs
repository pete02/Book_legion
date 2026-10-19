
use super::*;
use std::collections::HashMap;
use xmltree::{Element, XMLNode};

fn parse_opf(xml: &str) -> Element {
    Element::parse(xml.as_bytes()).expect("valid test OPF")
}

fn archive(entries: &[(&str, &[u8])]) -> HashMap<String, Vec<u8>> {
    entries
        .iter()
        .map(|(path, bytes)| ((*path).to_string(), bytes.to_vec()))
        .collect()
}

fn chapter(title: &str) -> Vec<u8> {
    format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title></head><body><h1>{title}</h1><p>Chapter text.</p></body></html>"#
    )
    .into_bytes()
}

fn nav_document(xml: &str) -> Vec<u8> {
    xml.as_bytes().to_vec()
}

fn manifest_items(opf: &Element) -> Vec<&Element> {
    opf.children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "manifest" => Some(element),
            _ => None,
        })
        .flat_map(|manifest| manifest.children.iter())
        .filter_map(|node| match node {
            XMLNode::Element(item) if item.name == "item" => Some(item),
            _ => None,
        })
        .collect()
}

fn spine_refs(opf: &Element) -> Vec<String> {
    opf.children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(element) if element.name == "spine" => Some(element),
            _ => None,
        })
        .flat_map(|spine| spine.children.iter())
        .filter_map(|node| match node {
            XMLNode::Element(itemref) if itemref.name == "itemref" => {
                itemref.attributes.get("idref").cloned()
            }
            _ => None,
        })
        .collect()
}

fn parse_archive_xml(files: &HashMap<String, Vec<u8>>, path: &str) -> Element {
    Element::parse(
        files
            .get(path)
            .unwrap_or_else(|| panic!("missing archive entry: {path}"))
            .as_slice(),
    )
    .unwrap_or_else(|_| panic!("invalid XML archive entry: {path}"))
}

fn toc_nav(document: &Element) -> &Element {
    find_elements(document, "nav")
        .into_iter()
        .find(|nav| is_toc(nav))
        .expect("navigation document should contain a TOC nav")
}

fn toc_entries(document: &Element) -> Vec<TocEntry> {
    extract_toc(document)
}

fn basic_opf() -> Element {
    parse_opf(
        r#"<package>
            <manifest>
                <item id="chapter-1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
                <item id="chapter-2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
                <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
            </manifest>
            <spine>
                <itemref idref="chapter-1"/>
                <itemref idref="chapter-2"/>
            </spine>
        </package>"#,
    )
}

fn basic_archive() -> HashMap<String, Vec<u8>> {
    archive(&[
        ("chapter1.xhtml", &chapter("First Chapter")),
        ("chapter2.xhtml", &chapter("Second Chapter")),
        (
            "nav.xhtml",
            &nav_document(
                r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="chapter1.xhtml">First Chapter</a></li><li><a href="chapter2.xhtml">Second Chapter</a></li></ol></nav></body></html>"#,
            ),
        ),
    ])
}

mod errors {
    use super::*;

    #[test]
    fn rejects_non_package_root() {
        let mut opf = parse_opf("<html/>");
        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            NavigationError::MissingPackage
        );
    }

    #[test]
    fn rejects_missing_manifest() {
        let mut opf = parse_opf("<package><spine/></package>");
        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            NavigationError::MissingManifest
        );
    }

    #[test]
    fn rejects_missing_spine() {
        let mut opf = parse_opf("<package><manifest/></package>");
        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            NavigationError::MissingSpine
        );
    }

    #[test]
    fn rejects_invalid_opf_path() {
        let mut opf = basic_opf();
        assert_eq!(
            repair_navigation(&mut opf, "../Book.opf", &mut basic_archive()).unwrap_err(),
            NavigationError::InvalidOpfPath
        );
    }

    #[test]
    fn rejects_empty_spine() {
        let mut opf = parse_opf(
            r#"
            <package>
                <manifest/>
                <spine/>
            </package>"#,
        );
        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            NavigationError::EmptySpine
        );
    }

    #[test]
    fn rejects_malformed_existing_nav_xml() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert("nav.xhtml".into(), b"<html><body>".to_vec());

        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut files).unwrap_err(),
            NavigationError::InvalidNavXml
        );
    }

    #[test]
    fn rejects_duplicate_nav_ids() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                    <item id="nav" href="other-nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("chapter.xhtml", &chapter("Chapter"))]);

        assert_eq!(
            repair_navigation(&mut opf, "Book.opf", &mut files).unwrap_err(),
            NavigationError::ConflictingNavigationItems
        );
    }
}

mod nav_item {
    use super::*;

    #[test]
    fn creates_missing_nav_manifest_item_and_document() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("chapter.xhtml", &chapter("Chapter One"))]);

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav_items: Vec<_> = manifest_items(&opf)
            .into_iter()
            .filter(|item| item.attributes.get("id").map(String::as_str) == Some("nav"))
            .collect();

        assert_eq!(nav_items.len(), 1);
        assert_eq!(
            nav_items[0].attributes.get("properties").map(String::as_str),
            Some("nav")
        );
        assert_eq!(
            nav_items[0].attributes.get("media-type").map(String::as_str),
            Some("application/xhtml+xml")
        );
        assert!(files.contains_key("nav.xhtml"));
        assert!(report.changed);
        assert!(report.synthesized);
    }

    #[test]
    fn repairs_nav_item_properties_and_media_type() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="text/plain" properties="cover"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("chapter.xhtml", &chapter("Chapter One")),
            (
                "nav.xhtml",
                &nav_document(
                    r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml">Chapter One</a></li></ol></nav></body></html>"#,
                ),
            ),
        ]);

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = manifest_items(&opf)
            .into_iter()
            .find(|item| item.attributes.get("id").map(String::as_str) == Some("nav"))
            .unwrap();

        assert_eq!(
            nav.attributes.get("properties").map(String::as_str),
            Some("nav")
        );
        assert_eq!(
            nav.attributes.get("media-type").map(String::as_str),
            Some("application/xhtml+xml")
        );
    }
}

mod spine {
    use super::*;

    #[test]
    fn removes_nav_itemref_without_changing_chapter_order() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter-1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                    <item id="chapter-2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine>
                    <itemref idref="chapter-1"/>
                    <itemref idref="nav"/>
                    <itemref idref="chapter-2"/>
                </spine>
            </package>"#,
        );
        let mut files = basic_archive();

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(spine_refs(&opf), ["chapter-1", "chapter-2"]);
        assert_eq!(report.removed_spine_refs, 1);
    }

    #[test]
    fn removes_ncx_spine_attribute_and_ncx_itemref() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
                </manifest>
                <spine toc="ncx">
                    <itemref idref="chapter"/>
                    <itemref idref="ncx"/>
                </spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("chapter.xhtml", &chapter("Chapter One")),
            (
                "nav.xhtml",
                &nav_document(
                    r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc">
                            <ol>
                                <li><a href="chapter.xhtml">Chapter One</a></li>
                            </ol>
                        </nav></body></html>"#,
                ),
            ),
            ("toc.ncx", b"<ncx/>"),
        ]);

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let spine = opf.children.iter().find_map(|node| match node {
            XMLNode::Element(element) if element.name == "spine" => Some(element),
            _ => None,
        }).unwrap();

        assert!(!spine.attributes.contains_key("toc"));
        assert_eq!(spine_refs(&opf), ["chapter"]);
    }
}

mod toc_normalization {
    use super::*;

    #[test]
    fn keeps_only_one_toc_and_removes_other_nav_elements() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
                    <nav epub:type="landmarks"><ol><li><a href="chapter1.xhtml">Landmark</a></li></ol></nav>
                    <nav epub:type="toc"><ol><li><a href="chapter1.xhtml">First Chapter</a></li></ol></nav>
                    <nav epub:type="page-list"><ol><li><a href="chapter2.xhtml">Page</a></li></ol></nav>
                </body></html>"#,
            ),
        );

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let navs = find_elements(&nav, "nav");

        assert_eq!(navs.len(), 1);
        assert!(is_toc(navs[0]));
    }

    #[test]
    fn drops_toc_entries_that_do_not_target_spine_documents() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>
                    <li><a href="chapter1.xhtml">First Chapter</a></li>
                    <li><a href="extra.xhtml">Extra</a></li>
                    <li><a href="https://example.invalid/book.xhtml">Remote</a></li>
                </ol></nav></body></html>"#,
            ),
        );

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].href, "chapter1.xhtml");
        assert_eq!(report.removed_entries, 2);
    }

    #[test]
    fn replaces_empty_labels_with_document_headings() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>
                    <li><a href="chapter1.xhtml">   </a></li>
                    <li><a href="chapter2.xhtml">Second Chapter</a></li>
                </ol></nav></body></html>"#,
            ),
        );

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert_eq!(entries[0].label, "First Chapter");
        assert_eq!(report.repaired_labels, 1);
    }

    #[test]
    fn preserves_nested_toc_entries() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>
                    <li><a href="chapter1.xhtml">First Chapter</a><ol>
                        <li><a href="chapter2.xhtml">Second Chapter</a></li>
                    </ol></li>
                </ol></nav></body></html>"#,
            ),
        );

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].children.len(), 1);
        assert_eq!(entries[0].children[0].label, "Second Chapter");
    }

    #[test]
    fn removes_fragment_only_when_resolving_the_target() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>
                    <li><a href="chapter1.xhtml#section-1">First Chapter</a></li>
                </ol></nav></body></html>"#,
            ),
        );

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        assert_eq!(toc_entries(&nav)[0].href, "chapter1.xhtml#section-1");
    }
}

mod ncx_conversion {
    use super::*;

    fn ncx() -> &'static [u8] {
        br#"<ncx><navMap>
            <navPoint id="p1">
                <navLabel><text>Part One</text></navLabel>
                <content src="chapter1.xhtml"/>
                <navPoint id="p2">
                    <navLabel><text>Part Two</text></navLabel>
                    <content src="chapter2.xhtml"/>
                </navPoint>
            </navPoint>
        </navMap></ncx>"#
    }

    #[test]
    fn converts_nested_ncx_entries_into_toc() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter-1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
                    <item id="chapter-2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
                </manifest>
                <spine toc="ncx">
                    <itemref idref="chapter-1"/>
                    <itemref idref="chapter-2"/>
                </spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("chapter1.xhtml", &chapter("First Chapter")),
            ("chapter2.xhtml", &chapter("Second Chapter")),
            ("toc.ncx", ncx()),
        ]);

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert!(report.converted_ncx);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Part One");
        assert_eq!(entries[0].children.len(), 1);
        assert_eq!(entries[0].children[0].label, "Part Two");
        assert!(!files.contains_key("toc.ncx"));
        assert!(!manifest_items(&opf).iter().any(|item| {
            item.attributes.get("media-type").map(String::as_str)
                == Some("application/x-dtbncx+xml")
        }));
    }

    #[test]
    fn removes_ncx_when_a_valid_toc_already_exists() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
                    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
                </manifest>
                <spine toc="ncx"><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("chapter.xhtml", &chapter("Chapter One")),
            (
                "nav.xhtml",
                &nav_document(
                    r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml">Existing TOC</a></li></ol></nav></body></html>"#,
                ),
            ),
            ("toc.ncx", ncx()),
        ]);

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        assert_eq!(toc_entries(&nav)[0].label, "Existing TOC");
        assert_eq!(report.removed_ncx, 1);
        assert!(!files.contains_key("toc.ncx"));
    }
}

mod synthesis {
    use super::*;

    #[test]
    fn synthesizes_toc_from_spine_when_nav_document_is_missing() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter1.xhtml", &chapter("First Chapter")),
            ("chapter2.xhtml", &chapter("Second Chapter")),
        ]);

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert!(report.synthesized);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "First Chapter");
        assert_eq!(entries[1].label, "Second Chapter");
    }

    #[test]
    fn uses_chapter_number_when_document_has_no_heading() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[(
            "chapter.xhtml",
            br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p>Text only.</p></body></html>"#,
        )]);

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        assert_eq!(toc_entries(&nav)[0].label, "Chapter 1");
    }

    #[test]
    fn synthesizes_toc_when_existing_toc_has_no_valid_entries() {
        let mut opf = basic_opf();
        let mut files = basic_archive();
        files.insert(
            "nav.xhtml".into(),
            nav_document(
                r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>
                    <li><a href="missing.xhtml">Missing</a></li>
                </ol></nav></body></html>"#,
            ),
        );

        let report = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        let nav = parse_archive_xml(&files, "nav.xhtml");
        let entries = toc_entries(&nav);

        assert!(report.synthesized);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "First Chapter");
        assert_eq!(entries[1].label, "Second Chapter");
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn second_repair_makes_no_changes() {
        let mut opf = basic_opf();
        let mut files = basic_archive();

        let first = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();
        let nav_after_first = files.get("nav.xhtml").unwrap().clone();
        let second = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        assert!(first.changed);
        assert!(!second.changed);
        assert_eq!(files.get("nav.xhtml").unwrap(), &nav_after_first);
    }

    #[test]
    fn synthesized_toc_is_idempotent() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("chapter.xhtml", &chapter("Chapter One"))]);

        repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();
        let nav_after_first = files.get("nav.xhtml").unwrap().clone();

        let second = repair_navigation(&mut opf, "Book.opf", &mut files).unwrap();

        assert!(!second.changed);
        assert_eq!(files.get("nav.xhtml").unwrap(), &nav_after_first);
    }
}
