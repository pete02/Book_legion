use super::*;

const OPF_PATH: &str = "OEBPS/content.opf";
const FIRST_PATH: &str = "OEBPS/c1.xhtml";
const NESTED_PATH: &str = "OEBPS/text/c3.xhtml";
const OPF: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0"><manifest><item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/><item id="c2" href="c2.xhtml" media-type="application/xhtml+xml"/><item id="c3" href="text/c3.xhtml" media-type="application/xhtml+xml"/><item id="css" href="style.css" media-type="text/css"/><item id="a" href="img/a.png" media-type="image/png"/><item id="pic" href="img/my pic.png" media-type="image/png"/><item id="art" href="art.svg" media-type="image/svg+xml"/></manifest><spine><itemref idref="c1"/><itemref idref="c2"/><itemref idref="c3"/></spine></package>"#;

fn page(body: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>T</title></head><body>{body}</body></html>"#
    )
    .into_bytes()
}

fn book(first_body: &str, nested_body: &str) -> HashMap<String, Vec<u8>> {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#.to_vec();
    HashMap::from([
        (FIRST_PATH.to_owned(), page(first_body)),
        ("OEBPS/c2.xhtml".to_owned(), page("<p>two</p>")),
        (NESTED_PATH.to_owned(), page(nested_body)),
        ("OEBPS/style.css".to_owned(), b"p{}".to_vec()),
        ("OEBPS/img/a.png".to_owned(), b"png".to_vec()),
        ("OEBPS/img/my pic.png".to_owned(), b"png".to_vec()),
        ("OEBPS/art.svg".to_owned(), svg),
    ])
}

fn repair(
    first_body: &str,
    nested_body: &str,
) -> (HashMap<String, Vec<u8>>, ReferenceReport) {
    let opf = Element::parse(OPF.as_bytes()).unwrap();
    let mut archive = book(first_body, nested_body);
    let report = repair_references(&opf, OPF_PATH, &mut archive).unwrap();
    (archive, report)
}

fn repair_first(body: &str) -> (String, ReferenceReport) {
    let (archive, report) = repair(body, "<p>three</p>");
    (String::from_utf8(archive[FIRST_PATH].clone()).unwrap(), report)
}

fn repair_nested(body: &str) -> (String, ReferenceReport) {
    let (archive, report) = repair("<p>one</p>", body);
    (String::from_utf8(archive[NESTED_PATH].clone()).unwrap(), report)
}

mod kept_resources {
    use super::*;

    #[test]
    fn canonical_references_leave_the_document_byte_identical() {
        let body = r##"<img src="img/a.png"/><link href="style.css"/><a href="#top">t</a>"##;
        let original = page(body);

        let (archive, report) = repair(body, "<p>three</p>");

        assert_eq!(archive[FIRST_PATH], original);
        assert!(!report.changed);
    }

    #[test]
    fn data_uri_for_an_allowed_image_is_kept() {
        let (_, report) = repair_first(r#"<img src="data:image/png;base64,AAAA"/>"#);

        assert!(!report.changed);
    }

    #[test]
    fn references_are_resolved_from_the_referencing_documents_directory() {
        let (_, report) = repair_nested(r#"<img src="../img/a.png"/><a href="../c2.xhtml">t</a>"#);

        assert!(!report.changed);
    }
}

mod canonicalized_references {
    use super::*;

    #[test]
    fn dot_segments_are_collapsed() {
        let (output, report) = repair_first(r#"<img src="./img/../img/a.png"/>"#);

        assert!(output.contains(r#"src="img/a.png""#));
        assert_eq!(report.rewritten_references, 1);
    }

    #[test]
    fn case_mismatch_is_fixed() {
        let (output, _) = repair_first(r#"<img src="IMG/A.PNG"/>"#);

        assert!(output.contains(r#"src="img/a.png""#));
    }

    #[test]
    fn names_with_spaces_are_percent_encoded() {
        let (output, _) = repair_first(r#"<img src="img/my pic.png"/>"#);

        assert!(output.contains(r#"src="img/my%20pic.png""#));
    }

    #[test]
    fn percent_encoded_names_resolve_to_the_decoded_entry() {
        let (_, report) = repair_first(r#"<img src="img/my%20pic.png"/>"#);

        assert!(!report.changed);
    }

    #[test]
    fn query_is_dropped_and_fragment_is_kept_on_links() {
        let (output, _) = repair_first(r#"<a href="./c2.xhtml?x=1#part">t</a>"#);

        assert!(output.contains(r##"href="c2.xhtml#part""##));
    }
}

mod removed_resources {
    use super::*;

    #[test]
    fn remote_image_is_replaced_by_its_alt_text() {
        let (output, report) =
            repair_first(r#"<p><img src="http://example.com/a.png" alt="A cat"/></p>"#);

        assert!(output.contains("<p>A cat</p>"));
        assert!(!output.contains("<img"));
        assert_eq!(report.removed_resource_references, 1);
    }

    #[test]
    fn image_without_alt_text_is_removed_entirely() {
        let (output, _) = repair_first(r#"<p>a<img src="https://example.com/a.png"/>b</p>"#);

        assert!(output.contains("<p>ab</p>"));
    }

    #[test]
    fn protocol_relative_and_absolute_paths_are_removed() {
        let body = r#"<img src="//example.com/a.png"/><img src="/img/a.png"/>"#;

        let (output, report) = repair_first(body);

        assert!(!output.contains("<img"));
        assert_eq!(report.removed_resource_references, 2);
    }

    #[test]
    fn dangling_reference_is_removed() {
        let (output, _) = repair_first(r#"<img src="img/missing.png"/>"#);

        assert!(!output.contains("<img"));
    }

    #[test]
    fn reference_escaping_the_archive_root_is_removed() {
        let (output, _) = repair_first(r#"<img src="../../../a.png"/>"#);

        assert!(!output.contains("<img"));
    }

    #[test]
    fn data_uri_for_a_non_image_is_removed() {
        let (output, _) = repair_first(r#"<img src="data:text/html;base64,AAAA"/>"#);

        assert!(!output.contains("<img"));
    }

    #[test]
    fn image_pointing_at_a_stylesheet_is_removed() {
        let (output, _) = repair_first(r#"<img src="style.css"/>"#);

        assert!(!output.contains("<img"));
    }

    #[test]
    fn image_without_src_is_removed() {
        let (output, _) = repair_first(r#"<img alt=""/>"#);

        assert!(!output.contains("<img"));
    }

    #[test]
    fn remote_stylesheet_link_is_removed() {
        let body = r#"<link rel="stylesheet" href="http://example.com/a.css"/><p>x</p>"#;

        let (output, _) = repair_first(body);

        assert!(!output.contains("<link"));
        assert!(output.contains("<p>x</p>"));
    }

    #[test]
    fn link_element_pointing_at_a_non_stylesheet_is_removed() {
        let (output, _) = repair_first(r#"<link rel="icon" href="img/a.png"/>"#);

        assert!(!output.contains("<link"));
    }

    #[test]
    fn srcset_is_removed_but_src_is_kept() {
        let (output, report) =
            repair_first(r#"<img src="img/a.png" srcset="http://example.com/a.png 2x"/>"#);

        assert!(!output.contains("srcset"));
        assert!(output.contains(r#"src="img/a.png""#));
        assert_eq!(report.removed_srcsets, 1);
    }

    #[test]
    fn dangling_svg_image_is_removed() {
        let body = r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="missing.png"/><rect/></svg>"#;

        let (output, _) = repair_first(body);

        assert!(!output.contains("<image"));
        assert!(output.contains("<rect"));
    }
}

mod links {
    use super::*;

    #[test]
    fn link_to_a_spine_item_is_kept_with_its_fragment() {
        let (_, report) = repair_first(r##"<a href="c2.xhtml#part">t</a>"##);

        assert!(!report.changed);
    }

    #[test]
    fn link_to_a_manifest_item_outside_the_spine_loses_its_href() {
        let (output, report) = repair_first(r#"<a href="img/a.png">pic</a>"#);

        assert!(output.contains("<a>pic</a>"));
        assert_eq!(report.removed_link_targets, 1);
    }

    #[test]
    fn dangling_link_loses_its_href_but_keeps_its_text() {
        let (output, _) = repair_first(r#"<a href="gone.xhtml">text</a>"#);

        assert!(output.contains("<a>text</a>"));
    }

    #[test]
    fn web_and_mail_links_are_left_alone() {
        let body = r#"<a href="http://example.com/">a</a><a href="https://example.com/">b</a><a href="mailto:me@example.com">c</a>"#;

        let (_, report) = repair_first(body);

        assert!(!report.changed);
    }

    #[test]
    fn other_schemes_are_removed() {
        let body = r#"<a href="ftp://example.com/f">a</a><a href="tel:123">b</a><a href="//example.com/">c</a>"#;

        let (output, report) = repair_first(body);

        assert!(!output.contains("href"));
        assert_eq!(report.removed_link_targets, 3);
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn a_repaired_document_is_left_untouched_by_a_second_pass() {
        let body = r#"<img src="./IMG/my pic.png"/><img src="http://example.com/a.png" alt="x"/><a href="gone.xhtml">t</a>"#;
        let opf = Element::parse(OPF.as_bytes()).unwrap();
        let mut archive = book(body, "<p>three</p>");
        repair_references(&opf, OPF_PATH, &mut archive).unwrap();
        let repaired = archive[FIRST_PATH].clone();

        let report = repair_references(&opf, OPF_PATH, &mut archive).unwrap();

        assert_eq!(archive[FIRST_PATH], repaired);
        assert!(!report.changed);
    }
}

mod errors {
    use super::*;

    #[test]
    fn non_package_root_is_rejected() {
        let result = repair_references(&Element::new("html"), OPF_PATH, &mut HashMap::new());

        assert_eq!(result, Err(ReferenceError::MissingPackage));
    }

    #[test]
    fn missing_manifest_is_rejected() {
        let result = repair_references(&Element::new("package"), OPF_PATH, &mut HashMap::new());

        assert_eq!(result, Err(ReferenceError::MissingManifest));
    }

    #[test]
    fn malformed_document_is_reported_by_path() {
        let opf = Element::parse(OPF.as_bytes()).unwrap();
        let mut archive = book("<p>one</p>", "<p>three</p>");
        archive.insert(FIRST_PATH.to_owned(), b"<html><body>".to_vec());

        let result = repair_references(&opf, OPF_PATH, &mut archive);

        assert_eq!(
            result,
            Err(ReferenceError::MalformedDocument(FIRST_PATH.to_owned()))
        );
    }
}