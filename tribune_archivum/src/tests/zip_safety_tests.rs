use anyhow::Result;
use zip::{ZipWriter, write::SimpleFileOptions, CompressionMethod};
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;



const MAX_FILES: usize = 2000;
const MAX_TOTAL_UNCOMPRESSED: u64 = 512 * 1024 * 1024; // 512MB
const MAX_SINGLE_FILE: u64 = 50 * 1024 * 1024; // 50MB
const MAX_COMPRESSION_RATIO: f64 = 100.0;


use crate::lib::verifiers::validate_zip_safety;
fn create_zip<F>(path: &std::path::Path, builder: F)
where
    F: FnOnce(&mut ZipWriter<File>) -> Result<()>,
{
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    builder(&mut zip).unwrap();
    zip.finish().unwrap();
}

#[test]
fn zip_safety_valid_archive() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("valid.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored);

        zip.start_file("file.txt", options)?;
        zip.write_all(b"hello world")?;
        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_ok());
}

#[test]
fn zip_safety_too_many_files() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("too_many.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default();

        for i in 0..(MAX_FILES + 1) {
            zip.start_file(format!("f{}.txt", i), options)?;
            zip.write_all(b"x")?;
        }
        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}

#[test]
fn zip_safety_total_uncompressed_limit() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("too_big_total.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default();

        let chunk = vec![0u8; (MAX_TOTAL_UNCOMPRESSED / 2 + 1) as usize];

        zip.start_file("a.bin", options)?;
        zip.write_all(&chunk)?;

        zip.start_file("b.bin", options)?;
        zip.write_all(&chunk)?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}

#[test]
fn zip_safety_single_file_limit() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("single_big.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default();

        let data = vec![0u8; (MAX_SINGLE_FILE + 1) as usize];

        zip.start_file("huge.bin", options)?;
        zip.write_all(&data)?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}

#[test]
fn zip_safety_high_compression_ratio() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("ratio.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);

        // Highly compressible data
        let data = vec![b'A'; 2_000_000];

        zip.start_file("bomb.txt", options)?;
        zip.write_all(&data)?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}

#[test]
fn zip_safety_path_traversal_dotdot() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("traversal.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default();

        zip.start_file("../evil.txt", options)?;
        zip.write_all(b"bad")?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}


#[test]
fn zip_safety_path_traversal_absolute() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("absolute.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default();

        zip.start_file("/root.txt", options)?;
        zip.write_all(b"bad")?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_err());
}

#[test]
fn zip_safety_zero_compressed_size() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("stored.zip");

    create_zip(&zip_path, |zip| {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored);

        zip.start_file("plain.txt", options)?;
        zip.write_all(b"normal file")?;

        Ok(())
    });

    assert!(validate_zip_safety(&zip_path).is_ok());
}