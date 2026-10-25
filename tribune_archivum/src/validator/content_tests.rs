use super::*;

const OPF_PATH: &str = "OEBPS/content.opf";
const CHAPTER_PATH: &str = "OEBPS/c1.xhtml";
const SVG_PATH: &str = "OEBPS/art.svg";
const OPF: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0"><manifest><item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/><item id="art" href="art.svg" media-type="image/svg+xml"/></manifest><spine><itemref idref="c1"/></spine></package>"#;
const SVG_NAMESPACE_DECLARATION: &str = r#"xmlns="http://www.w3.org/2000/svg""#;

fn chapter(head: &str, body: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>T</title>{head}</head><body>{body}</body></html>"#
    )
    .into_bytes()
}

fn repair_files(
    files: &[(&str, &[u8])],
) -> (Element, HashMap<String, Vec<u8>>, ContentReport) {
    let mut opf = Element::parse(OPF.as_bytes()).unwrap();
    let mut archive: HashMap<String, Vec<u8>> = files
        .iter()
        .map(|(path, bytes)| (path.to_string(), bytes.to_vec()))
        .collect();
    let report = repair_content_documents(&mut opf, OPF_PATH, &mut archive).unwrap();
    (opf, archive, report)
}

fn repair_chapter(bytes: &[u8]) -> (String, ContentReport) {
    let (_, archive, report) = repair_files(&[(CHAPTER_PATH, bytes)]);
    (String::from_utf8(archive[CHAPTER_PATH].clone()).unwrap(), report)
}

fn repair_svg_file(svg: &[u8]) -> (Element, HashMap<String, Vec<u8>>, ContentReport) {
    repair_files(&[(CHAPTER_PATH, &chapter("", "<p>x</p>")), (SVG_PATH, svg)])
}

mod encoding {
    use super::*;

    #[test]
    fn utf8_bom_is_removed() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(chapter("", "<p>Hello</p>"));

        let (output, report) = repair_chapter(&bytes);

        assert!(!output.starts_with('\u{feff}'));
        assert!(output.contains("<p>Hello</p>"));
        assert_eq!(report.transcoded_documents, vec![CHAPTER_PATH]);
    }

    #[test]
    fn latin1_bytes_are_transcoded_to_utf8() {
        let mut bytes = br#"<?xml version="1.0" encoding="ISO-8859-1"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>T</title></head><body><p>caf"#.to_vec();
        bytes.push(0xE9);
        bytes.extend_from_slice(b"</p></body></html>");

        let (output, report) = repair_chapter(&bytes);

        assert!(output.contains("<p>café</p>"));
        assert_eq!(report.transcoded_documents, vec![CHAPTER_PATH]);
    }

    #[test]
    fn valid_utf8_is_not_reinterpreted_by_a_wrong_declaration() {
        let bytes = chapter("", "<p>café</p>");
        let declared_latin1 = String::from_utf8(bytes)
            .unwrap()
            .replace("UTF-8", "ISO-8859-1")
            .into_bytes();

        let (_, report) = repair_chapter(&declared_latin1);

        assert!(report.transcoded_documents.is_empty());
        assert!(!report.changed);
    }
}

mod well_formedness {
    use super::*;

    #[test]
    fn canonical_document_keeps_its_exact_bytes() {
        let original = chapter("", "<p>Hello</p>");

        let (_, archive, report) = repair_files(&[(CHAPTER_PATH, &original)]);

        assert_eq!(archive[CHAPTER_PATH], original);
        assert!(!report.changed);
    }

    #[test]
    fn named_entities_and_unquoted_attributes_are_repaired_by_html_parse() {
        let bytes = b"<html><body><p class=intro>a&nbsp;b</p></body></html>";

        let (output, report) = repair_chapter(bytes);

        assert!(output.contains(r#"xmlns="http://www.w3.org/1999/xhtml""#));
        assert!(output.contains(r#"class="intro""#));
        assert!(output.contains('\u{a0}'));
        assert_eq!(report.reparsed_documents, vec![CHAPTER_PATH]);
    }

    #[test]
    fn non_html_root_is_rebuilt_as_an_html_document() {
        let (output, report) = repair_chapter(b"<div>text</div>");

        assert!(output.contains("<body><div>text</div></body>"));
        assert_eq!(report.reparsed_documents, vec![CHAPTER_PATH]);
    }

    #[test]
    fn missing_namespace_is_declared_without_reparsing() {
        let bytes = b"<html><head><title>T</title></head><body><p>x</p></body></html>";

        let (output, report) = repair_chapter(bytes);

        assert!(output.contains(r#"xmlns="http://www.w3.org/1999/xhtml""#));
        assert!(report.reparsed_documents.is_empty());
        assert!(report.changed);
    }

    #[test]
    fn loose_content_is_moved_into_a_new_body() {
        let bytes = br#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>T</title></head><p>loose</p></html>"#;

        let (output, _) = repair_chapter(bytes);

        assert!(output.contains("<body><p>loose</p></body>"));
    }

    #[test]
    fn epub_prefixed_attributes_get_their_namespace_declared_after_html_parse() {
        let bytes = br#"<html><body><p epub:type="chapter">x&nbsp;y</p></body></html>"#;

        let (output, _) = repair_chapter(bytes);

        assert!(output.contains(r#"xmlns:epub="http://www.idpf.org/2007/ops""#));
    }

    #[test]
    fn empty_body_is_reported_but_not_rewritten() {
        let original = chapter("", "  ");

        let (_, archive, report) = repair_files(&[(CHAPTER_PATH, &original)]);

        assert_eq!(report.empty_documents, vec![CHAPTER_PATH]);
        assert_eq!(archive[CHAPTER_PATH], original);
    }
}

mod forbidden_elements {
    use super::*;

    #[test]
    fn executable_and_embedding_elements_are_removed() {
        let cases = [
            ("script", "<script>alert(1)</script>"),
            ("iframe", r#"<iframe src="x.html"></iframe>"#),
            ("object", r#"<object data="a.swf"></object>"#),
            ("embed", r#"<embed src="a.swf"/>"#),
            ("applet", r#"<applet code="A"></applet>"#),
        ];

        for (name, markup) in cases {
            let (output, report) = repair_chapter(&chapter("", &format!("<p>a</p>{markup}")));

            assert!(!output.contains(&format!("<{name}")), "{name} survived");
            assert_eq!(report.removed_elements, 1, "{name}");
        }
    }

    #[test]
    fn base_element_is_removed() {
        let head = r#"<base href="http://example.com/"/>"#;

        let (output, _) = repair_chapter(&chapter(head, "<p>a</p>"));

        assert!(!output.contains("<base"));
    }

    #[test]
    fn meta_http_equiv_is_removed_but_meta_charset_is_kept() {
        let head = r#"<meta http-equiv="refresh" content="0"/><meta charset="utf-8"/>"#;

        let (output, _) = repair_chapter(&chapter(head, "<p>a</p>"));

        assert!(!output.contains("http-equiv"));
        assert!(output.contains("charset"));
    }

    #[test]
    fn form_is_unwrapped_and_keeps_its_text() {
        let body = r#"<form action="x"><p>kept</p></form>"#;

        let (output, report) = repair_chapter(&chapter("", body));

        assert!(!output.contains("<form"));
        assert!(output.contains("<p>kept</p>"));
        assert_eq!(report.unwrapped_elements, 1);
    }

    #[test]
    fn video_is_replaced_by_its_fallback_text() {
        let body = r#"<video src="a.mp4"><source src="a.mp4" type="video/mp4"/>No video here.</video>"#;

        let (output, _) = repair_chapter(&chapter("", body));

        assert!(output.contains("No video here."));
        assert!(!output.contains("<video"));
        assert!(!output.contains("<source"));
    }
}

mod forbidden_attributes {
    use super::*;

    #[test]
    fn event_handler_attributes_are_removed() {
        let (output, report) = repair_chapter(&chapter("", r#"<p onclick="x()">t</p>"#));

        assert!(!output.contains("onclick"));
        assert_eq!(report.removed_attributes, 1);
    }

    #[test]
    fn javascript_urls_are_removed_even_when_disguised_with_whitespace() {
        let body = "<a href=\" JaVa&#10;script:alert(1)\">link</a>";

        let (output, _) = repair_chapter(&chapter("", body));

        assert!(!output.contains("href"));
        assert!(output.contains(">link</a>"));
    }

    #[test]
    fn text_attributes_mentioning_javascript_are_kept() {
        let body = r#"<p title="javascript: the good parts">t</p>"#;

        let (output, report) = repair_chapter(&chapter("", body));

        assert!(output.contains("javascript: the good parts"));
        assert!(!report.changed);
    }

    #[test]
    fn srcdoc_is_removed() {
        let (output, _) = repair_chapter(&chapter("", r#"<p srcdoc="x">t</p>"#));

        assert!(!output.contains("srcdoc"));
    }
}

mod inline_css {
    use super::*;

    #[test]
    fn style_element_with_import_is_removed() {
        let head = "<style>@import url(http://example.com/a.css); p { color: red }</style>";

        let (output, _) = repair_chapter(&chapter(head, "<p>a</p>"));

        assert!(!output.contains("<style"));
    }

    #[test]
    fn style_attribute_with_remote_url_is_removed() {
        let body = r#"<p style="background:url(http://example.com/a.png)">t</p>"#;

        let (output, _) = repair_chapter(&chapter("", body));

        assert!(!output.contains("style="));
    }

    #[test]
    fn escaped_expression_is_detected() {
        let body = r#"<p style="width:\65 xpression(alert(1))">t</p>"#;

        let (output, _) = repair_chapter(&chapter("", body));

        assert!(!output.contains("style="));
    }

    #[test]
    fn local_and_data_image_urls_are_kept() {
        let body = r#"<p style="background:url(img/a.png)">a</p><p style="background:url(data:image/png;base64,AAAA)">b</p>"#;

        let (_, report) = repair_chapter(&chapter("", body));

        assert!(!report.changed);
    }
}

mod inline_svg {
    use super::*;

    #[test]
    fn foreign_object_is_removed() {
        let body = format!("<svg {SVG_NAMESPACE_DECLARATION}><foreignObject><p>x</p></foreignObject><rect/></svg>");

        let (output, _) = repair_chapter(&chapter("", &body));

        assert!(!output.contains("foreignObject"));
        assert!(output.contains("<rect"));
    }

    #[test]
    fn image_with_remote_href_is_removed() {
        let body = format!(
            r#"<svg {SVG_NAMESPACE_DECLARATION} xmlns:xlink="http://www.w3.org/1999/xlink"><image xlink:href="http://example.com/a.png"/></svg>"#
        );

        let (output, _) = repair_chapter(&chapter("", &body));

        assert!(!output.contains("example.com"));
    }

    #[test]
    fn link_outside_svg_may_point_at_the_web() {
        let body = r#"<a href="http://example.com/">out</a>"#;

        let (_, report) = repair_chapter(&chapter("", body));

        assert!(!report.changed);
    }
}

mod standalone_svg {
    use super::*;

    #[test]
    fn script_is_removed_from_the_image() {
        let svg = format!("<svg {SVG_NAMESPACE_DECLARATION}><script>x()</script><rect/></svg>");

        let (_, archive, report) = repair_svg_file(svg.as_bytes());

        let output = String::from_utf8(archive[SVG_PATH].clone()).unwrap();
        assert!(!output.contains("script"));
        assert_eq!(report.removed_elements, 1);
    }

    #[test]
    fn clean_svg_keeps_its_exact_bytes() {
        let svg = format!("<svg {SVG_NAMESPACE_DECLARATION}><rect/></svg>");

        let (_, archive, _) = repair_svg_file(svg.as_bytes());

        assert_eq!(archive[SVG_PATH], svg.as_bytes());
    }

    #[test]
    fn unparseable_svg_is_dropped_from_archive_and_manifest() {
        let (opf, archive, report) = repair_svg_file(b"<svg><rect>");

        assert!(!archive.contains_key(SVG_PATH));
        assert_eq!(report.removed_images, vec![SVG_PATH]);
        assert!(!manifest_ids(&opf).contains(&"art".to_owned()));
        assert!(manifest_ids(&opf).contains(&"c1".to_owned()));
    }

    fn manifest_ids(opf: &Element) -> Vec<String> {
        child_element(opf, "manifest")
            .unwrap()
            .children
            .iter()
            .filter_map(|node| match node {
                XMLNode::Element(item) => item.attributes.get("id").cloned(),
                _ => None,
            })
            .collect()
    }
}

mod epub_switch {
    use super::*;

    #[test]
    fn default_branch_replaces_the_switch() {
        let body = r#"<epub:switch xmlns:epub="http://www.idpf.org/2007/ops"><epub:case required-namespace="http://www.w3.org/1998/Math/MathML"><p>math</p></epub:case><epub:default><p>fallback</p></epub:default></epub:switch>"#;

        let (output, report) = repair_chapter(&chapter("", body));

        assert!(output.contains("<p>fallback</p>"));
        assert!(!output.contains("math"));
        assert_eq!(report.resolved_switches, 1);
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn a_repaired_document_is_left_untouched_by_a_second_pass() {
        let dirty = br#"<html><body><p class=a onclick="x()">a&nbsp;b</p><script>1</script></body></html>"#;
        let (_, first_archive, first_report) = repair_files(&[(CHAPTER_PATH, dirty)]);
        assert!(first_report.changed);

        let repaired = first_archive[CHAPTER_PATH].clone();
        let (_, second_archive, second_report) = repair_files(&[(CHAPTER_PATH, &repaired)]);

        assert_eq!(second_archive[CHAPTER_PATH], repaired);
        assert!(!second_report.changed);
    }
}

mod package_errors {
    use super::*;

    #[test]
    fn non_package_root_is_rejected() {
        let mut opf = Element::new("html");
        let mut archive = HashMap::new();

        let result = repair_content_documents(&mut opf, OPF_PATH, &mut archive);

        assert_eq!(result, Err(ContentError::MissingPackage));
    }

    #[test]
    fn missing_manifest_is_rejected() {
        let mut opf = Element::new("package");
        let mut archive = HashMap::new();

        let result = repair_content_documents(&mut opf, OPF_PATH, &mut archive);

        assert_eq!(result, Err(ContentError::MissingManifest));
    }
}