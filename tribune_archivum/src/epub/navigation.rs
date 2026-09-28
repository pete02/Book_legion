use crate::archive::{entry_exists, read_entry};
use crate::core::{
    ErrorCode, ManifestInfo, NavInfo, OpfPackage, TocEntry, ValidationError, ValidationLocation,
    ValidationResult,
};
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;
use std::error::Error;
use zip::ZipArchive;

type XmlResult<T> = Result<T, Box<dyn Error>>;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const NCX_MEDIA_TYPE: &str = "application/x-dtbncx+xml";

// ---- Shared intermediate representation ------------------------------------

/// One TOC node, independent of whether it came from an EPUB3 nav or an EPUB2 NCX.
#[derive(Debug, Default)]
struct NavItem {
    /// `None` = no link (e.g. a heading `<span>`); `Some("")` = link without a target.
    href: Option<String>,
    title: String,
    children: Vec<NavItem>,
}

/// Converts parsed items into `TocEntry`s, recording invalid entries.
///
/// - unlinked headings and fragment-only links are dropped, but their children are kept
/// - links with an empty target are reported as `InvalidTocEntry`
///
/// `counter` numbers entries in document order and is used as the error location.
fn collect_entries(
    items: Vec<NavItem>,
    counter: &mut usize,
    result: &mut ValidationResult,
) -> Vec<TocEntry> {
    let mut out = Vec::new();
    for item in items {
        let index = *counter;
        *counter += 1;
        let children = collect_entries(item.children, counter, result);

        match item.href.as_deref() {
            None => out.extend(children),
            Some("") => {
                result.add_error(ValidationError::new(
                    ErrorCode::InvalidTocEntry,
                    ValidationLocation::TocEntry { index },
                ));
                out.extend(children);
            }
            Some(h) if h.starts_with('#') => out.extend(children),
            Some(h) => out.push(TocEntry {
                title: normalize_ws(&item.title),
                href: h.to_string(),
                children,
            }),
        }
    }
    out
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn single_error(code: ErrorCode, location: ValidationLocation) -> ValidationResult {
    let mut r = ValidationResult::new();
    r.add_error(ValidationError::new(code, location));
    r
}

fn decode(data: &[u8]) -> Option<&str> {
    std::str::from_utf8(data.strip_prefix(UTF8_BOM).unwrap_or(data)).ok()
}

// ---- Discovery -----------------------------------------------------------------

/// Discovers and validates navigation documents (EPUB3 nav or EPUB2 NCX)
pub fn validate_navigation(
    archive: &mut ZipArchive<std::fs::File>,
    _opf: &OpfPackage,
    manifest_info: &ManifestInfo,
) -> Result<NavInfo, ValidationResult> {
    // Prefer the EPUB3 nav document
    if let Some(nav_path) = find_nav_document(manifest_info) {
        if entry_exists(archive, &nav_path) {
            return validate_epub3_nav(archive, &nav_path);
        }
    }

    // Fall back to the EPUB2 NCX
    if let Some(ncx_path) = find_ncx_document(manifest_info) {
        if entry_exists(archive, &ncx_path) {
            return validate_ncx(archive, &ncx_path);
        }
    }

    Err(single_error(ErrorCode::MissingNavDocument, ValidationLocation::Root))
}

/// The EPUB3 nav document is the manifest item with the `nav` property.
fn find_nav_document(manifest_info: &ManifestInfo) -> Option<String> {
    manifest_info
        .items
        .iter()
        .find(|item| item.properties.iter().any(|p| p == "nav"))
        .map(|item| item.href.clone())
}

/// The NCX is identified by its media type.
fn find_ncx_document(manifest_info: &ManifestInfo) -> Option<String> {
    manifest_info
        .items
        .iter()
        .find(|item| item.media_type == NCX_MEDIA_TYPE)
        .map(|item| item.href.clone())
}

// ---- EPUB3 nav -------------------------------------------------------------------

fn attr_value(e: &BytesStart, key: &[u8]) -> XmlResult<Option<String>> {
    for a in e.attributes() {
        let a = a?;
        if a.key.as_ref() == key {
            return Ok(Some(a.unescape_value()?.into_owned()));
        }
    }
    Ok(None)
}

/// True for `<nav epub:type="toc">` (the prefix is ignored; the value is a token list).
fn is_toc_nav(e: &BytesStart) -> XmlResult<bool> {
    for a in e.attributes() {
        let a = a?;
        if a.key.local_name().as_ref() == b"type"
            && a.unescape_value()?.split_whitespace().any(|t| t == "toc")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Extracts the hierarchy of the `epub:type="toc"` nav.
///
/// Returns `Ok(None)` if the document has no toc nav. Titles are gathered from
/// all text inside the first `<a>` of each `<li>`, so `<a><span>Title</span></a>`
/// and `<a>Chapter <em>1</em></a>` both work, at any nesting depth.
fn parse_toc_nav(xml: &str) -> XmlResult<Option<Vec<NavItem>>> {
    let mut reader = Reader::from_str(xml);

    let mut depth = 0usize;
    let mut in_toc = false;
    let mut found_toc = false;
    let mut saw_root = false;
    let mut in_anchor = false;
    let mut open_items: Vec<NavItem> = Vec::new();
    let mut roots: Vec<NavItem> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                saw_root=true;
                depth += 1;
                match e.local_name().as_ref() {
                    b"nav" if !found_toc => {
                        if is_toc_nav(&e)? {
                            in_toc = true;
                            found_toc = true;
                        }
                    }
                    b"li" if in_toc => open_items.push(NavItem::default()),
                    b"a" if in_toc => {
                        // Only the first link of each <li> counts
                        if let Some(item) = open_items.last_mut() {
                            if item.href.is_none() {
                                item.href = Some(attr_value(&e, b"href")?.unwrap_or_default());
                                in_anchor = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => {
                saw_root=true;
                // <a href="x"/>: a link without a title
                if in_toc && e.local_name().as_ref() == b"a" {
                    if let Some(item) = open_items.last_mut() {
                        if item.href.is_none() {
                            item.href = Some(attr_value(&e, b"href")?.unwrap_or_default());
                        }
                    }
                }
            }
            Event::Text(t) if in_anchor => {
                if let Some(item) = open_items.last_mut() {
                    item.title.push_str(&t.unescape()?);
                }
            }
            Event::CData(c) if in_anchor => {
                if let Some(item) = open_items.last_mut() {
                    item.title.push_str(&String::from_utf8_lossy(&c));
                }
            }
            Event::End(e) => {
                depth = depth.checked_sub(1).ok_or("unbalanced end tag")?;
                match e.local_name().as_ref() {
                    b"a" => in_anchor = false,
                    b"li" if in_toc => {
                        if let Some(item) = open_items.pop() {
                            match open_items.last_mut() {
                                Some(parent) => parent.children.push(item),
                                None => roots.push(item),
                            }
                        }
                    }
                    b"nav" if in_toc => in_toc = false,
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if !saw_root {
       return Err("invalid XML".into());
    }

    if depth != 0 {
        return Err("unclosed element(s)".into());
    }
    Ok(found_toc.then_some(roots))
}

fn parse_nav_document(nav_path: &str, xml: &str) -> Result<NavInfo, ValidationResult> {
    let location = || ValidationLocation::Navigation { path: nav_path.to_string() };

    let items = match parse_toc_nav(xml) {
        Ok(Some(items)) => items,
        Ok(None) => return Err(single_error(ErrorCode::MissingTocNav, location())),
        Err(_) => return Err(single_error(ErrorCode::InvalidNavXml, location())),
    };

    let mut result = ValidationResult::new();
    let mut counter = 0;
    let toc_entries = collect_entries(items, &mut counter, &mut result);

    if toc_entries.is_empty() {
        result.add_error(ValidationError::new(ErrorCode::EmptyToc, location()));
    }
    if !result.errors.is_empty() {
        return Err(result);
    }

    Ok(NavInfo {
        nav_path: Some(nav_path.to_string()),
        ncx_path: None,
        toc_entries,
    })
}

fn validate_epub3_nav(
    archive: &mut ZipArchive<std::fs::File>,
    nav_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let location = || ValidationLocation::Navigation { path: nav_path.to_string() };

    let data = read_entry(archive, nav_path)
        .map_err(|_| single_error(ErrorCode::MissingNavDocument, location()))?;
    let xml = decode(&data).ok_or_else(|| single_error(ErrorCode::InvalidNavXml, location()))?;
    parse_nav_document(nav_path, xml)
}

// ---- EPUB2 NCX -------------------------------------------------------------------
// The NCX is plain nested elements with no mixed content, so serde works well here.

#[derive(Deserialize)]
enum NcxRoot {
    #[serde(rename = "ncx")]
    Ncx(Ncx),
}

#[derive(Deserialize)]
struct Ncx {
    #[serde(rename = "navMap")]
    nav_map: NcxNavMap,
}

#[derive(Deserialize)]
struct NcxNavMap {
    #[serde(rename = "navPoint", default)]
    nav_points: Vec<NcxNavPoint>,
}

#[derive(Deserialize)]
struct NcxNavPoint {
    /// May be repeated (one per language); the first is used.
    #[serde(rename = "navLabel", default)]
    labels: Vec<NcxNavLabel>,
    content: Option<NcxContent>,
    #[serde(rename = "navPoint", default)]
    children: Vec<NcxNavPoint>,
}

#[derive(Deserialize)]
struct NcxNavLabel {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct NcxContent {
    #[serde(rename = "@src", default)]
    src: String,
}

fn ncx_to_items(points: Vec<NcxNavPoint>) -> Vec<NavItem> {
    points
        .into_iter()
        .map(|p| NavItem {
            // A navPoint without <content> is treated as a link with no target (invalid)
            href: Some(p.content.map(|c| c.src).unwrap_or_default()),
            title: p.labels.into_iter().next().map(|l| l.text).unwrap_or_default(),
            children: ncx_to_items(p.children),
        })
        .collect()
}

fn parse_ncx_document(ncx_path: &str, xml: &str) -> Result<NavInfo, ValidationResult> {
    let location = || ValidationLocation::Ncx { path: ncx_path.to_string() };

    let Ok(NcxRoot::Ncx(ncx)) = quick_xml::de::from_str::<NcxRoot>(xml) else {
        return Err(single_error(ErrorCode::InvalidNcxXml, location()));
    };

    let mut result = ValidationResult::new();
    let mut counter = 0;
    let toc_entries = collect_entries(ncx_to_items(ncx.nav_map.nav_points), &mut counter, &mut result);

    if toc_entries.is_empty() {
        result.add_error(ValidationError::new(ErrorCode::EmptyToc, location()));
    }
    if !result.errors.is_empty() {
        return Err(result);
    }

    Ok(NavInfo {
        nav_path: None,
        ncx_path: Some(ncx_path.to_string()),
        toc_entries,
    })
}

fn validate_ncx(
    archive: &mut ZipArchive<std::fs::File>,
    ncx_path: &str,
) -> Result<NavInfo, ValidationResult> {
    let data = read_entry(archive, ncx_path).map_err(|_| {
        single_error(
            ErrorCode::MissingNavDocument,
            ValidationLocation::Navigation { path: ncx_path.to_string() },
        )
    })?;
    let xml = decode(&data).ok_or_else(|| {
        single_error(ErrorCode::InvalidNcxXml, ValidationLocation::Ncx { path: ncx_path.to_string() })
    })?;

    parse_ncx_document(ncx_path, xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(r: &ValidationResult) -> Vec<ErrorCode> {
        r.errors.iter().map(|e| e.code).collect()
    }

    fn nav_doc(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Nav</title></head>
<body>{body}</body></html>"#
        )
    }

    #[test]
    fn nav_titles_from_nested_markup_and_deep_nesting() {
        let xml = nav_doc(
            r#"<nav epub:type="toc"><h1>Contents</h1><ol>
  <li><a href="c1.xhtml"><span>One</span></a>
    <ol><li><a href="c1a.xhtml">Chapter <em>1</em>a</a>
      <ol><li><a href="c1a1.xhtml">Deep</a></li></ol></li></ol></li>
  <li><a href="c2.xhtml">Two</a></li>
</ol></nav>"#,
        );
        let info = parse_nav_document("nav.xhtml", &xml).expect("should parse");
        let e = &info.toc_entries;
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].title, "One");
        assert_eq!(e[0].children[0].title, "Chapter 1a");
        assert_eq!(e[0].children[0].children[0].href, "c1a1.xhtml");
        assert_eq!(e[1].title, "Two");
    }

    #[test]
    fn nav_ignores_non_toc_navs_and_keeps_children_of_headings_and_fragments() {
        let xml = nav_doc(
            r##"<nav epub:type="landmarks"><ol><li><a href="cover.xhtml">Cover</a></li></ol></nav>
<nav epub:type="toc"><ol>
  <li><span>Part I</span><ol><li><a href="a.xhtml">A</a></li></ol></li>
  <li><a href="#frag">Skipped</a><ol><li><a href="b.xhtml">B</a></li></ol></li>
</ol></nav>"##,
        );
        let info = parse_nav_document("nav.xhtml", &xml).expect("should parse");
        let hrefs: Vec<_> = info.toc_entries.iter().map(|e| e.href.as_str()).collect();
        assert_eq!(hrefs, vec!["a.xhtml", "b.xhtml"]);
    }

    #[test]
    fn nav_without_toc_is_missing_toc_nav() {
        let xml = nav_doc(r#"<nav epub:type="landmarks"><ol><li><a href="x.xhtml">X</a></li></ol></nav>"#);
        assert_eq!(codes(&parse_nav_document("n", &xml).unwrap_err()), vec![ErrorCode::MissingTocNav]);
    }

    #[test]
    fn nav_empty_href_and_empty_toc() {
        let xml = nav_doc(r#"<nav epub:type="toc"><ol><li><a href="">Bad</a></li></ol></nav>"#);
        let c = codes(&parse_nav_document("n", &xml).unwrap_err());
        assert!(c.contains(&ErrorCode::InvalidTocEntry) && c.contains(&ErrorCode::EmptyToc), "{c:?}");
    }

    #[test]
    fn nav_malformed_is_invalid_nav_xml() {
        let xml = r#"<html><body><nav epub:type="toc"><ol><li><a href="a">A</a></li></ol>"#;
        assert_eq!(codes(&parse_nav_document("n", xml).unwrap_err()), vec![ErrorCode::InvalidNavXml]);
    }

    const NCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="x"/></head>
  <docTitle><text>Book</text></docTitle>
  <navMap>
    <navPoint id="n1" playOrder="1"><navLabel><text>One</text></navLabel><content src="c1.xhtml"/>
      <navPoint id="n2" playOrder="2"><navLabel><text>One A</text></navLabel><content src="c1.xhtml#a"/></navPoint>
    </navPoint>
    <navPoint id="n3" playOrder="3"><navLabel><text>Two</text></navLabel><content src="c2.xhtml"/></navPoint>
  </navMap>
</ncx>"#;

    #[test]
    fn ncx_parses_with_doctype_and_nesting() {
        let info = parse_ncx_document("toc.ncx", NCX).expect("should parse");
        assert_eq!(info.toc_entries.len(), 2);
        assert_eq!(info.toc_entries[0].children[0].title, "One A");
        assert_eq!(info.ncx_path.as_deref(), Some("toc.ncx"));
    }

    #[test]
    fn ncx_without_namespace_attribute_is_fine() {
        let xml = NCX.replace(r#" xmlns="http://www.daisy.org/z3986/2005/ncx/""#, "");
        assert!(parse_ncx_document("toc.ncx", &xml).is_ok());
    }

    #[test]
    fn ncx_empty_navmap_is_an_error() {
        let xml = r#"<ncx><navMap></navMap></ncx>"#;
        assert_eq!(codes(&parse_ncx_document("toc.ncx", xml).unwrap_err()), vec![ErrorCode::EmptyToc]);
    }

    #[test]
    fn ncx_wrong_root_or_malformed() {
        assert_eq!(codes(&parse_ncx_document("t", "<html/>").unwrap_err()), vec![ErrorCode::InvalidNcxXml]);
        assert_eq!(codes(&parse_ncx_document("t", "<ncx><navMap>").unwrap_err()), vec![ErrorCode::InvalidNcxXml]);
    }
}