
use std::collections::HashMap;
use std::io::Cursor;

use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use xmltree::{Element, XMLNode};

const MIN_DIMENSION: u32 = 100;
const MAX_DIMENSION: u32 = 10_000;
const PLACEHOLDER_WIDTH: u32 = 600;
const PLACEHOLDER_HEIGHT: u32 = 900;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverError {
    MissingPackage,
    InvalidOpfPath,
    MissingManifest,
    MissingSpine,
    InvalidXml,
    SerializationFailed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoverReport {
    pub changed: bool,
    pub generated_placeholder: bool,
    pub selected_source: Option<String>,
    pub cover_path: Option<String>,
    pub reencoded: bool,
    pub renamed_item_ids: usize,
    pub removed_cover_metadata: usize,
    pub removed_cover_properties: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestItem {
    id: String,
    href: String,
    path: String,
    media_type: String,
    properties: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoverSource {
    CoverImageProperty,
    LegacyMetadata,
    Guide,
    FirstSpineImage,
    Placeholder,
}

struct SelectedCover {
    item: ManifestItem,
    bytes: Vec<u8>,
    media_type: String,
    source: CoverSource,
    reencoded: bool,
}

pub fn repair_cover(
    opf: &mut Element,
    opf_path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
) -> Result<CoverReport, CoverError> {
    validate_package(opf)?;
    let opf_path = normalize_path(opf_path).ok_or(CoverError::InvalidOpfPath)?;
    let opf_dir = parent_path(&opf_path);
    let manifest_index =direct_child_index(opf, "manifest").ok_or(CoverError::MissingManifest)?;
    let spine_index = direct_child_index(opf, "spine").ok_or(CoverError::MissingSpine)?;
    let items = read_manifest(opf, manifest_index, &opf_dir);
    let mut report = CoverReport::default();

    let selected = select_cover(opf, &items, spine_index, &opf_dir, archive)
        .and_then(|(item, source)| normalize_selected_cover(item, source, archive));

    let selected = match selected {
        Some(selected) => selected,
        None => create_placeholder_cover(opf, &items, &opf_dir, archive),
    };

    report.selected_source = (selected.source != CoverSource::Placeholder)
        .then(|| selected.item.path.clone());
    report.generated_placeholder = selected.source == CoverSource::Placeholder;
    report.reencoded = selected.reencoded;
    report.cover_path = Some(selected.item.path.clone());

    report.renamed_item_ids = rename_conflicting_cover_ids(
        opf,
        manifest_index,
        &items,
        &selected.item.path,
    );
    report.changed |= report.renamed_item_ids > 0;

    report.changed |= upsert_cover_item(
        opf,
        manifest_index,
        &selected.item,
        &selected.media_type,
    )?;

    report.removed_cover_properties =
        remove_duplicate_cover_properties(opf, manifest_index, &selected.item.path, &opf_dir);
    report.changed |= report.removed_cover_properties > 0;

    report.removed_cover_metadata = remove_legacy_cover_metadata(opf);
    report.changed |= report.removed_cover_metadata > 0;

    if archive.get(&selected.item.path) != Some(&selected.bytes) {
        archive.insert(selected.item.path, selected.bytes);
        report.changed = true;
    }

    Ok(report)
}

fn validate_package(opf: &Element) -> Result<(), CoverError> {
    if local_name(&opf.name) != "package" {
        return Err(CoverError::MissingPackage);
    }
    Ok(())
}

fn select_cover(
    opf: &Element,
    items: &[ManifestItem],
    spine_index: usize,
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<(ManifestItem, CoverSource)> {
    find_cover_image_property(items)
        .or_else(|| find_legacy_cover_metadata(opf, items))
        .or_else(|| find_guide_cover(opf, items, opf_dir, archive))
        .or_else(|| find_first_spine_image(opf, items, spine_index, archive))
}

fn normalize_selected_cover(
    item: ManifestItem,
    source: CoverSource,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<SelectedCover> {
    let bytes = archive.get(&item.path)?;
    let (bytes, media_type, reencoded) = normalize_cover_image(bytes)?;

    Some(SelectedCover {
        item,
        bytes,
        media_type,
        source,
        reencoded,
    })
}

fn create_placeholder_cover(
    opf: &Element,
    items: &[ManifestItem],
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> SelectedCover {
    let path = unused_placeholder_path(opf_dir, archive);
    let bytes = placeholder_image(&book_title(opf));
    let href = relative_href(opf_dir, &path);

    SelectedCover {
        item: ManifestItem {
            id: unique_id("cover", items),
            href,
            path,
            media_type: "image/png".into(),
            properties: "cover-image".into(),
        },
        bytes,
        media_type: "image/png".into(),
        source: CoverSource::Placeholder,
        reencoded: false,
    }
}

fn find_cover_image_property(items: &[ManifestItem]) -> Option<(ManifestItem, CoverSource)> {
    items
        .iter()
        .find(|item| {
            has_property_value(&item.properties, "cover-image")
                && is_supported_image_type(&item.media_type)
        })
        .cloned()
        .map(|item| (item, CoverSource::CoverImageProperty))
}

fn find_legacy_cover_metadata(
    opf: &Element,
    items: &[ManifestItem],
) -> Option<(ManifestItem, CoverSource)> {
    let cover_id = find_elements(opf, "meta")
        .into_iter()
        .find(|meta| meta.attributes.get("name").map(String::as_str) == Some("cover"))?
        .attributes
        .get("content")?;

    items
        .iter()
        .find(|item| item.id == *cover_id && is_supported_image_type(&item.media_type))
        .cloned()
        .map(|item| (item, CoverSource::LegacyMetadata))
}

fn find_guide_cover(
    opf: &Element,
    items: &[ManifestItem],
    opf_dir: &str,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<(ManifestItem, CoverSource)> {
    let href = find_elements(opf, "reference")
        .into_iter()
        .find(|reference| reference.attributes.get("type").map(String::as_str) == Some("cover"))?
        .attributes
        .get("href")?;
    let path = resolve_path(opf_dir, href)?;

    if let Some(item) = items.iter().find(|item| {
        item.path == path && is_supported_image_type(&item.media_type)
    }) {
        return Some((item.clone(), CoverSource::Guide));
    }

    let wrapper = Element::parse(archive.get(&path)?.as_slice()).ok()?;
    let image_href = find_elements(&wrapper, "img")
        .into_iter()
        .find_map(|image| image.attributes.get("src"))?;
    let image_path = resolve_path(&parent_path(&path), image_href)?;

    items
        .iter()
        .find(|item| {
            item.path == image_path && is_supported_image_type(&item.media_type)
        })
        .cloned()
        .map(|item| (item, CoverSource::Guide))
}

fn find_first_spine_image(
    opf: &Element,
    items: &[ManifestItem],
    spine_index: usize,
    archive: &HashMap<String, Vec<u8>>,
) -> Option<(ManifestItem, CoverSource)> {
    let spine = element_at(opf, spine_index)?;
    let first_id = spine.children.iter().find_map(|node| match node {
        XMLNode::Element(itemref) if local_name(&itemref.name) == "itemref" => {
            itemref.attributes.get("idref")
        }
        _ => None,
    })?;
    let first = items.iter().find(|item| item.id == *first_id)?;

    if is_supported_image_type(&first.media_type) {
        return Some((first.clone(), CoverSource::FirstSpineImage));
    }

    let document = Element::parse(archive.get(&first.path)?.as_slice()).ok()?;
    let image_href = find_elements(&document, "img")
        .into_iter()
        .find_map(|image| image.attributes.get("src"))?;
    let image_path = resolve_path(&parent_path(&first.path), image_href)?;

    items
        .iter()
        .find(|item| {
            item.path == image_path && is_supported_image_type(&item.media_type)
        })
        .cloned()
        .map(|item| (item, CoverSource::FirstSpineImage))
}

fn read_manifest(
    opf: &Element,
    manifest_index: usize,
    opf_dir: &str,
) -> Vec<ManifestItem> {
    let Some(manifest) = element_at(opf, manifest_index) else {
        return Vec::new();
    };

    manifest
        .children
        .iter()
        .filter_map(|node| {
            let XMLNode::Element(item) = node else {
                return None;
            };
            if local_name(&item.name) != "item" {
                return None;
            }

            let id = item.attributes.get("id")?.clone();
            let href = item.attributes.get("href")?.clone();
            let path = resolve_path(opf_dir, &href)?;

            Some(ManifestItem {
                id,
                href,
                path,
                media_type: item.attributes.get("media-type").cloned().unwrap_or_default(),
                properties: item.attributes.get("properties").cloned().unwrap_or_default(),
            })
        })
        .collect()
}

fn normalize_cover_image(bytes: &[u8]) -> Option<(Vec<u8>, String, bool)> {
    let format = image::guess_format(bytes).ok()?;
    let media_type = media_type_for_format(format)?;
    let decoded = image::load_from_memory_with_format(bytes, format).ok()?;
    let (width, height) = decoded.dimensions();

    if !(MIN_DIMENSION..=MAX_DIMENSION).contains(&width)
        || !(MIN_DIMENSION..=MAX_DIMENSION).contains(&height)
    {
        return None;
    }

    let mut output = Cursor::new(Vec::new());
    decoded.write_to(&mut output, format).ok()?;
    let normalized = output.into_inner();
    let reencoded = normalized != bytes;

    Some((normalized, media_type.into(), reencoded))
}

fn media_type_for_format(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Jpeg => Some("image/jpeg"),
        ImageFormat::Png => Some("image/png"),
        ImageFormat::WebP => Some("image/webp"),
        _ => None,
    }
}

fn is_supported_image_type(media_type: &str) -> bool {
    matches!(media_type, "image/jpeg" | "image/png" | "image/webp")
}

fn rename_conflicting_cover_ids(
    opf: &mut Element,
    manifest_index: usize,
    items: &[ManifestItem],
    selected_path: &str,
) -> usize {
    let conflicting = items
        .iter()
        .filter(|item| item.id == "cover" && item.path != selected_path)
        .map(|item| (item.id.clone(), item.path.clone()))
        .collect::<Vec<_>>();

    let mut renamed = 0;
    for (old_id, path) in conflicting {
        let new_id = unique_id(&format!("{old_id}-original"), items);
        rename_item_id(opf, manifest_index, &old_id, &new_id, &path, items);
        renamed += 1;
    }
    renamed
}

fn rename_item_id(
    opf: &mut Element,
    manifest_index: usize,
    old_id: &str,
    new_id: &str,
    target_path: &str,
    items: &[ManifestItem],
) {
    let target_hrefs = items
        .iter()
        .filter(|item| item.id == old_id && item.path == target_path)
        .map(|item| item.href.clone())
        .collect::<Vec<_>>();

    if let Some(manifest) = element_at_mut(opf, manifest_index) {
        for node in &mut manifest.children {
            let XMLNode::Element(item) = node else {
                continue;
            };
            if local_name(&item.name) != "item" {
                continue;
            }

            if item.attributes.get("id").map(String::as_str) == Some(old_id)
                && item
                    .attributes
                    .get("href")
                    .is_some_and(|href| target_hrefs.contains(href))
            {
                item.attributes.insert("id".into(), new_id.into());
            }
        }
    }

    update_references(opf, old_id, new_id);
}

fn update_references(opf: &mut Element, old_id: &str, new_id: &str) {
    for node in &mut opf.children {
        let XMLNode::Element(element) = node else {
            continue;
        };

        if local_name(&element.name) == "spine" {
            for node in &mut element.children {
                let XMLNode::Element(itemref) = node else {
                    continue;
                };
                if local_name(&itemref.name) == "itemref"
                    && itemref.attributes.get("idref").map(String::as_str) == Some(old_id)
                {
                    itemref.attributes.insert("idref".into(), new_id.into());
                }
            }
        }

        if local_name(&element.name) == "metadata" {
            for node in &mut element.children {
                let XMLNode::Element(meta) = node else {
                    continue;
                };
                if local_name(&meta.name) == "meta"
                    && meta.attributes.get("content").map(String::as_str) == Some(old_id)
                {
                    meta.attributes.insert("content".into(), new_id.into());
                }
            }
        }
    }
}

fn upsert_cover_item(
    opf: &mut Element,
    manifest_index: usize,
    selected: &ManifestItem,
    media_type: &str,
) -> Result<bool, CoverError> {
    let manifest = element_at_mut(opf, manifest_index).ok_or(CoverError::MissingManifest)?;
    let mut changed = false;
    let mut found = false;

    for node in &mut manifest.children {
        let XMLNode::Element(item) = node else {
            continue;
        };
        if local_name(&item.name) != "item" {
            continue;
        }

        let selected_item = item.attributes.get("href").is_some_and(|href| {
            resolve_path("", href).as_deref() == Some(selected.path.as_str())
                || href == &selected.href
        });

        if selected_item || item.attributes.get("id").map(String::as_str) == Some("cover") {
            if found {
                continue;
            }
            found = true;
            changed |= set_attribute(item, "id", "cover");
            changed |= set_attribute(item, "href", &selected.href);
            changed |= set_attribute(item, "media-type", media_type);
            changed |= set_attribute(item, "properties", "cover-image");
        } else if has_property(item, "cover-image") {
            remove_property(item, "cover-image");
            changed = true;
        }
    }

    if !found {
        let mut item = Element::new("item");
        item.attributes.insert("id".into(), "cover".into());
        item.attributes.insert("href".into(), selected.href.clone());
        item.attributes.insert("media-type".into(), media_type.into());
        item.attributes.insert("properties".into(), "cover-image".into());
        manifest.children.push(XMLNode::Element(item));
        changed = true;
    }

    Ok(changed)
}

fn remove_duplicate_cover_properties(
    opf: &mut Element,
    manifest_index: usize,
    selected_path: &str,
    opf_dir: &str,
) -> usize {
    let Some(manifest) = element_at_mut(opf, manifest_index) else {
        return 0;
    };

    let mut removed = 0;
    for node in &mut manifest.children {
        let XMLNode::Element(item) = node else {
            continue;
        };
        if local_name(&item.name) != "item" {
            continue;
        }

        let href = item.attributes.get("href").cloned().unwrap_or_default();
        let path = resolve_path(opf_dir, &href).unwrap_or_default();
        if path != selected_path && has_property(item, "cover-image") {
            remove_property(item, "cover-image");
            removed += 1;
        }
    }
    removed
}

fn remove_legacy_cover_metadata(opf: &mut Element) -> usize {
    let mut removed = 0;
    for node in &mut opf.children {
        let XMLNode::Element(metadata) = node else {
            continue;
        };
        if local_name(&metadata.name) != "metadata" {
            continue;
        }

        metadata.children.retain(|node| {
            let XMLNode::Element(meta) = node else {
                return true;
            };
            let is_cover =
                local_name(&meta.name) == "meta"
                    && meta.attributes.get("name").map(String::as_str) == Some("cover");
            removed += usize::from(is_cover);
            !is_cover
        });
    }
    removed
}

fn placeholder_image(title: &str) -> Vec<u8> {
    let mut image = RgbImage::from_pixel(PLACEHOLDER_WIDTH, PLACEHOLDER_HEIGHT, Rgb([245, 241, 232]));
    draw_title(&mut image, title);
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image)
        .write_to(&mut output, ImageFormat::Png)
        .expect("writing a PNG to memory");
    output.into_inner()
}

fn draw_title(image: &mut RgbImage, title: &str) {
    let lines = wrap_title(title, 20);
    let scale = 3;
    let line_height = 24;
    let total_height = lines.len() as u32 * line_height;
    let start_y = (image.height() - total_height) / 2;

    for (line_index, line) in lines.iter().enumerate() {
        let width = line.chars().count() as u32 * 6 * scale;
        let start_x = image.width().saturating_sub(width) / 2;
        for (character_index, character) in line.chars().enumerate() {
            draw_glyph(
                image,
                character,
                start_x + character_index as u32 * 6 * scale,
                start_y + line_index as u32 * line_height,
                scale,
            );
        }
    }
}

fn wrap_title(title: &str, max_chars: usize) -> Vec<String> {
    let words = title.split_whitespace();
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in words {
        if !current.is_empty() && current.len() + word.len() + 1 > max_chars {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push("Untitled".into());
    }
    lines.truncate(8);
    lines
}

fn draw_glyph(image: &mut RgbImage, character: char, x: u32, y: u32, scale: u32) {
    let glyph = glyph(character.to_ascii_uppercase());
    for (row, bits) in glyph.iter().enumerate() {
        for column in 0..5 {
            if bits & (1 << (4 - column)) == 0 {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x + column * scale + dx;
                    let py = y + row as u32 * scale + dy;
                    if px < image.width() && py < image.height() {
                        image.put_pixel(px, py, Rgb([45, 48, 52]));
                    }
                }
            }
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        ' ' => [0; 7],
        _ => [31, 17, 5, 9, 4, 0, 4],
    }
}

fn book_title(opf: &Element) -> String {
    find_elements(opf, "title")
        .into_iter()
        .find_map(element_text)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Untitled".into())
}

fn element_text(element: &Element) -> Option<String> {
    let mut text = String::new();
    collect_text(element, &mut text);
    (!text.trim().is_empty()).then(|| text.trim().to_string())
}

fn collect_text(element: &Element, output: &mut String) {
    for node in &element.children {
        match node {
            XMLNode::Text(text) => output.push_str(text),
            XMLNode::Element(child) => collect_text(child, output),
            _ => {}
        }
    }
}

fn unused_placeholder_path(opf_dir: &str, archive: &HashMap<String, Vec<u8>>) -> String {
    for suffix in 0.. {
        let filename = if suffix == 0 {
            "cover-generated.png".to_string()
        } else {
            format!("cover-generated-{suffix}.png")
        };
        let path = join_path(opf_dir, &filename);
        if !archive.contains_key(&path) {
            return path;
        }
    }
    unreachable!()
}

fn unique_id(base: &str, items: &[ManifestItem]) -> String {
    if !items.iter().any(|item| item.id == base) {
        return base.into();
    }
    for suffix in 2.. {
        let candidate = format!("{base}-{suffix}");
        if !items.iter().any(|item| item.id == candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn has_property(element: &Element, property: &str) -> bool {
    element
        .attributes
        .get("properties")
        .is_some_and(|properties| has_property_value(properties, property))
}

fn has_property_value(properties: &str, property: &str) -> bool {
    properties.split_whitespace().any(|value| value == property)
}

fn remove_property(element: &mut Element, property: &str) {
    let remaining = element
        .attributes
        .get("properties")
        .into_iter()
        .flat_map(|properties| properties.split_whitespace())
        .filter(|value| *value != property)
        .collect::<Vec<_>>()
        .join(" ");

    if remaining.is_empty() {
        element.attributes.remove("properties");
    } else {
        element.attributes.insert("properties".into(), remaining);
    }
}

fn set_attribute(element: &mut Element, name: &str, value: &str) -> bool {
    if element.attributes.get(name).map(String::as_str) == Some(value) {
        return false;
    }
    element.attributes.insert(name.into(), value.into());
    true
}

fn find_elements<'a>(element: &'a Element, target: &str) -> Vec<&'a Element> {
    let mut found = Vec::new();
    collect_elements(element, target, &mut found);
    found
}

fn collect_elements<'a>(element: &'a Element, target: &str, found: &mut Vec<&'a Element>) {
    if local_name(&element.name) == target {
        found.push(element);
    }
    for node in &element.children {
        if let XMLNode::Element(child) = node {
            collect_elements(child, target, found);
        }
    }
}

fn direct_child_index(element: &Element, target: &str) -> Option<usize> {
    element.children.iter().position(|node| {
        matches!(node, XMLNode::Element(child) if local_name(&child.name) == target)
    })
}

fn element_at(element: &Element, index: usize) -> Option<&Element> {
    match element.children.get(index) {
        Some(XMLNode::Element(child)) => Some(child),
        _ => None,
    }
}

fn element_at_mut(element: &mut Element, index: usize) -> Option<&mut Element> {
    match element.children.get_mut(index) {
        Some(XMLNode::Element(child)) => Some(child),
        _ => None,
    }
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

fn normalize_path(path: &str) -> Option<String> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return None;
    }

    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            value if value.contains('\0') => return None,
            value => components.push(value),
        }
    }

    (!components.is_empty()).then(|| components.join("/"))
}

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn join_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.into()
    } else {
        format!("{parent}/{child}")
    }
}

fn resolve_path(parent: &str, href: &str) -> Option<String> {
    let href = href.split(['#', '?']).next().unwrap_or_default();
    if href.is_empty() || href.starts_with('/') || href.contains("://") {
        return None;
    }
    normalize_path(&join_path(parent, href))
}

fn relative_href(parent: &str, target: &str) -> String {
    let parent_parts = parent.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
    let target_parts = target.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
    let shared = parent_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(left, right)| left == right)
        .count();

    let mut result = vec![".."; parent_parts.len() - shared];
    result.extend(target_parts[shared..].iter().copied());
    result.join("/")
}

#[cfg(test)]
#[path = "cover_tests.rs"]
mod tests;
