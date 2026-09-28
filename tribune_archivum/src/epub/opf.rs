use crate::core::{
    ErrorCode, ManifestItem, OpfPackage, SpineItem, ValidationError, ValidationLocation,
    ValidationResult,
};
use serde::Deserialize;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

// ---- Serde model of the OPF document -------------------------------------
// quick-xml's deserializer matches elements by local name (so `dc:title` and
// `opf:package` work) and maps attributes via the `@` prefix.

/// Root wrapper: makes deserialization fail unless the root element is <package>.
#[derive(Deserialize)]
enum Root {
    #[serde(rename = "package")]
    Package(Package),
}

#[derive(Deserialize)]
struct Package {
    #[serde(default)]
    metadata: Metadata,
    manifest: Option<Manifest>,
    spine: Option<Spine>,
}

#[derive(Deserialize, Default)]
struct Metadata {
    #[serde(rename = "title", default)]
    titles: Vec<Text>,
    #[serde(rename = "creator", default)]
    creators: Vec<Text>,
    #[serde(rename = "language", default)]
    languages: Vec<Text>,
    #[serde(rename = "identifier", default)]
    identifiers: Vec<Text>,
}

/// An element's text content; the struct form tolerates attributes (id, xml:lang, opf:role...).
#[derive(Deserialize)]
struct Text {
    #[serde(rename = "$text", default)]
    text: String,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "item", default)]
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    #[serde(rename = "@id", default)]
    id: String,
    #[serde(rename = "@href", default)]
    href: String,
    #[serde(rename = "@media-type", default)]
    media_type: String,
    #[serde(rename = "@properties", default)]
    properties: Option<String>,
}

#[derive(Deserialize)]
struct Spine {
    #[serde(rename = "itemref", default)]
    items: Vec<ItemRef>,
}

#[derive(Deserialize)]
struct ItemRef {
    #[serde(rename = "@idref", default)]
    idref: String,
    #[serde(rename = "@linear", default = "default_linear")]
    linear: String,
    #[serde(rename = "@properties", default)]
    properties: Option<String>,
}

fn default_linear() -> String {
    "yes".to_string()
}

// ---- Conversion ------------------------------------------------------------

fn split_properties(p: Option<String>) -> Vec<String> {
    p.map(|s| s.split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default()
}

fn texts(v: &[Text]) -> Vec<String> {
    v.iter().map(|t| t.text.trim().to_string()).collect()
}

/// Parses and validates the OPF document (path unknown).
pub fn parse_opf(opf_data: &[u8]) -> Result<OpfPackage, ValidationResult> {
    parse_opf_at("", opf_data)
}

/// Parses and validates the OPF document, reporting errors against `path`.
pub fn parse_opf_at(path: &str, opf_data: &[u8]) -> Result<OpfPackage, ValidationResult> {
    let mut result = ValidationResult::new();
    let location = || ValidationLocation::Opf { path: path.to_string() };

    let bytes = opf_data.strip_prefix(UTF8_BOM).unwrap_or(opf_data);
    let parsed = std::str::from_utf8(bytes)
        .ok()
        .and_then(|s| quick_xml::de::from_str::<Root>(s).ok());

    let Some(Root::Package(pkg)) = parsed else {
        result.add_error(ValidationError::new(ErrorCode::InvalidOpfXml, location()));
        return Err(result);
    };

    let mut failed = false;
    if pkg.manifest.as_ref().map_or(true, |m| m.items.is_empty()) {
        result.add_error(ValidationError::new(ErrorCode::MissingManifest, location()));
        failed = true;
    }
    if pkg.spine.as_ref().map_or(true, |s| s.items.is_empty()) {
        result.add_error(ValidationError::new(ErrorCode::MissingSpine, location()));
        failed = true;
    }
    if failed {
        return Err(result);
    }

    let manifest_items = pkg
        .manifest
        .into_iter()
        .flat_map(|m| m.items)
        .map(|i| ManifestItem {
            id: i.id,
            href: i.href,
            media_type: i.media_type,
            properties: split_properties(i.properties),
        })
        .collect();

    let spine_items = pkg
        .spine
        .into_iter()
        .flat_map(|s| s.items)
        .map(|i| SpineItem {
            idref: i.idref,
            linear: i.linear,
            properties: split_properties(i.properties),
        })
        .collect();

    // Multiple titles/languages are allowed; the first is the primary.
    let m = &pkg.metadata;
    let metadata = serde_json::json!({
        "title": texts(&m.titles).into_iter().next(),
        "creators": texts(&m.creators),
        "language": texts(&m.languages).into_iter().next(),
        "identifiers": texts(&m.identifiers),
    });

    Ok(OpfPackage {
        path: path.to_string(),
        root_file_path: path.to_string(),
        metadata,
        manifest_items,
        spine_items,
    })
}