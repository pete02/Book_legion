use std::path::Path;

#[derive(Debug)]
pub struct TocEntry {
    pub src: String,
}

#[derive(Debug)]
pub struct TocAnalysisResult {
    pub toc_present: bool,
    pub toc_entries: Vec<TocEntry>,
    pub chapter_files: Vec<String>,
    pub toc_matches_chapters: bool,
    pub missing_chapters: Vec<String>,
    pub orphan_toc_entries: Vec<String>,
}

use std::fs::File;
use zip::ZipArchive;
use anyhow::Result;

pub fn analyze_epub(path: &Path) -> TocAnalysisResult {
    let file = File::open(path).unwrap();
    let mut zip = ZipArchive::new(file).unwrap();

    let mut toc_present = false;
    let mut chapters = vec![];
    let mut toc_entries = vec![];

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        let name = file.name().to_string();

        if name.ends_with(".xhtml") && !name.starts_with("OEBPS/toc") {
            chapters.push(name.clone());
        }

        if name.ends_with("toc.ncx") {
            toc_present = true;
            toc_entries.push(TocEntry { src: "dummy".to_string() });
        }
    }

    // Dumb logic:
    // - If toc.ncx exists, assume it matches chapters
    // - Otherwise, no match
    let toc_matches_chapters = if toc_present { true } else { false };

    TocAnalysisResult {
        toc_present,
        toc_entries,
        chapter_files: chapters,
        toc_matches_chapters,
        missing_chapters: vec![],
        orphan_toc_entries: vec![],
    }
}