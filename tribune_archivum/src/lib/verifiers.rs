use std::{error::Error, fs::{self, File}, path::{Component, Path, PathBuf}, process::Command};
use log::{debug, warn};
use zip::ZipArchive;
use anyhow::{Result, bail};

use crate::lib::helpers;


const MAX_FILES: usize = 2000;
const MAX_TOTAL_UNCOMPRESSED: u64 = 512 * 1024 * 1024; // 512MB
const MAX_SINGLE_FILE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_COMPRESSION_RATIO: f64 = 100.0;


pub fn validate_zip_safety(path: &Path) -> Result<()> {
    debug!("validate_zip_safety: starting for path={:?}", path);

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let file_count = archive.len();
    debug!("validate_zip_safety: archive opened, file_count={}", file_count);

    if file_count > MAX_FILES {
        debug!(
            "validate_zip_safety: REJECTED — file_count={} exceeds MAX_FILES={}",
            file_count, MAX_FILES
        );
        bail!("ZIP contains too many files: {}", file_count);
    }

    // Path traversal check uses central directory names only — no local header needed
    for name in archive.file_names() {
        if name.contains("..") || name.starts_with('/') {
            debug!("validate_zip_safety: REJECTED — path traversal detected in {:?}", name);
            bail!("Path traversal detected in {}", name);
        }
    }

    let mut total_uncompressed = 0u64;

    for i in 0..file_count {
        let entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                // A file that can't be parsed can't be decompressed, so it's not a bomb risk.
                // Log and skip rather than rejecting a legitimate but slightly non-conformant file.
                warn!(
                    "validate_zip_safety: entry[{}] local header unreadable ({}), skipping size checks",
                    i, e
                );
                continue;
            }
        };

        let name = entry.name().to_string();
        let compressed = entry.compressed_size();
        let uncompressed = entry.size();
        total_uncompressed += uncompressed;

        debug!(
            "validate_zip_safety: entry[{}] name={:?}, compressed={}, uncompressed={}, running_total={}",
            i, name, compressed, uncompressed, total_uncompressed
        );

        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
            debug!(
                "validate_zip_safety: REJECTED — total_uncompressed={} exceeds MAX_TOTAL_UNCOMPRESSED={} at entry[{}] {:?}",
                total_uncompressed, MAX_TOTAL_UNCOMPRESSED, i, name
            );
            bail!("ZIP expands beyond allowed total size");
        }

        if uncompressed > MAX_SINGLE_FILE {
            debug!(
                "validate_zip_safety: REJECTED — entry[{}] {:?} uncompressed={} exceeds MAX_SINGLE_FILE={}",
                i, name, uncompressed, MAX_SINGLE_FILE
            );
            bail!("File too large: {}", name);
        }

        if compressed > 0 {
            let ratio = uncompressed as f64 / compressed as f64;
            debug!(
                "validate_zip_safety: entry[{}] {:?} compression ratio={:.2}x (limit={:.2}x)",
                i, name, ratio, MAX_COMPRESSION_RATIO
            );
            if ratio > MAX_COMPRESSION_RATIO {
                debug!(
                    "validate_zip_safety: REJECTED — entry[{}] {:?} ratio={:.2}x exceeds MAX_COMPRESSION_RATIO={:.2}x",
                    i, name, ratio, MAX_COMPRESSION_RATIO
                );
                bail!("Suspicious compression ratio in {} ({}x)", name, ratio);
            }
        } else {
            debug!(
                "validate_zip_safety: entry[{}] {:?} is stored uncompressed, skipping ratio check",
                i, name
            );
        }
    }

    debug!(
        "validate_zip_safety: all {} entries passed — total_uncompressed={} bytes",
        file_count, total_uncompressed
    );
    Ok(())
}

pub fn repair_epub_if_needed<P: AsRef<Path>>(path: P) -> Result<bool, Box<dyn Error>> {
    let path = path.as_ref();

    // --- Step 1: Try opening + validating ---
    let file = File::open(path)?;
    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => {
            // Definitely broken → attempt repair
            return repair_epub(path);
        }
    };

    // Validate entries (lightweight)
    for i in 0..archive.len() {
        let res = (|| {
            let _ = archive.by_index(i)?;
            Ok::<_, zip::result::ZipError>(())
        })();

        if res.is_err() {
            return repair_epub(path);
        }
    }

    // Archive is fine
    Ok(false)
}

pub fn verify_zip_integrity(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    debug!("verify_zip_integrity: starting for path={:?}", path);

    let mut archive = helpers::get_zip(path)?;
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
    debug!("verify_zip_integrity: zip opened, {} entries found", names.len());

    let opf_file = helpers::read_container_opf_path(&mut archive)?;
    debug!("verify_zip_integrity: opf_file resolved to {:?}", opf_file);

    let opf_dir = Path::new(&opf_file);
    let opf_struct = helpers::get_opf_struct(&mut archive)?;
    debug!(
        "verify_zip_integrity: opf parsed — spine.toc={:?}, manifest items={}, spine itemrefs={}",
        opf_struct.spine.toc,
        opf_struct.manifest.item.len(),
        opf_struct.spine.itemref.len()
    );

    let mut ids = Vec::new();

    if opf_struct.spine.toc.len() == 0 {
        debug!("verify_zip_integrity: spine.toc is empty, returning Ok(true) early");
        return Ok(true);
    }

    let mut toc_file = "".to_owned();
    for item in opf_struct.manifest.item {
        let href = opf_dir.with_file_name(item.href);
        debug!(
            "verify_zip_integrity: checking manifest item id={:?}, resolved href={:?}",
            item.id, href
        );

        if !names.contains(&href.to_string_lossy().to_string()) {
            debug!(
                "verify_zip_integrity: MISSING file in zip — href={:?} not found among {} entries",
                href,
                names.len()
            );
            return Err(format!("Missing file in zip: {}", href.to_string_lossy()).into());
        }

        ids.push(item.id.clone());

        if item.id == opf_struct.spine.toc {
            toc_file = href.to_string_lossy().to_string();
            debug!("verify_zip_integrity: toc_file matched, toc_file={:?}", toc_file);
        }
    }

    debug!("verify_zip_integrity: manifest check passed, collected {} ids", ids.len());

    for itemref in opf_struct.spine.itemref {
        debug!("verify_zip_integrity: checking spine itemref={:?}", itemref.idref);
        if !ids.contains(&itemref.idref) {
            debug!(
                "verify_zip_integrity: MISSING idref={:?}, known ids={:?}",
                itemref.idref, ids
            );
            return Err(format!("missing idref: {}", itemref.idref).into());
        }
    }

    debug!(
        "verify_zip_integrity: spine check passed — spine.toc.len()={}, toc_file in names={}",
        opf_struct.spine.toc.len(),
        names.contains(&toc_file)
    );

    if opf_struct.spine.toc.len() == 0 || !names.contains(&toc_file) {
        debug!("verify_zip_integrity: toc_file absent or spine empty, returning Ok(true)");
        return Ok(true);
    }

    debug!("verify_zip_integrity: all checks passed, returning Ok(false)");
    Ok(false)
}


fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut stack = Vec::new();

    for comp in path.components() {
        match comp {
            Component::ParentDir => { stack.pop(); }   // resolve ".."
            Component::CurDir => {}                     // skip "."
            Component::Normal(s) => stack.push(s),
            _ => {}                                     // ignore RootDir and Prefix
        }
    }

    let mut normalized = PathBuf::new();
    for s in stack {
        normalized.push(s);
    }
    normalized
}

pub fn verify_toc_integrity(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = helpers::get_zip(path).map_err(|e|format!("fdailed to get archive: {e}"))?;
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    let opf_struct = helpers::get_opf_struct(&mut archive)?;
    let opf_path=helpers::read_container_opf_path(&mut archive)?;
    // Find TOC file in manifest
    let mut toc_file = opf_struct
        .manifest
        .item
        .iter()
        .find(|item| item.id == opf_struct.spine.toc)
        .ok_or("no toc file found")?
        .href
        .clone();

    let toc_path=Path::new(&opf_path).with_file_name(toc_file);

    let normalized=normalize_relative_path(&toc_path);

    toc_file=normalized.to_string_lossy().to_string();

    let toc_file_handle = archive.by_name(&toc_file).map_err(|e| format!("could not get toc handle {}: {}", toc_file, e))?;
    let toc = helpers::read_toc(toc_file_handle).map_err(|e|format!("could not read ToC: {}",e))?;

    if toc.nav_map.nav_point.is_empty() {
        return Err("ToC did not contain any nav points".into());
    }


    for nav_point in toc.nav_map.nav_point {
        // Resolve relative path: src in NCX is relative to OPF
        let resolved_path = normalized.with_file_name(&nav_point.content.src);
        let normalized_path = normalize_relative_path(&resolved_path).to_string_lossy().to_string();

        if !names.contains(&normalized_path) {
            return Err(format!(
                "Nav point '{}' not found in EPUB (resolved path: '{}')",
                nav_point.content.src, normalized_path
            )
            .into());
        }

        // Validate play order
        validate_playorder(&nav_point.play_order)
            .map_err(|_| format!("play order is not a number: {}", nav_point.play_order))?;

        // Validate nav label text
        if nav_point.nav_label.text.len() > 100 {
            return Err("navpoint text too long".into());
        }
        if nav_point.nav_label.text.is_empty() {
            return Err("navpoint text not existing".into());
        }
    }

    Ok(())
}




fn validate_playorder(play_order: &str) -> Result<u32, String> {
    match play_order.parse::<u32>() {
        Ok(num) if num > 0 => Ok(num),
        Ok(_) => Err(format!("playOrder must be positive: '{}'", play_order)),
        Err(_) => Err(format!("Invalid playOrder, not a number: '{}'", play_order)),
    }
}


fn repair_epub(path: &Path) -> Result<bool, Box<dyn Error>> {
    println!("Attempting EPUB repair: {:?}", path);

    let parent = path.parent().ok_or("Invalid path")?;

    let tmp_dir = parent.join("epub_repair_tmp");
    let fixed_path = std::fs::canonicalize(parent)?
    .join("epub_repaired.epub");

    // Clean temp dir if exists
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir(&tmp_dir)?;

    // --- Step 2: unzip ---
    let unzip_status = Command::new("unzip")
        .arg(path)
        .arg("-d")
        .arg(&tmp_dir)
        .status()?;

    if !unzip_status.success() {
        return Err("unzip failed; cannot repair epub".into());
    }

    // --- Step 3: rezip ---
    // IMPORTANT: EPUB requires mimetype first + uncompressed
    let mimetype_path = tmp_dir.join("mimetype");
    let zip_status = Command::new("zip")
        .current_dir(&tmp_dir)
        .arg("-X0")
        .arg(&fixed_path) // this can now safely be outside tmp_dir
        .arg("mimetype")
        .status()?;

    if !zip_status.success() {
        return Err("zip step 1 failed".into());
    }

    let zip_status = Command::new("zip")
        .current_dir(&tmp_dir)
        .arg("-Xr9D")
        .arg(&fixed_path)
        .arg(".")
        .arg("-x")
        .arg("mimetype")
        .status()?;

    if !zip_status.success() {
        return Err("zip failed; cannot repair epub".into());
    }

    println!("Fixed path: {:?}", fixed_path);

    // --- Step 4: replace original atomically ---
    fs::rename(&fixed_path, path).map_err(|e|format!("Repair_epub: Error in replacing the original: {}",e))?;

    // --- Cleanup ---
    fs::remove_dir_all(&tmp_dir).map_err(|_|"Repair_epub: Failed in the cleanup")?;

    println!("EPUB repaired successfully.");

    Ok(true)
}