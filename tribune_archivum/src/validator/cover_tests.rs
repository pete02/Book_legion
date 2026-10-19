
use super::*;
use std::collections::HashMap;

fn parse_opf(xml: &str) -> Element {
    Element::parse(xml.as_bytes()).expect("valid test OPF")
}

fn image_bytes(format: ImageFormat, width: u32, height: u32) -> Vec<u8> {
    let image = DynamicImage::ImageRgb8(RgbImage::from_pixel(
        width,
        height,
        Rgb([120, 80, 160]),
    ));
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, format).expect("encode test image");
    output.into_inner()
}

fn archive(entries: &[(&str, Vec<u8>)]) -> HashMap<String, Vec<u8>> {
    entries
        .iter()
        .map(|(path, bytes)| ((*path).to_string(), bytes.clone()))
        .collect()
}

fn chapter() -> Vec<u8> {
    br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><p>Chapter text</p></body></html>"#
        .to_vec()
}

fn basic_opf() -> Element {
    parse_opf(
        r#"<package>
            <metadata>
                <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">Test Book</dc:title>
            </metadata>
            <manifest>
                <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                <item id="art" href="cover.png" media-type="image/png"/>
            </manifest>
            <spine><itemref idref="chapter"/></spine>
        </package>"#,
    )
}

fn basic_archive() -> HashMap<String, Vec<u8>> {
    archive(&[
        ("chapter.xhtml", chapter()),
        ("cover.png", image_bytes(ImageFormat::Png, 300, 450)),
    ])
}

fn manifest_items(opf: &Element) -> Vec<&Element> {
    let manifest = find_elements(opf, "manifest")
        .into_iter()
        .next()
        .expect("manifest exists");

    manifest
        .children
        .iter()
        .filter_map(|node| match node {
            XMLNode::Element(item) if local_name(&item.name) == "item" => Some(item),
            _ => None,
        })
        .collect()
}

fn cover_item(opf: &Element) -> &Element {
    manifest_items(opf)
        .into_iter()
        .find(|item| item.attributes.get("id").map(String::as_str) == Some("cover"))
        .expect("cover manifest item exists")
}

fn archive_cover_bytes<'a>(
    opf: &Element,
    opf_path: &str,
    archive: &'a HashMap<String, Vec<u8>>,
) -> &'a [u8] {
    let href = cover_item(opf)
        .attributes
        .get("href")
        .expect("cover href exists");
    let path = resolve_path(&parent_path(opf_path), href).expect("valid cover href");
    archive.get(&path).expect("cover file exists")
}

fn property_value<'a>(item: &'a Element) -> &'a str {
    item.attributes
        .get("properties")
        .map(String::as_str)
        .unwrap_or("")
}

mod errors {
    use super::*;

    #[test]
    fn rejects_non_package_root() {
        let mut opf = parse_opf("<html/>");
        assert_eq!(
            repair_cover(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            CoverError::MissingPackage
        );
    }

    #[test]
    fn rejects_invalid_opf_path() {
        let mut opf = basic_opf();
        assert_eq!(
            repair_cover(&mut opf, "../Book.opf", &mut basic_archive()).unwrap_err(),
            CoverError::InvalidOpfPath
        );
    }

    #[test]
    fn rejects_missing_manifest() {
        let mut opf = parse_opf("<package><spine/></package>");
        assert_eq!(
            repair_cover(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            CoverError::MissingManifest
        );
    }

    #[test]
    fn rejects_missing_spine() {
        let mut opf = parse_opf("<package><manifest/></package>");
        assert_eq!(
            repair_cover(&mut opf, "Book.opf", &mut archive(&[])).unwrap_err(),
            CoverError::MissingSpine
        );
    }
}

mod manifest_normalization {
    use super::*;

    #[test]
    fn assigns_canonical_cover_id_and_properties() {
        let mut opf = basic_opf();
        let mut files = basic_archive();

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let cover = cover_item(&opf);

        assert_eq!(cover.attributes.get("href").map(String::as_str), Some("cover.png"));
        assert_eq!(
            cover.attributes.get("media-type").map(String::as_str),
            Some("image/png")
        );
        assert_eq!(property_value(cover), "cover-image");
        assert!(report.changed);
        assert_eq!(report.selected_source.as_deref(), Some("cover.png"));
        assert_eq!(report.cover_path.as_deref(), Some("cover.png"));
    }

    #[test]
    fn removes_legacy_cover_metadata_after_using_it() {
        let mut opf = parse_opf(
            r#"<package>
                <metadata><meta name="cover" content="legacy"/></metadata>
                <manifest>
                    <item id="legacy" href="cover.png" media-type="image/png"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = basic_archive();

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("cover.png"));
        assert_eq!(report.removed_cover_metadata, 1);
        assert!(find_elements(&opf, "meta").iter().all(|meta| {
            meta.attributes.get("name").map(String::as_str) != Some("cover")
        }));
    }

    #[test]
    fn removes_cover_image_property_from_other_items() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="selected" href="cover.png" media-type="image/png" properties="cover-image"/>
                    <item id="duplicate" href="other.png" media-type="image/png" properties="cover-image scripted"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("cover.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("other.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("chapter.xhtml", chapter()),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        let duplicate = manifest_items(&opf)
            .into_iter()
            .find(|item| item.attributes.get("id").map(String::as_str) == Some("duplicate"))
            .unwrap();

        assert_eq!(property_value(cover_item(&opf)), "cover-image");
        assert_eq!(property_value(duplicate), "scripted");
        assert_eq!(report.removed_cover_properties, 1);
    }

    #[test]
    fn renames_an_existing_cover_id_before_assigning_the_selected_cover() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="cover" href="old-cover.png" media-type="image/png"/>
                    <item id="selected" href="new-cover.png" media-type="image/png" properties="cover-image"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine>
                    <itemref idref="cover"/>
                    <itemref idref="chapter"/>
                </spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("old-cover.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("new-cover.png", image_bytes(ImageFormat::Png, 500, 700)),
            ("chapter.xhtml", chapter()),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(cover_item(&opf).attributes.get("href").map(String::as_str), Some("new-cover.png"));
        assert_eq!(report.renamed_item_ids, 1);

        let old_cover = manifest_items(&opf)
            .into_iter()
            .find(|item| item.attributes.get("href").map(String::as_str) == Some("old-cover.png"))
            .unwrap();
        let old_id = old_cover.attributes.get("id").unwrap();
        assert_ne!(old_id, "cover");

        let spine_refs = find_elements(&opf, "itemref")
            .into_iter()
            .filter_map(|itemref| itemref.attributes.get("idref").cloned())
            .collect::<Vec<_>>();

        assert_eq!(spine_refs[0], *old_id);
    }
}

mod cover_discovery {
    use super::*;

    #[test]
    fn prefers_cover_image_property_over_legacy_metadata() {
        let mut opf = parse_opf(
            r#"<package>
                <metadata><meta name="cover" content="legacy"/></metadata>
                <manifest>
                    <item id="legacy" href="legacy.png" media-type="image/png"/>
                    <item id="preferred" href="preferred.webp" media-type="image/webp" properties="cover-image"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("legacy.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("preferred.webp", image_bytes(ImageFormat::WebP, 400, 600)),
            ("chapter.xhtml", chapter()),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("preferred.webp"));
        assert_eq!(
            cover_item(&opf).attributes.get("media-type").map(String::as_str),
            Some("image/webp")
        );
    }

    #[test]
    fn follows_legacy_cover_metadata() {
        let mut opf = parse_opf(
            r#"<package>
                <metadata><meta name="cover" content="legacy"/></metadata>
                <manifest>
                    <item id="legacy" href="cover.png" media-type="image/png"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = basic_archive();

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("cover.png"));
        assert_eq!(cover_item(&opf).attributes.get("id").map(String::as_str), Some("cover"));
    }

    #[test]
    fn follows_guide_reference_to_an_image() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="art" href="cover.png" media-type="image/png"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
                <guide><reference type="cover" href="cover.png"/></guide>
            </package>"#,
        );
        let mut files = basic_archive();

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("cover.png"));
    }

    #[test]
    fn follows_guide_xhtml_wrapper_to_its_image() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="wrapper" href="cover.xhtml" media-type="application/xhtml+xml"/>
                    <item id="art" href="cover.png" media-type="image/png"/>
                    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
                <guide><reference type="cover" href="cover.xhtml"/></guide>
            </package>"#,
        );
        let mut files = archive(&[
            (
                "cover.xhtml",
                br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="cover.png"/></body></html>"#.to_vec(),
            ),
            ("cover.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("chapter.xhtml", chapter()),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("cover.png"));
    }

    #[test]
    fn finds_image_referenced_by_first_spine_document() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="chapter" href="Text/chapter.xhtml" media-type="application/xhtml+xml"/>
                    <item id="art" href="Images/cover.png" media-type="image/png"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            (
                "Text/chapter.xhtml",
                br#"<html xmlns="http://www.w3.org/1999/xhtml"><body><img src="../Images/cover.png"/></body></html>"#.to_vec(),
            ),
            ("Images/cover.png", image_bytes(ImageFormat::Png, 300, 450)),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(report.selected_source.as_deref(), Some("Images/cover.png"));
    }
}

mod image_validation {
    use super::*;

    #[test]
    fn accepts_dimensions_at_the_minimum_boundary() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover.png", image_bytes(ImageFormat::Png, 100, 100)),
        ]);

        repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(
            image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files))
                .unwrap()
                .dimensions(),
            (100, 100)
        );
    }

    #[test]
    fn accepts_dimensions_at_the_maximum_boundary() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover.png", image_bytes(ImageFormat::Png, 10_000, 100)),
        ]);

        repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(
            image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files))
                .unwrap()
                .dimensions(),
            (10_000, 100)
        );
    }

    #[test]
    fn replaces_an_image_below_minimum_dimensions() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover.png", image_bytes(ImageFormat::Png, 99, 300)),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let image = image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files)).unwrap();

        assert!(report.generated_placeholder);
        assert!((MIN_DIMENSION..=MAX_DIMENSION).contains(&image.width()));
        assert!((MIN_DIMENSION..=MAX_DIMENSION).contains(&image.height()));
    }

    #[test]
    fn replaces_an_image_above_maximum_dimensions() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover.png", image_bytes(ImageFormat::Png, 10_001, 300)),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let image = image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files)).unwrap();

        assert!(report.generated_placeholder);
        assert!(image.width() <= MAX_DIMENSION);
        assert!(image.height() <= MAX_DIMENSION);
    }

    #[test]
    fn replaces_corrupt_image_with_placeholder() {
        let mut opf = basic_opf();
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover.png", b"not an image".to_vec()),
        ]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert!(report.generated_placeholder);
        assert_eq!(
            cover_item(&opf).attributes.get("media-type").map(String::as_str),
            Some("image/png")
        );
        image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files))
            .expect("generated placeholder is valid");
    }
}

mod placeholder {
    use super::*;

    #[test]
    fn creates_placeholder_when_no_cover_exists() {
        let mut opf = parse_opf(
            r#"<package>
                <metadata>
                    <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">A Test Title</dc:title>
                </metadata>
                <manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/></manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("chapter.xhtml", chapter())]);

        let report = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let image = image::load_from_memory(archive_cover_bytes(&opf, "Book.opf", &files)).unwrap();

        assert!(report.generated_placeholder);
        assert_eq!(cover_item_count(&opf), 1);
        assert_eq!(image.dimensions(), (PLACEHOLDER_WIDTH, PLACEHOLDER_HEIGHT));
        assert_eq!(
            cover_item(&opf).attributes.get("media-type").map(String::as_str),
            Some("image/png")
        );
    }

    #[test]
    fn does_not_overwrite_an_existing_placeholder_path() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/></manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("chapter.xhtml", chapter()),
            ("cover-generated.png", b"existing content".to_vec()),
        ]);

        repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert_eq!(files.get("cover-generated.png").unwrap(), b"existing content");
        assert!(files.contains_key("cover-generated-1.png"));
    }
}

mod nested_opf {
    use super::*;

    #[test]
    fn resolves_cover_paths_relative_to_nested_opf() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest>
                    <item id="art" href="Images/cover.png" media-type="image/png"/>
                    <item id="chapter" href="Text/chapter.xhtml" media-type="application/xhtml+xml"/>
                </manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[
            ("OPS/Images/cover.png", image_bytes(ImageFormat::Png, 300, 450)),
            ("OPS/Text/chapter.xhtml", chapter()),
        ]);

        let report = repair_cover(&mut opf, "OPS/content.opf", &mut files).unwrap();

        assert_eq!(report.cover_path.as_deref(), Some("OPS/Images/cover.png"));
        assert!(files.contains_key("OPS/Images/cover.png"));
        assert_eq!(
            cover_item(&opf).attributes.get("href").map(String::as_str),
            Some("Images/cover.png")
        );
    }

    #[test]
    fn generates_placeholder_inside_nested_opf_directory() {
        let mut opf = parse_opf(
            r#"<package>
                <metadata>
                    <dc:title xmlns:dc="http://purl.org/dc/elements/1.1/">Nested Book</dc:title>
                </metadata>
                <manifest><item id="chapter" href="Text/chapter.xhtml" media-type="application/xhtml+xml"/></manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("OPS/Text/chapter.xhtml", chapter())]);

        let report = repair_cover(&mut opf, "OPS/content.opf", &mut files).unwrap();

        assert_eq!(report.cover_path.as_deref(), Some("OPS/cover-generated.png"));
        assert!(files.contains_key("OPS/cover-generated.png"));
        assert_eq!(
            cover_item(&opf).attributes.get("href").map(String::as_str),
            Some("cover-generated.png")
        );
    }
}

mod idempotence {
    use super::*;

    #[test]
    fn repairing_a_valid_cover_twice_makes_no_second_change() {
        let mut opf = basic_opf();
        let mut files = basic_archive();

        repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let first_image = files.get("cover.png").unwrap().clone();

        let second = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert!(!second.changed);
        assert_eq!(files.get("cover.png").unwrap(), &first_image);
        assert_eq!(
            manifest_items(&opf)
                .into_iter()
                .filter(|item| item.attributes.get("id").map(String::as_str) == Some("cover"))
                .count(),
            1
        );
    }

    #[test]
    fn repairing_a_generated_placeholder_twice_makes_no_second_change() {
        let mut opf = parse_opf(
            r#"<package>
                <manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/></manifest>
                <spine><itemref idref="chapter"/></spine>
            </package>"#,
        );
        let mut files = archive(&[("chapter.xhtml", chapter())]);

        repair_cover(&mut opf, "Book.opf", &mut files).unwrap();
        let path = resolve_path(
            "",
            cover_item(&opf).attributes.get("href").unwrap(),
        )
        .unwrap();
        let first_image = files.get(&path).unwrap().clone();

        let second = repair_cover(&mut opf, "Book.opf", &mut files).unwrap();

        assert!(!second.changed);
        assert_eq!(files.get(&path).unwrap(), &first_image);
    }
}
