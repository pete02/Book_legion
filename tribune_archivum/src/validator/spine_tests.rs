use super::*;

fn parse_opf(manifest_ids: &[&str], spine: &str) -> Element {
    let items: String = manifest_ids
        .iter()
        .map(|id| {
            format!(r#"<item id="{id}" href="{id}.xhtml" media-type="application/xhtml+xml"/>"#)
        })
        .collect();
    let xml = format!("<package><manifest>{items}</manifest>{spine}</package>");
    Element::parse(xml.as_bytes()).unwrap()
}

fn spine_of(opf: &Element) -> &Element {
    child_elements(opf, "spine").next().unwrap()
}

fn idrefs(opf: &Element) -> Vec<&str> {
    child_elements(spine_of(opf), "itemref")
        .map(idref)
        .collect()
}

fn any_linear_attribute(opf: &Element) -> bool {
    child_elements(spine_of(opf), "itemref")
        .any(|itemref| itemref.attributes.contains_key("linear"))
}

mod whole_repair {
    use super::*;

    #[test]
    fn valid_spine_is_left_unchanged() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"<spine>
                <itemref idref="a"/>
                <itemref idref="b"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report, SpineReport::default());
        assert_eq!(idrefs(&opf), ["a", "b"]);
    }

    #[test]
    fn repairing_twice_changes_nothing_the_second_time() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"<spine>
                <itemref idref="a" linear="no"/>
                <itemref idref="ghost"/>
                <itemref idref="a"/>
                <itemref idref="b"/>
            </spine>"#,
        );

        assert!(repair_spine(&mut opf).unwrap().changed);
        assert_eq!(repair_spine(&mut opf).unwrap(), SpineReport::default());
    }

    #[test]
    fn spine_attributes_are_preserved() {
        let mut opf = parse_opf(
            &["a"],
            r#"
        <spine toc="ncx">
            <itemref idref="a"/>
        </spine>"#,
        );

        repair_spine(&mut opf).unwrap();

        assert_eq!(
            spine_of(&opf).attributes.get("toc").map(String::as_str),
            Some("ncx")
        );
    }
}

mod s1_at_least_one_itemref {
    use super::*;

    #[test]
    fn rejects_missing_spine() {
        let mut opf = parse_opf(&["a"], "");

        assert_eq!(repair_spine(&mut opf), Err(SpineError::MissingSpine));
    }

    #[test]
    fn rejects_spine_without_itemrefs() {
        let mut opf = parse_opf(&["a"], "<spine/>");

        assert_eq!(repair_spine(&mut opf), Err(SpineError::EmptySpine));
    }

    #[test]
    fn rejects_spine_whose_itemrefs_all_dangle() {
        let mut opf = parse_opf(
            &["a"],
            r#"
        <spine>
            <itemref idref="ghost"/>
        </spine>"#,
        );

        assert_eq!(repair_spine(&mut opf), Err(SpineError::EmptySpine));
    }

    #[test]
    fn accepts_a_single_itemref() {
        let mut opf = parse_opf(
            &["a"],
            r#"
            <spine>
                <itemref idref="a"/>
            </spine>"#,
        );

        assert!(repair_spine(&mut opf).is_ok());
    }
}

mod s2_dangling_idrefs {
    use super::*;

    #[test]
    fn drops_itemrefs_missing_from_the_manifest() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"
            <spine>
                <itemref idref="a"/>
                <itemref idref="ghost"/>
                <itemref idref="b"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.dangling_idrefs, ["ghost"]);
        assert!(report.changed);
        assert_eq!(idrefs(&opf), ["a", "b"]);
    }

    #[test]
    fn drops_itemref_without_idref() {
        let mut opf = parse_opf(
            &["a"],
            r#"
        <spine>
            <itemref/>
            <itemref idref="a"/>
        </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.dangling_idrefs, [""]);
        assert_eq!(idrefs(&opf), ["a"]);
    }
}

mod s3_non_linear_items {
    use super::*;

    #[test]
    fn removes_linear_no_and_reports_it() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"
            <spine>
                <itemref idref="a" linear="no"/>
                <itemref idref="b"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.non_linear_idrefs, ["a"]);
        assert!(report.changed);
        assert!(!any_linear_attribute(&opf));
        assert_eq!(idrefs(&opf), ["a", "b"]);
    }

    #[test]
    fn leaves_linear_yes_alone() {
        let mut opf = parse_opf(
            &["a"],
            r#"
        <spine>
            <itemref idref="a" linear="yes"/>
        </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report, SpineReport::default());
        assert!(any_linear_attribute(&opf));
    }
}

mod s4_duplicate_idrefs {
    use super::*;

    #[test]
    fn keeps_the_first_of_duplicate_idrefs() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"
            <spine>
                <itemref idref="a"/>
                <itemref idref="b"/>
                <itemref idref="a"/>
                <itemref idref="a"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.duplicate_idrefs, ["a", "a"]);
        assert_eq!(idrefs(&opf), ["a", "b"]);
    }
}

mod rule_interaction {
    use super::*;

    #[test]
    fn first_duplicate_is_kept_even_when_it_is_non_linear() {
        let mut opf = parse_opf(
            &["a", "b"],
            r#"
            <spine>
                <itemref idref="a" linear="no"/>
                <itemref idref="b"/>
                <itemref idref="a"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.duplicate_idrefs, ["a"]);
        assert_eq!(report.non_linear_idrefs, ["a"]);
        assert_eq!(idrefs(&opf), ["a", "b"]);
        assert!(!any_linear_attribute(&opf));
    }

    #[test]
    fn dropped_duplicate_is_not_reported_as_non_linear() {
        let mut opf = parse_opf(
            &["a"],
            r#"
            <spine>
                <itemref idref="a"/>
                <itemref idref="a" linear="no"/>
            </spine>"#,
        );

        let report = repair_spine(&mut opf).unwrap();

        assert_eq!(report.duplicate_idrefs, ["a"]);
        assert!(report.non_linear_idrefs.is_empty());
    }
}
