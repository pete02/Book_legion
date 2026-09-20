use crate::core::types::ErrorCode;
use std::io::Write;
use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Helper function to create a valid test EPUB archive
fn create_valid_test_epub(temp_dir: &TempDir, name: &str) -> String {
    let epub_path = temp_dir.path().join(name);
    let mut archive = ZipWriter::new(std::fs::File::create(&epub_path).unwrap());
    let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add mimetype
    archive.start_file("mimetype", options).unwrap();
    archive.write_all(b"application/epub+zip").unwrap();

    // Add container.xml
    archive
        .start_file("META-INF/container.xml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>",
        )
        .unwrap();

    // Add content.opf
    archive
        .start_file("content.opf", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n    <dc:identifier id=\"uid\">test-123</dc:identifier>\n    <dc:title>Test Book</dc:title>\n    <dc:language>en</dc:language>\n  </metadata>\n  <manifest>\n    <item id=\"chapter1\" href=\"chapter1.html\" media-type=\"application/xhtml+xml\"/>\n    <item id=\"cover-image\" href=\"cover.jpg\" media-type=\"image/jpeg\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n  </manifest>\n  <spine>\n    <itemref idref=\"chapter1\"/>\n  </spine>\n  <nav xmlns=\"http://www.idpf.org/2016/ncx\" type=\"nav\"/>\n</package>",
        )
        .unwrap();

    // Add nav document
    archive
        .start_file("nav.xhtml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>Nav</title></head>\n<body>\n<nav type=\"toc\">\n<ol>\n<li><a href=\"chapter1.html\">Chapter 1</a></li>\n</ol>\n</nav>\n</body>\n</html>",
        )
        .unwrap();

    // Add chapter file
    archive
        .start_file("chapter1.html", options)
        .unwrap();
    archive
        .write_all(b"<!DOCTYPE html><html><head><title>Chapter 1</title></head><body><h1>Chapter 1</h1></body></html>")
        .unwrap();

    // Add cover image
    archive
        .start_file("cover.jpg", options)
        .unwrap();
    archive.write_all(b"fake-jpeg-data").unwrap();

    archive.finish().unwrap();
    epub_path.to_str().unwrap().to_string()
}

/// Helper function to create an EPUB with spine-manifest mismatch
fn create_spine_manifest_mismatch_epub(temp_dir: &TempDir, name: &str) -> String {
    let epub_path = temp_dir.path().join(name);
    let mut archive = ZipWriter::new(std::fs::File::create(&epub_path).unwrap());
    let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add mimetype
    archive.start_file("mimetype", options).unwrap();
    archive.write_all(b"application/epub+zip").unwrap();

    // Add container.xml
    archive
        .start_file("META-INF/container.xml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>",
        )
        .unwrap();

    // Add content.opf with spine referencing non-existent manifest item
    archive
        .start_file("content.opf", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n    <dc:identifier id=\"uid\">test-123</dc:identifier>\n    <dc:title>Test Book</dc:title>\n    <dc:language>en</dc:language>\n  </metadata>\n  <manifest>\n    <item id=\"chapter1\" href=\"chapter1.html\" media-type=\"application/xhtml+xml\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n  </manifest>\n  <spine>\n    <itemref idref=\"nonexistent-chapter\"/>\n  </spine>\n  <nav xmlns=\"http://www.idpf.org/2016/ncx\" type=\"nav\"/>\n</package>",
        )
        .unwrap();

    // Add nav document
    archive
        .start_file("nav.xhtml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>Nav</title></head>\n<body>\n<nav type=\"toc\">\n<ol>\n<li><a href=\"chapter1.html\">Chapter 1</a></li>\n</ol>\n</nav>\n</body>\n</html>",
        )
        .unwrap();

    archive.finish().unwrap();
    epub_path.to_str().unwrap().to_string()
}

/// Helper function to create an EPUB with manifest-archive mismatch
fn create_manifest_archive_mismatch_epub(temp_dir: &TempDir, name: &str) -> String {
    let epub_path = temp_dir.path().join(name);
    let mut archive = ZipWriter::new(std::fs::File::create(&epub_path).unwrap());
    let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add mimetype
    archive.start_file("mimetype", options).unwrap();
    archive.write_all(b"application/epub+zip").unwrap();

    // Add container.xml
    archive
        .start_file("META-INF/container.xml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>",
        )
        .unwrap();

    // Add content.opf with manifest referencing non-existent file
    archive
        .start_file("content.opf", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n    <dc:identifier id=\"uid\">test-123</dc:identifier>\n    <dc:title>Test Book</dc:title>\n    <dc:language>en</dc:language>\n  </metadata>\n  <manifest>\n    <item id=\"chapter1\" href=\"missing-chapter.html\" media-type=\"application/xhtml+xml\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n  </manifest>\n  <spine>\n    <itemref idref=\"chapter1\"/>\n  </spine>\n  <nav xmlns=\"http://www.idpf.org/2016/ncx\" type=\"nav\"/>\n</package>",
        )
        .unwrap();

    // Add nav document
    archive
        .start_file("nav.xhtml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>Nav</title></head>\n<body>\n<nav type=\"toc\">\n<ol>\n<li><a href=\"missing-chapter.html\">Chapter 1</a></li>\n</ol>\n</nav>\n</body>\n</html>",
        )
        .unwrap();

    archive.finish().unwrap();
    epub_path.to_str().unwrap().to_string()
}

/// Helper function to create an EPUB with nav-spine mismatch
fn create_nav_spine_mismatch_epub(temp_dir: &TempDir, name: &str) -> String {
    let epub_path = temp_dir.path().join(name);
    let mut archive = ZipWriter::new(std::fs::File::create(&epub_path).unwrap());
    let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add mimetype
    archive.start_file("mimetype", options).unwrap();
    archive.write_all(b"application/epub+zip").unwrap();

    // Add container.xml
    archive
        .start_file("META-INF/container.xml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>",
        )
        .unwrap();

    // Add content.opf
    archive
        .start_file("content.opf", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n    <dc:identifier id=\"uid\">test-123</dc:identifier>\n    <dc:title>Test Book</dc:title>\n    <dc:language>en</dc:language>\n  </metadata>\n  <manifest>\n    <item id=\"chapter1\" href=\"chapter1.html\" media-type=\"application/xhtml+xml\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n  </manifest>\n  <spine>\n    <itemref idref=\"chapter1\"/>\n  </spine>\n</package>",
        )
        .unwrap();

    // Add nav document with link to non-spine item
    archive
        .start_file("nav.xhtml", options)
        .unwrap();
    archive
        .write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>Nav</title></head>\n<body>\n<nav type=\"toc\">\n<ol>\n<li><a href=\"non-spine-chapter.html\">Non-Spine Chapter</a></li>\n</ol>\n</nav>\n</body>\n</html>",
        )
        .unwrap();

    // Add chapter file
    archive
        .start_file("chapter1.html", options)
        .unwrap();
    archive
        .write_all(b"<!DOCTYPE html><html><head><title>Chapter 1</title></head><body><h1>Chapter 1</h1></body></html>")
        .unwrap();

    archive.finish().unwrap();
    epub_path.to_str().unwrap().to_string()
}

mod spine_manifest_tests {
    use super::*;

    #[test]
    fn test_spine_manifest_match_valid() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_valid_test_epub(&temp_dir, "valid-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get spine and manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let spine_info = crate::epub::validate_spine(&opf_package.spine_items).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        // Get nav info
        let nav_info = crate::epub::validate_navigation(&mut archive, &opf_package, &manifest_info).unwrap();

        let result = crate::consistency::validate_consistency(
            &spine_info,
            &manifest_info,
            &mut archive,
            &nav_info,
        );

        assert!(!result.has_errors(), "Valid EPUB should pass consistency check");
    }

    #[test]
    fn test_spine_manifest_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_spine_manifest_mismatch_epub(&temp_dir, "mismatch-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get spine and manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let spine_info = crate::epub::validate_spine(&opf_package.spine_items).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        // Get nav info
        let nav_info = crate::epub::validate_navigation(&mut archive, &opf_package, &manifest_info).unwrap();

        let result = crate::consistency::validate_consistency(
            &spine_info,
            &manifest_info,
            &mut archive,
            &nav_info,
        );

        assert!(
            result.has_errors(),
            "EPUB with spine-manifest mismatch should fail"
        );
        assert!(result.errors.iter().any(|e| {
            e.code == ErrorCode::SpineManifestMismatch
        }), "Should report SpineManifestMismatch error");
    }
}

mod manifest_archive_tests {
    use super::*;

    #[test]
    fn test_manifest_archive_match_valid() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_valid_test_epub(&temp_dir, "valid-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        let result = crate::consistency::validate_manifest_archive(&manifest_info, &mut archive);

        assert!(!result.has_errors(), "Valid EPUB should pass manifest-archive consistency");
    }

    #[test]
    fn test_manifest_archive_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_manifest_archive_mismatch_epub(&temp_dir, "mismatch-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        let result = crate::consistency::validate_manifest_archive(&manifest_info, &mut archive);

        assert!(
            result.has_errors(),
            "EPUB with manifest-archive mismatch should fail"
        );
        assert!(result.errors.iter().any(|e| {
            e.code == ErrorCode::ManifestArchiveMismatch
        }), "Should report ManifestArchiveMismatch error");
    }
}

mod nav_spine_tests {
    use super::*;

    #[test]
    fn test_nav_spine_match_valid() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_valid_test_epub(&temp_dir, "valid-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get spine and manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let spine_info = crate::epub::validate_spine(&opf_package.spine_items).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        // Get nav info
        let nav_info = crate::epub::validate_navigation(&mut archive, &opf_package, &manifest_info).unwrap();

        let result = crate::consistency::validate_consistency(
            &spine_info,
            &manifest_info,
            &mut archive,
            &nav_info,
        );

        assert!(!result.has_errors(), "Valid EPUB should pass consistency check");
    }

    #[test]
    fn test_nav_spine_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = create_nav_spine_mismatch_epub(&temp_dir, "mismatch-test.epub");

        // Open archive for reading
        let file = std::fs::File::open(&epub_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Parse OPF to get spine and manifest info
        let opf_data = crate::archive::read_entry(&mut archive, "content.opf").unwrap();
        let opf_package = crate::epub::parse_opf(&opf_data).unwrap();
        let spine_info = crate::epub::validate_spine(&opf_package.spine_items).unwrap();
        let manifest_info = crate::epub::validate_manifest(&opf_package.manifest_items).unwrap();

        // Get nav info
        let nav_info = crate::epub::validate_navigation(&mut archive, &opf_package, &manifest_info).unwrap();

        let result = crate::consistency::validate_consistency(
            &spine_info,
            &manifest_info,
            &mut archive,
            &nav_info,
        );

        assert!(result.has_errors(), "EPUB with nav-spine mismatch should fail");
        assert!(result.errors.iter().any(|e| {
            e.code == ErrorCode::NavSpineMismatch
        }), "Should report NavSpineMismatch error");
    }
}
