// src/archive/testkit.rs

#[cfg(test)]
mod tests{
    use super::super::*;
    use std::{assert_eq, io::{Cursor, Write}};
    use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};


    #[derive(Default)]
    pub struct ZipBuilder {
        entries: Vec<(String, Vec<u8>, CompressionMethod)>,
        symlinks: Vec<(String, String)>,
    }

    impl ZipBuilder {
        pub fn new() -> Self { Self::default() }

        pub fn stored(mut self, name: &str, data: &[u8]) -> Self {
            self.entries.push((name.into(), data.into(), CompressionMethod::Stored));
            self
        }
        pub fn deflated(mut self, name: &str, data: &[u8]) -> Self {
            self.entries.push((name.into(), data.into(), CompressionMethod::Deflated));
            self
        }
        pub fn symlink(mut self, name: &str, target: &str) -> Self {
            self.symlinks.push((name.into(), target.into()));
            self
        }

        pub fn build(self) -> Vec<u8> {
            let mut zw = ZipWriter::new(Cursor::new(Vec::new()));
            for (name, data, method) in self.entries {
                zw.start_file(name, SimpleFileOptions::default().compression_method(method)).unwrap();
                zw.write_all(&data).unwrap();
            }
            for (name, target) in self.symlinks {
                zw.add_symlink(name, target, SimpleFileOptions::default()).unwrap();
            }
            zw.finish().unwrap().into_inner()
        }
    }

    /// Zip entries are order-independent on read, so this is fine for tests.
    pub fn minimal_valid() -> Vec<u8> {
        ZipBuilder::new()
            .stored("mimetype", b"application/epub+zip")
            .deflated("META-INF/container.xml", b"<container/>")
            .build()
    }

    #[test]
    fn reads_entries_into_map() {
        let files = verify_archive(&minimal_valid(), &Limits::default()).unwrap();
        assert_eq!(files.len(),2, "expected 1 fie, got {:?}", files);
        assert!(files.contains(&"mimetype".to_string()) && files.contains(&"META-INF/container.xml".to_string()), "Expected to contain mimetype and container.xml, got: {:?}",files);
    }

    #[test]
    fn reader_rejects_zips_with_too_many_entries(){
        let mut lims=Limits::default();
        lims.entry_count=10;
        let zip=ZipBuilder::new()
            .deflated("1", b"<container/>")
            .deflated("2", b"<container/>")
            .deflated("3", b"<container/>")
            .deflated("4", b"<container/>")
            .deflated("5", b"<container/>")
            .deflated("6", b"<container/>")
            .deflated("7", b"<container/>")
            .deflated("8", b"<container/>")
            .deflated("9", b"<container/>")
            .deflated("10", b"<container/>")
            .deflated("11", b"<container/>")
            .build();

        let err= verify_archive(&zip, &lims);

        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), ArchiveError::TooManyFiles);

    }
    #[test]
    fn reader_accepts_with_exact_lfile_limit_count(){
        let mut lims=Limits::default();
        let filenames= ["1","2","3","4","5"];
        lims.entry_count=5;
        let mut zip=ZipBuilder::new();
        for n in filenames{
            zip=zip.deflated(n, b"test");
        }

        let files= verify_archive(&zip.build(), &lims);

        assert!(files.is_ok());


    }

    #[test]
    fn normalize_paths_ok() {
        let tests = [
            ("root/folder/../file", "root/file"),
            ("root/./folder/file", "root/folder/file"),
            ("root/folder/../../file", "file"),
            ("root//folder///file", "root/folder/file"),
            ("file", "file"),
            ("./file", "file"),
        ];

        for (input, expected) in tests {
            assert_eq!(
                normalize_path(input).unwrap(),
                PathBuf::from(expected),
                "failed for input: {input}"
            );
        }
    }
    #[test]
    fn reject_path_errs(){
        let tests=[
            ("root/../../file", ArchiveError::PathTraversal),
            ("/root/file", ArchiveError::AbsolutePath),
            ("root\\backslash",ArchiveError::BackSlash),
            ("root/nullbyte\0", ArchiveError::NullByte),
            ("root/file\tname", ArchiveError::ControlCharacter),
        ];

        for (input,expected) in tests{
            assert_eq!(
                normalize_path(input).unwrap_err(),
                expected,
                "Failed for input {input}"
            )
        }

    }

    use zip::write::{ExtendedFileOptions, FileOptions};

    #[test]
    fn is_symlink_detects_symlink() {
        let mut bytes = Cursor::new(Vec::new());

        {
            let mut zip = ZipWriter::new(&mut bytes);

            zip.add_symlink(
                "link",
                "target",
                FileOptions::<ExtendedFileOptions>::default(),
            )
            .unwrap();

            zip.finish().unwrap();
        }

        let mut zip = ZipArchive::new(Cursor::new(bytes.into_inner())).unwrap();
        let entry = zip.by_index(0).unwrap();

        assert!(is_symlink(&entry));
    }

    #[test]
    fn verify_archive_rejects_symlink() {
        let mut bytes = Cursor::new(Vec::new());

        {
            let mut zip = ZipWriter::new(&mut bytes);

            zip.start_file(
                "normal.txt",
                FileOptions::<ExtendedFileOptions>::default(),
            )
            .unwrap();
            zip.write_all(b"normal").unwrap();

            zip.add_symlink(
                "link",
                "normal.txt",
                FileOptions::<ExtendedFileOptions>::default(),
            )
            .unwrap();

            zip.finish().unwrap();
        }

        let limits = Limits::default();

        assert_eq!(
            verify_archive(&bytes.into_inner(), &limits),
            Err(ArchiveError::SymLink)
        );
    }

    #[test]
    fn rejects_total_uncompressed_size_limit() {
        let bytes = ZipBuilder::new()
            .stored("one", b"12345")
            .stored("two", b"12345")
            .build();

        let limits = Limits {
            size_limit: 9,
            per_entry_limit: 10,
            compression_rate: 100,
            entry_count: 10_000,
        };

        assert_eq!(
            verify_archive(&bytes, &limits),
            Err(ArchiveError::SizeLimit)
        );
    }

    #[test]
    fn rejects_per_entry_size_limit() {
        let bytes = ZipBuilder::new()
            .stored("large", b"1234567890")
            .build();

        let limits = Limits {
            size_limit: 100,
            per_entry_limit: 5,
            compression_rate: 100,
            entry_count: 10_000,
        };

        assert_eq!(
            verify_archive(&bytes, &limits),
            Err(ArchiveError::EntrySizeLimit)
        );
    }

    #[test]
    fn rejects_compression_rate_limit() {
        let data = vec![b'A'; 1_000];

        let bytes = ZipBuilder::new()
            .deflated("highly_compressible", &data)
            .build();

        let limits = Limits {
            size_limit: 10_000,
            per_entry_limit: 10_000,
            compression_rate: 2,
            entry_count: 10_000,
        };

        assert_eq!(
            verify_archive(&bytes, &limits),
            Err(ArchiveError::CompressionRate)
        );
    }

    #[test]
    fn allows_entry_at_per_entry_limit() {
        let bytes = ZipBuilder::new()
            .stored("file", b"12345")
            .build();

        let limits = Limits {
            size_limit: 5,
            per_entry_limit: 5,
            compression_rate: 100,
            entry_count: 10_000,
        };

        assert!(verify_archive(&bytes, &limits).is_ok());
    }

    #[test]
    fn allows_total_size_at_limit() {
        let bytes = ZipBuilder::new()
            .stored("one", b"12345")
            .stored("two", b"12345")
            .build();

        let limits = Limits {
            size_limit: 10,
            per_entry_limit: 5,
            compression_rate: 100,
            entry_count: 10_000,
        };

        assert!(verify_archive(&bytes, &limits).is_ok());
    }
}

