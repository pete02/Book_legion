use super::*;

const OPF_PATH: &str = "OEBPS/content.opf";
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const COMPLETE: &str = r#"<dc:title>A Tale</dc:title><dc:creator>Ann</dc:creator><dc:creator>Bob</dc:creator><dc:language>en</dc:language><dc:identifier id="id">urn:uuid:1</dc:identifier>"#;
const TWO_CHAPTERS: &str = r#"<itemref idref="a"/><itemref idref="b"/>"#;

fn package(metadata: &str, spine: &str) -> Element {
    let xml = format!(
        r#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/">{metadata}</metadata><manifest/><spine>{spine}</spine></package>"#
    );
    Element::parse(xml.as_bytes()).unwrap()
}

fn extract(opf: &mut Element) -> MetadataResult {
    repair_metadata(opf, OPF_PATH, b"abc").unwrap()
}

fn extract_from(metadata: &str) -> MetadataResult {
    extract(&mut package(metadata, TWO_CHAPTERS))
}

fn first_text(opf: &Element, name: &str) -> Option<String> {
    child_element(opf, "metadata")
        .and_then(|metadata| dc_elements(metadata, name).next())
        .map(element_text)
}

fn round_trip(opf: &Element) -> Element {
    let mut bytes = Vec::new();
    opf.write(&mut bytes).unwrap();
    Element::parse(bytes.as_slice()).unwrap()
}

mod complete_metadata {
    use super::*;

    #[test]
    fn every_field_is_returned() {
        let result = extract_from(COMPLETE);

        assert_eq!(
            result.metadata,
            LibraryMetadata {
                title: "A Tale".to_owned(),
                authors: vec!["Ann".to_owned(), "Bob".to_owned()],
                language: "en".to_owned(),
                identifier: "urn:uuid:1".to_owned(),
                series: None,
                series_index: None,
                file_hash: ABC_SHA256.to_owned(),
                chapter_count: 2,
                profile_version: PROFILE_VERSION,
                opf_path: OPF_PATH.to_owned(),
            }
        );
    }

    #[test]
    fn a_complete_opf_is_not_modified() {
        let mut opf = package(COMPLETE, TWO_CHAPTERS);
        let original = opf.clone();

        let result = extract(&mut opf);

        assert!(!result.report.changed);
        assert_eq!(opf, original);
    }

    #[test]
    fn opf_path_is_normalized() {
        let result = repair_metadata(
            &mut package(COMPLETE, TWO_CHAPTERS),
            "./OEBPS/../OEBPS/content.opf",
            b"abc",
        )
        .unwrap();

        assert_eq!(result.metadata.opf_path, OPF_PATH);
    }
}

mod title {
    use super::*;

    #[test]
    fn whitespace_is_collapsed() {
        let result = extract_from("<dc:title>  A   Tale\n of Two </dc:title>");

        assert_eq!(result.metadata.title, "A Tale of Two");
    }

    #[test]
    fn blank_titles_are_removed_and_the_next_one_is_used() {
        let mut opf = package("<dc:title> </dc:title><dc:title>Real</dc:title>", TWO_CHAPTERS);

        let result = extract(&mut opf);

        assert_eq!(result.metadata.title, "Real");
        assert_eq!(first_text(&opf, "title"), Some("Real".to_owned()));
        assert!(result.report.changed);
    }

    #[test]
    fn missing_title_becomes_untitled() {
        let mut opf = package("", TWO_CHAPTERS);

        let result = extract(&mut opf);

        assert_eq!(result.metadata.title, "Untitled");
        assert_eq!(first_text(&opf, "title"), Some("Untitled".to_owned()));
        assert!(result.report.inserted_title);
    }
}

mod language {
    use super::*;

    #[test]
    fn tags_are_normalized_to_bcp47_case() {
        let cases = [
            ("en", "en"),
            ("EN", "en"),
            ("en_us", "en-US"),
            ("zh-hant-tw", "zh-Hant-TW"),
            (" fi ", "fi"),
            ("es-419", "es-419"),
        ];

        for (raw, expected) in cases {
            let result = extract_from(&format!("<dc:language>{raw}</dc:language>"));

            assert_eq!(result.metadata.language, expected, "{raw}");
        }
    }

    #[test]
    fn normalized_value_is_written_back() {
        let mut opf = package("<dc:language>en_us</dc:language>", TWO_CHAPTERS);

        let result = extract(&mut opf);

        assert_eq!(first_text(&opf, "language"), Some("en-US".to_owned()));
        assert!(result.report.repaired_language);
    }

    #[test]
    fn unknown_values_become_und() {
        for raw in ["English", "", "e", "en--US", "en-toolongsubtag"] {
            let mut opf = package(&format!("<dc:language>{raw}</dc:language>"), TWO_CHAPTERS);

            let result = extract(&mut opf);

            assert_eq!(result.metadata.language, "und", "{raw:?}");
            assert_eq!(first_text(&opf, "language"), Some("und".to_owned()), "{raw:?}");
        }
    }

    #[test]
    fn missing_language_is_inserted_as_und() {
        let mut opf = package("", TWO_CHAPTERS);

        let result = extract(&mut opf);

        assert_eq!(result.metadata.language, "und");
        assert_eq!(first_text(&opf, "language"), Some("und".to_owned()));
    }
}

mod identifier {
    use super::*;

    #[test]
    fn the_unique_identifier_element_is_used() {
        let metadata = r#"<dc:identifier id="other">urn:other</dc:identifier><dc:identifier id="id"> urn:main </dc:identifier>"#;

        let result = extract_from(metadata);

        assert_eq!(result.metadata.identifier, "urn:main");
        assert!(!result.report.repaired_identifier);
    }

    #[test]
    fn dangling_unique_identifier_points_at_the_first_identifier() {
        let mut opf = package(r#"<dc:identifier id="real">urn:real</dc:identifier>"#, TWO_CHAPTERS);
        opf.attributes
            .insert("unique-identifier".to_owned(), "missing".to_owned());

        let result = extract(&mut opf);

        assert_eq!(result.metadata.identifier, "urn:real");
        assert_eq!(opf.attributes.get("unique-identifier").unwrap(), "real");
    }

    #[test]
    fn identifier_without_an_id_is_given_one() {
        let mut opf = package("<dc:identifier>urn:x</dc:identifier>", TWO_CHAPTERS);
        opf.attributes.remove("unique-identifier");

        let result = extract(&mut opf);

        assert_eq!(result.metadata.identifier, "urn:x");
        assert_eq!(opf.attributes.get("unique-identifier").unwrap(), "pub-id");
        let element = dc_elements(child_element(&opf, "metadata").unwrap(), "identifier")
            .next()
            .unwrap();
        assert_eq!(element.attributes.get("id").unwrap(), "pub-id");
    }

    #[test]
    fn blank_identifiers_are_ignored() {
        let result = extract_from(r#"<dc:identifier id="id">  </dc:identifier>"#);

        assert_eq!(result.metadata.identifier, format!("urn:sha256:{ABC_SHA256}"));
    }

    #[test]
    fn missing_identifier_is_derived_from_the_file_hash() {
        let mut opf = package("", TWO_CHAPTERS);

        let result = extract(&mut opf);

        assert_eq!(result.metadata.identifier, format!("urn:sha256:{ABC_SHA256}"));
        assert_eq!(opf.attributes.get("unique-identifier").unwrap(), "pub-id");
        assert!(result.report.repaired_identifier);
    }

    #[test]
    fn generated_id_does_not_collide_with_an_existing_one() {
        let mut opf = package(r#"<dc:creator id="pub-id">Ann</dc:creator>"#, TWO_CHAPTERS);

        extract(&mut opf);

        assert_eq!(opf.attributes.get("unique-identifier").unwrap(), "pub-id-2");
    }
}

mod authors {
    use super::*;

    #[test]
    fn blank_creators_are_skipped_and_names_are_trimmed() {
        let result = extract_from("<dc:creator> Ann  Lee </dc:creator><dc:creator> </dc:creator>");

        assert_eq!(result.metadata.authors, vec!["Ann Lee"]);
    }

    #[test]
    fn no_creator_gives_no_authors() {
        assert!(extract_from("").metadata.authors.is_empty());
    }
}

mod series {
    use super::*;

    #[test]
    fn epub3_collection_with_a_series_type_is_read() {
        let metadata = r##"<meta property="belongs-to-collection" id="c1">Saga</meta><meta refines="#c1" property="collection-type">series</meta><meta refines="#c1" property="group-position">2</meta>"##;

        let result = extract_from(metadata);

        assert_eq!(result.metadata.series.as_deref(), Some("Saga"));
        assert_eq!(result.metadata.series_index, Some(2.0));
    }

    #[test]
    fn collection_without_a_type_counts_as_a_series() {
        let metadata = r#"<meta property="belongs-to-collection" id="c1">Saga</meta>"#;

        let result = extract_from(metadata);

        assert_eq!(result.metadata.series.as_deref(), Some("Saga"));
        assert_eq!(result.metadata.series_index, None);
    }

    #[test]
    fn collection_of_another_type_is_not_a_series() {
        let metadata = r##"<meta property="belongs-to-collection" id="c1">Box</meta><meta refines="#c1" property="collection-type">set</meta>"##;

        assert_eq!(extract_from(metadata).metadata.series, None);
    }

    #[test]
    fn calibre_series_is_read_with_a_fractional_index() {
        let metadata = r#"<meta name="calibre:series" content="Saga"/><meta name="calibre:series_index" content="1.5"/>"#;

        let result = extract_from(metadata);

        assert_eq!(result.metadata.series.as_deref(), Some("Saga"));
        assert_eq!(result.metadata.series_index, Some(1.5));
    }

    #[test]
    fn no_series_metadata_gives_none() {
        let result = extract_from(COMPLETE);

        assert_eq!(result.metadata.series, None);
        assert_eq!(result.metadata.series_index, None);
    }
}

mod missing_metadata_element {
    use super::*;

    fn package_without_metadata() -> Element {
        Element::parse(
            br#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0"><manifest/><spine><itemref idref="a"/></spine></package>"#
                .as_slice(),
        )
        .unwrap()
    }

    #[test]
    fn metadata_is_created_with_the_required_fields() {
        let mut opf = package_without_metadata();

        let result = extract(&mut opf);

        assert_eq!(result.metadata.title, "Untitled");
        assert_eq!(result.metadata.language, "und");
        assert_eq!(result.metadata.chapter_count, 1);
        assert!(result.report.changed);
    }

    #[test]
    fn inserted_elements_keep_the_opf_well_formed_without_a_dc_declaration() {
        let mut opf = package_without_metadata();
        extract(&mut opf);

        let reparsed = round_trip(&opf);

        assert_eq!(first_text(&reparsed, "title"), Some("Untitled".to_owned()));
    }

    #[test]
    fn inserted_elements_keep_the_opf_well_formed_with_a_dc_declaration() {
        let mut opf = package("", TWO_CHAPTERS);
        extract(&mut opf);

        let reparsed = round_trip(&opf);

        assert_eq!(first_text(&reparsed, "language"), Some("und".to_owned()));
    }
}

mod chapters {
    use super::*;

    #[test]
    fn chapter_count_is_the_number_of_spine_items() {
        let spine = r#"<itemref idref="a"/><itemref idref="b"/><itemref idref="c"/>"#;

        let result = extract(&mut package(COMPLETE, spine));

        assert_eq!(result.metadata.chapter_count, 3);
    }

    #[test]
    fn missing_spine_is_rejected() {
        let mut opf = Element::parse(
            br#"<package xmlns="http://www.idpf.org/2007/opf"><metadata/></package>"#.as_slice(),
        )
        .unwrap();

        let result = repair_metadata(&mut opf, OPF_PATH, b"abc");

        assert_eq!(result, Err(MetadataError::MissingSpine));
    }
}

mod errors {
    use super::*;

    #[test]
    fn non_package_root_is_rejected() {
        let result = repair_metadata(&mut Element::new("html"), OPF_PATH, b"abc");

        assert_eq!(result, Err(MetadataError::MissingPackage));
    }

    #[test]
    fn invalid_opf_path_is_rejected() {
        let result = repair_metadata(&mut package(COMPLETE, TWO_CHAPTERS), "/abs/content.opf", b"abc");

        assert_eq!(result, Err(MetadataError::InvalidOpfPath));
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn a_second_run_returns_the_same_metadata_without_changes() {
        let mut opf = package("<dc:language>EN_us</dc:language><dc:identifier>urn:x</dc:identifier>", TWO_CHAPTERS);
        opf.attributes.remove("unique-identifier");
        let first = extract(&mut opf);
        assert!(first.report.changed);

        let second = extract(&mut opf);

        assert_eq!(second.metadata, first.metadata);
        assert!(!second.report.changed);
    }

    #[test]
    fn a_run_on_a_reparsed_opf_changes_nothing() {
        let mut opf = package("", TWO_CHAPTERS);
        let first = extract(&mut opf);
        let mut reparsed = round_trip(&opf);

        let second = extract(&mut reparsed);

        assert_eq!(second.metadata, first.metadata);
        assert!(!second.report.changed);
    }
}