use std::{ fs, io::{Cursor, Read}, path::Path};
use zip::ZipArchive;

pub mod errors;

use errors::{ChapterError, CoverError, ManifestError, SpineError, ZipError, ValidationError, NavError, ReadFail};

use crate::validator::errors::PackageError;

#[derive(Debug)]
struct Package {
    manifest: Vec<Item>,
    spine: Vec<SpineItem>
}

#[derive(Debug)]
struct Item {
    id: String,
    href: String,
    media_type: String,
}

#[derive(Debug)]
struct SpineItem {
    idref: String,
}

pub fn verify_ebook(zip_path: &str, opf_path: &str) -> Result<(), ValidationError> {
    let mut book = get_book(zip_path)?;
    let package=get_package(&mut book, opf_path)?;

    check_cover(&mut book, opf_path, &package.manifest)?;

    check_nav(&mut book, opf_path, &package.manifest)?;

    for item in &package.spine {
        verify_chapter(&mut book, item, opf_path, &package.manifest)?;
    }

    Ok(())
}

fn get_book(zip_path: &str) -> Result<ZipArchive<Cursor<Vec<u8>>>, ZipError> {
    let bytes = fs::read(zip_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ZipError::ZipFileMissing
        } else {
            ZipError::Internal(Box::new(error))
        }
    })?;

    ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| ZipError::Internal(Box::new(error)))
}


fn get_package(book: &mut ZipArchive<Cursor<Vec<u8>>>, opf_path: &str)->Result<Package, PackageError>{
    let opf = read_zip_text(book, opf_path).map_err(|e| match e {
        ReadFail::NotFound   => PackageError::MissingOpf(opf_path.to_string()),
        ReadFail::Unreadable => PackageError::MissingOpf(opf_path.to_string()), // no ReadError variant for chapters
        ReadFail::NotUtf8    => PackageError::MalformedFile(opf_path.to_string()),
    })?;
    let doc = roxmltree::Document::parse(&opf).map_err(|_|PackageError::MalformedFile(opf_path.to_string()))?;

    if !doc.root_element().has_tag_name("package") {
        return Err(PackageError::MalformedFile(opf_path.to_string()));
    }

    Ok(Package{
        manifest: get_manifest(&doc)?,
        spine: get_spine(&doc)?
    })

}

fn get_manifest(doc: &roxmltree::Document)->Result<Vec<Item>,ManifestError>{
    let manifest: Vec<Item> = doc.descendants()
        .filter(|n| n.has_tag_name("item"))
        .map(|n| Ok(Item {
            id: n.attribute("id").ok_or(ManifestError::MissingItem("id".to_string()))?.to_string(),
            href: n.attribute("href").ok_or(ManifestError::MissingItem("href".to_string()))?.to_string(),
            media_type: n.attribute("media-type").ok_or(ManifestError::MissingItem("media-type".to_string()))?.to_string(),
        }))
        .collect::<Result<_, ManifestError>>()?;
    
    if manifest.is_empty(){
        return Err(ManifestError::EmptyManifest)
    }

    check_manifest_id_duplication(&manifest)?;


    Ok(manifest)
}


fn get_spine(doc: &roxmltree::Document)->Result<Vec<SpineItem>, SpineError>{
    let spine: Vec<SpineItem>=doc.descendants()
        .filter(|n| n.has_tag_name("itemref"))
        .map(|n| Ok(SpineItem {
            idref: n.attribute("idref").ok_or(SpineError::MissingIdref)?.to_string(),
        }))
        .collect::<Result<_, SpineError>>()?;
    
    if spine.is_empty(){
        return Err(SpineError::EmptySpine)
    }

    check_spine_id_duplication(&spine)?;

    Ok(spine)
}


fn check_manifest_id_duplication(manifest: &[Item]) -> Result<(), ManifestError> {
    let mut ids = std::collections::HashSet::new();

    for item in manifest {
        if !ids.insert(&item.id) {
            return Err(ManifestError::DuplicateId(item.id.clone()));
        }
    }

    Ok(())
}

fn check_spine_id_duplication(spine: &[SpineItem]) -> Result<(), SpineError> {
    let mut idrefs = std::collections::HashSet::new();

    for item in spine {
        if !idrefs.insert(&item.idref) {
            return Err(SpineError::DuplicateIdref(item.idref.clone()));
        }
    }

    Ok(())
}
use zip::result::ZipError as ZipLibError;

fn read_zip_text(book: &mut ZipArchive<Cursor<Vec<u8>>>, path: &str) -> Result<String, ReadFail> {
    let mut file = book.by_name(path).map_err(|e| match e {
        ZipLibError::FileNotFound => ReadFail::NotFound,
        _ => ReadFail::Unreadable,
    })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|_| ReadFail::Unreadable)?;
    String::from_utf8(bytes).map_err(|_| ReadFail::NotUtf8)
}

fn resolve_href(opf_path: &str, href: &str) -> Result<String, &'static str> {
    let base = Path::new(opf_path).parent().unwrap_or(Path::new(""));
    let path = base.join(href);

    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Path escapes EPUB root");
    }

    Ok(path.to_string_lossy().into_owned())
}

fn check_cover(
    book: &mut ZipArchive<Cursor<Vec<u8>>>,
    opf_path: &str,
    manifest: &[Item],
) -> Result<(), CoverError> {
    let cover = manifest.iter()
        .find(|x| x.id == "cover")
        .ok_or(CoverError::MissingId)?;

    if !matches!(cover.media_type.as_str(), "image/jpeg" | "image/png" | "image/webp") {
        return Err(CoverError::UnsupportedMediaType(cover.media_type.clone()));
    }

    let path = resolve_href(opf_path, &cover.href).map_err(|_|CoverError::PathEscape(cover.href.clone()))?;
    let mut data = Vec::new();

    book
    .by_name(&path)
    .map_err(|_| CoverError::MissingFile(path.clone()))?
    .read_to_end(&mut data)
    .map_err(|_| CoverError::ReadError(path.clone()))?;
    
    
    let img=image::load_from_memory(&data).map_err(|_|CoverError::InvalidImage(path.clone()))?;
    let (w, h) = (img.width(), img.height());
    if !(100..=10_000).contains(&w) || !(100..=10_000).contains(&h) {
        return Err(CoverError::InvalidImage(format!("{path}: {w}x{h}, must be 100-10000 px")));
    }

    Ok(())
}


fn check_nav(
    book: &mut ZipArchive<Cursor<Vec<u8>>>,
    opf_path: &str,
    manifest: &[Item],
) -> Result<(), NavError> {
    let nav = manifest
        .iter()
        .find(|item| item.id == "nav")
        .ok_or(NavError::MissingId)?;

    if nav.media_type != "application/xhtml+xml" {
        return Err(NavError::UnsupportedMediaType(nav.media_type.clone()));
    }

    let path = resolve_href(opf_path, &nav.href)
        .map_err(|_| NavError::PathEscape(nav.href.clone()))?;

    let text = read_zip_text(book, &path).map_err(|e| match e {
        ReadFail::NotFound   => NavError::MissingFile(path.clone()),
        ReadFail::Unreadable => NavError::MissingFile(path.clone()), // no ReadError variant for chapters
        ReadFail::NotUtf8    => NavError::MalformedFile(path.clone()),
    })?;

    let doc = roxmltree::Document::parse(&text)
        .map_err(|_| NavError::MalformedFile(path.clone()))?;

    let toc = doc.descendants()
        .find(|node| {
            node.has_tag_name("nav")
                && node.attribute(("http://www.idpf.org/2007/ops", "type"))
                    == Some("toc")
        })
        .ok_or(NavError::MissingToc)?;

    let mut entries = 0;

    for li in toc.descendants().filter(|node| node.has_tag_name("li")) {
        let link = li.descendants()
            .find(|node| node.has_tag_name("a") && node.attribute("href").is_some())
            .ok_or(NavError::MissingHref)?;

        let label = link.text().unwrap_or("").trim();
        if label.is_empty() {
            return Err(NavError::EmptyLabel);
        }

        let href = link.attribute("href").unwrap();
        if href.starts_with('#') || href.starts_with("data:") {
            return Err(NavError::InvalidHref(href.to_string()));
        }

        let target = href.split('#').next().unwrap();
        let target_path = resolve_href(&path, target)
            .map_err(|_| NavError::PathEscape(href.to_string()))?;

        let target_in_manifest = manifest.iter().any(|item| {
            resolve_href(opf_path, &item.href)
                .is_ok_and(|item_path| item_path == target_path)
        });

        if !target_in_manifest {
            return Err(NavError::TargetNotInManifest(href.to_string()));
        }

        entries += 1;
    }

    if entries == 0 {
        return Err(NavError::EmptyToc);
    }

    Ok(())
}


fn verify_chapter(
    book: &mut ZipArchive<Cursor<Vec<u8>>>,
    item: &SpineItem,
    opf_path: &str,
    manifest: &[Item],
) -> Result<(), ChapterError> {
    let entry = manifest.iter()
        .find(|x| x.id == item.idref)
        .ok_or(ChapterError::SpineReferenceNotFound(item.idref.clone()))?;

    if entry.media_type != "application/xhtml+xml" {
        return Err(ChapterError::UnsupportedMediaType(entry.media_type.clone()));
    }

    let path = resolve_href(opf_path, &entry.href).map_err(|_| ChapterError::PathEscape(entry.href.clone()))?;
    let text = read_zip_text(book, &path).map_err(|e| match e {
        ReadFail::NotFound   => ChapterError::MissingFile(path.clone()),
        ReadFail::Unreadable => ChapterError::MissingFile(path.clone()), // no ReadError variant for chapters
        ReadFail::NotUtf8    => ChapterError::MalformedFile(path.clone()),
    })?;
    roxmltree::Document::parse(&text).map_err(|_|ChapterError::MalformedFile(path.clone()))?;

    Ok(())
}



#[cfg(test)]
mod validator_tests;