use super::*;

const OPF_PATH: &str = "OEBPS/content.opf";
const STYLE_PATH: &str = "OEBPS/style.css";
const NESTED_STYLE_PATH: &str = "OEBPS/css/s.css";
const OPF: &str = r#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0"><manifest><item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/><item id="css" href="style.css" media-type="text/css"/><item id="css2" href="css/s.css" media-type="text/css"/><item id="a" href="img/a.png" media-type="image/png"/><item id="font" href="fonts/f.woff2" media-type="font/woff2"/></manifest><spine><itemref idref="c1"/></spine></package>"#;

fn repair_files(files: &[(&str, &[u8])]) -> (HashMap<String, Vec<u8>>, CssReport) {
    let opf = Element::parse(OPF.as_bytes()).unwrap();
    let mut archive: HashMap<String, Vec<u8>> = files
        .iter()
        .map(|(path, bytes)| (path.to_string(), bytes.to_vec()))
        .chain([
            ("OEBPS/c1.xhtml".to_owned(), b"<html/>".to_vec()),
            ("OEBPS/img/a.png".to_owned(), b"png".to_vec()),
            ("OEBPS/fonts/f.woff2".to_owned(), b"wOF2".to_vec()),
        ])
        .collect();
    let report = repair_css(&opf, OPF_PATH, &mut archive).unwrap();
    (archive, report)
}

fn repair_bytes(css: &[u8]) -> (String, CssReport) {
    let (archive, report) = repair_files(&[(STYLE_PATH, css)]);
    (String::from_utf8(archive[STYLE_PATH].clone()).unwrap(), report)
}

fn repair(css: &str) -> (String, CssReport) {
    repair_bytes(css.as_bytes())
}

fn assert_untouched(css: &str) {
    let (output, report) = repair(css);

    assert_eq!(output, css);
    assert!(!report.changed);
}

mod syntax_repair {
    use super::*;

    #[test]
    fn valid_stylesheet_keeps_its_exact_bytes() {
        assert_untouched("body { margin: 0; }\n@media print { p { color: black } }\n");
    }

    #[test]
    fn unterminated_comment_is_dropped() {
        let (output, report) = repair("p{color:red} /* never closed");

        assert_eq!(output, "p{color:red}\n");
        assert_eq!(report.edits.repaired_syntax, 1);
    }

    #[test]
    fn unclosed_block_is_closed() {
        let (output, _) = repair("p{color:red");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn stray_closing_brace_is_dropped() {
        let (output, _) = repair("} p{color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn invalid_declarations_are_dropped() {
        let (output, report) = repair("p{color:red;;bogus;*zoom:1;margin:0}");

        assert_eq!(output, "p{color:red;margin:0}\n");
        assert_eq!(report.edits.removed_declarations, 2);
    }

    #[test]
    fn rule_left_without_declarations_is_removed() {
        let (output, _) = repair("p{} a{color:red}");

        assert_eq!(output, "a{color:red}\n");
    }

    #[test]
    fn braces_inside_strings_do_not_end_the_rule() {
        assert_untouched("p:after{content:\"}{\"}\n");
    }
}

mod encoding {
    use super::*;

    #[test]
    fn bom_is_removed() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"p{color:red}");

        let (output, report) = repair_bytes(&bytes);

        assert!(!output.starts_with('\u{feff}'));
        assert_eq!(report.transcoded_stylesheets, vec![STYLE_PATH]);
    }

    #[test]
    fn declared_latin1_is_transcoded_and_charset_rule_removed() {
        let mut bytes = b"@charset \"iso-8859-1\"; p:after{content:\"caf".to_vec();
        bytes.push(0xE9);
        bytes.extend_from_slice(b"\"}");

        let (output, report) = repair_bytes(&bytes);

        assert!(output.contains("caf\u{e9}"));
        assert!(!output.contains("@charset"));
        assert_eq!(report.transcoded_stylesheets, vec![STYLE_PATH]);
    }
}

mod at_rules {
    use super::*;

    #[test]
    fn only_font_face_media_page_and_supports_are_kept() {
        let css = "@import \"a.css\"; @namespace x url(http://example.com/); @keyframes k{from{a:b}} @font-face{font-family:F;src:url(fonts/f.woff2)} @media print{p{color:red}} @page{margin:1cm} @supports (display:grid){p{display:grid}}";

        let (output, report) = repair(css);

        for kept in ["@font-face", "@media print", "@page", "@supports (display:grid)"] {
            assert!(output.contains(kept), "{kept} was removed");
        }
        for removed in ["@import", "@namespace", "@keyframes"] {
            assert!(!output.contains(removed), "{removed} survived");
        }
        assert_eq!(report.edits.removed_at_rules, 3);
    }

    #[test]
    fn font_face_without_a_source_is_removed() {
        let (output, _) = repair("@font-face{font-family:F} p{color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn media_rule_with_nothing_left_is_removed() {
        let (output, _) = repair("@media print{p{}} a{color:red}");

        assert_eq!(output, "a{color:red}\n");
    }

    #[test]
    fn nested_margin_rules_inside_page_are_removed() {
        let (output, report) = repair("@page{margin:1cm;@top-left{content:\"x\"}}");

        assert_eq!(output, "@page{margin:1cm}\n");
        assert_eq!(report.edits.removed_rules, 1);
    }
}

mod forbidden_constructs {
    use super::*;

    #[test]
    fn expression_declaration_is_dropped() {
        let (output, _) = repair("p{width:expression(alert(1));color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn escaped_expression_is_dropped() {
        let (output, _) = repair("p{width:\\65 xpression(alert(1));color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn behavior_and_binding_properties_are_dropped() {
        let (output, _) = repair("p{behavior:url(x.htc);-moz-binding:url(x.xml);color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn scroll_behavior_is_not_mistaken_for_behavior() {
        assert_untouched("p{scroll-behavior:smooth}\n");
    }

    #[test]
    fn url_function_spelled_with_escapes_is_dropped() {
        let (output, _) = repair("p{background:\\75rl(http://example.com/a.png);color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn image_set_is_dropped() {
        let (output, _) = repair("p{background:image-set(\"http://example.com/a.png\" 1x);color:red}");

        assert_eq!(output, "p{color:red}\n");
    }
}

mod urls {
    use super::*;

    #[test]
    fn remote_and_protocol_relative_urls_drop_their_declaration() {
        let css = "p{background:url(http://example.com/a.png);color:red} a{background:url(//example.com/a.png);color:blue}";

        let (output, _) = repair(css);

        assert_eq!(output, "p{color:red}\na{color:blue}\n");
    }

    #[test]
    fn dangling_url_drops_its_declaration() {
        let (output, _) = repair("p{background:url(img/missing.png);color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn url_to_a_non_resource_drops_its_declaration() {
        let (output, _) = repair("p{background:url(c1.xhtml);color:red}");

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn local_image_url_is_kept_as_written() {
        assert_untouched("p{background:url(img/a.png)}\n");
    }

    #[test]
    fn fragment_only_and_data_image_urls_are_kept() {
        assert_untouched("p{filter:url(#blur)} a{background:url(data:image/png;base64,AAAA)}\n");
    }

    #[test]
    fn non_canonical_url_is_rewritten() {
        let (output, report) = repair("p{background:url('./IMG/A.png')}");

        assert_eq!(output, "p{background:url(\"img/a.png\")}\n");
        assert_eq!(report.edits.rewritten_urls, 1);
    }

    #[test]
    fn urls_are_resolved_from_the_stylesheets_directory() {
        let css = b"p{background:url(../img/a.png)} a{background:url(img/a.png)}";

        let (archive, _) = repair_files(&[(NESTED_STYLE_PATH, css)]);

        let output = String::from_utf8(archive[NESTED_STYLE_PATH].clone()).unwrap();
        assert_eq!(output, "p{background:url(../img/a.png)}\n");
    }

    #[test]
    fn url_inside_a_string_is_not_a_reference() {
        assert_untouched("p:after{content:\"url(http://example.com/)\"}\n");
    }
}

mod font_sources {
    use super::*;

    #[test]
    fn only_the_failing_sources_are_dropped() {
        let css = "@font-face{font-family:F;src:url(http://example.com/f.woff2),url(fonts/f.woff2) format(\"woff2\")}";

        let (output, report) = repair(css);

        assert_eq!(
            output,
            "@font-face{font-family:F;src:url(fonts/f.woff2) format(\"woff2\")}\n"
        );
        assert_eq!(report.edits.removed_font_sources, 1);
    }

    #[test]
    fn font_face_with_only_remote_sources_is_removed() {
        let css = "@font-face{font-family:F;src:url(http://example.com/f.woff2)} p{color:red}";

        let (output, _) = repair(css);

        assert_eq!(output, "p{color:red}\n");
    }

    #[test]
    fn data_font_source_is_kept() {
        assert_untouched("@font-face{font-family:F;src:url(data:font/woff2;base64,AAAA)}\n");
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn a_repaired_stylesheet_is_left_untouched_by_a_second_pass() {
        let css = "@import 'x.css'; p{background:url('./IMG/A.png');width:expression(1)} } /* open";
        let opf = Element::parse(OPF.as_bytes()).unwrap();
        let (mut archive, first) = repair_files(&[(STYLE_PATH, css.as_bytes())]);
        assert!(first.changed);
        let repaired = archive[STYLE_PATH].clone();

        let second = repair_css(&opf, OPF_PATH, &mut archive).unwrap();

        assert_eq!(archive[STYLE_PATH], repaired);
        assert!(!second.changed);
    }
}

mod errors {
    use super::*;

    #[test]
    fn non_package_root_is_rejected() {
        let result = repair_css(&Element::new("html"), OPF_PATH, &mut HashMap::new());

        assert_eq!(result, Err(CssError::MissingPackage));
    }

    #[test]
    fn missing_manifest_is_reported_as_an_invalid_package() {
        let result = repair_css(&Element::new("package"), OPF_PATH, &mut HashMap::new());

        assert_eq!(
            result,
            Err(CssError::InvalidPackage(ReferenceError::MissingManifest))
        );
    }
}