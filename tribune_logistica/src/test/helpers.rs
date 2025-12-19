pub mod test_helpers {
    use std::{collections::HashMap, path::Path};
    use std::fs::{self, File};
    use std::io::Write;    
    use tempfile::TempDir;
    use tribune_logistica::db_handlers::load_books;
    use tribune_logistica::models::*;
    use zip::ZipWriter;
    use zip::write::FileOptions;
    use std::sync::Arc;
    use tokio::sync::{RwLock, mpsc};
    use tribune_logistica::buffer_handler::{self, FillerCommand};

    #[allow(dead_code)]
    fn generate_temp_book_data(name:&str,)->BookData{
        let mut g=HashMap::new();
        g.insert(1, 10);
        g.insert(2, 10);
        g.insert(3, 10);
        BookData{
            path: name.to_owned(),
            initial_chapter: 1,
            duration: 100.0,
            current_chunk:1,
            current_chapter: 1,
            current_time: 0.0,
            max_chapter: 3,
            chapter_to_chunk: g
        }
    }

    #[allow(dead_code)]
    fn gen_book_status(name:&str, book:&BookData, basedir:&Path, manifest_hapt:&Path)->BookStatus{
        BookStatus{
            name: name.to_owned(),
            path: basedir.to_string_lossy().to_string(),
            chapter: book.current_chapter,
            chunk: book.current_chunk,
            chapter_to_chunk: book.chapter_to_chunk.clone(),
            time: book.current_time,
            initial_chapter: book.initial_chapter,
            json: manifest_hapt.to_string_lossy().to_string(),
            max_chapter: book.max_chapter,
            duration: book.duration
        }
    }

    #[allow(dead_code)]
    fn gen_audio_map(book:&BookStatus,path:&Path)->AudioMap{
        let mut h:HashMap<String,AudioMapEntry>=HashMap::new();

        for i in 1..=10{
            h.insert(format!("{},{}",book.chapter,i), AudioMapEntry { chapter_number: book.chapter as usize, chunk_number: i, start_time: 0.0, duration: 10.0 });
        }

        fs::create_dir_all(path).unwrap();
        let ap=format!("{}/{}.json",path.to_string_lossy(), book.name);
        let map=AudioMap{name:book.name.clone(), map:h};
        serde_json::to_writer_pretty(File::create(ap).unwrap(),&map).unwrap();
        map

    }

    #[allow(dead_code)]
    fn generate_temp_manifest(bookdir:&Path, data:&BookData, name:&str){
        let mut h=HashMap::new();
        h.insert(name.to_owned(), data);
        serde_json::to_writer_pretty(File::create(bookdir).unwrap(),&h).unwrap();
    }
    #[allow(dead_code)]
    fn generate_temp_book_files(book_path:&Path, name:&str){
        let mp3=book_path.join(format!("{}.mp3",name));
        
        std::fs::write(&mp3, vec![1u8; 100]).unwrap();
        let epub=book_path.join(format!("{}.epub",name));
        create_minimal_epub(&epub).unwrap()

    }

    fn create_minimal_epub(path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);

        let stored: FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let deflated: FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // REQUIRED: mimetype must be first and uncompressed
        zip.start_file("mimetype", stored)?;
        zip.write_all(b"application/epub+zip")?;

        // META-INF/container.xml
        zip.start_file("META-INF/container.xml", deflated)?;
        zip.write_all(br#"<?xml version="1.0"?>
    <container version="1.0"
        xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OEBPS/content.opf"
                media-type="application/oebps-package+xml"/>
    </rootfiles>
    </container>"#)?;

        // content.opf
        zip.start_file("OEBPS/content.opf", deflated)?;
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
    <package version="3.0"
            xmlns="http://www.idpf.org/2007/opf"
            unique-identifier="BookId">
    <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:title>Test Book</dc:title>
        <dc:language>en</dc:language>
        <dc:identifier id="BookId">test</dc:identifier>
    </metadata>
    <manifest>
        <item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
    </manifest>
    <spine>
        <itemref idref="c1"/>
    </spine>
    </package>"#)?;

        // chapter
        zip.start_file("OEBPS/chapter1.xhtml", deflated)?;
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
    <html xmlns="http://www.w3.org/1999/xhtml">
    <body>
        <p>Hello chapter one</p>
    </body>
    </html>"#)?;

        zip.finish()?;
        Ok(())
    }

#[allow(dead_code)]
    pub fn setup_test_book()->(TempDir,BookStatus,BookData, AudioMap) {
        let book_name = "testbook";
        let dir=TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(book_name)).unwrap();
        let manifest_path=dir.path().join("books.json");
        let book_path=dir.path().join(book_name);

        let bookdata=generate_temp_book_data(book_name);
        let bookstatus=gen_book_status(book_name, &bookdata,&book_path, &manifest_path);
        
        generate_temp_book_files(&book_path, &book_name);
        generate_temp_manifest(&manifest_path, &bookdata, book_name);
        let amap=gen_audio_map(&bookstatus,&book_path);


        (dir,bookstatus, bookdata, amap)
    }
    #[allow(dead_code)]

    pub fn load_real_data()->(BookStatus,BookData){
        let b=load_books("./data/books.json").unwrap();
        let book=b.get("mageling").unwrap();

        let stat=BookStatus{
            name: "mageling".to_owned(),
            path: "./data/mageling/mageling".to_owned(),
            chapter: book.initial_chapter,
            chunk: 1,
            chapter_to_chunk: book.chapter_to_chunk.clone(),
            time: 0.0,
            initial_chapter: book.initial_chapter,
            json: "./data/books.json".to_owned(),
            max_chapter: book.max_chapter,
            duration: book.duration
        };

        return (stat,book.clone())
    }


    #[allow(dead_code)]
    pub fn get_real_data(name:&str,base:&str,json:&str)->BookStatus{
        let b=load_books(&format!("{}/{}",base,json)).unwrap();
        let book=b.get(name).unwrap();
        BookStatus::new(name, base, book.clone(), json)
    }

    #[allow(dead_code)]
    pub async fn start_filler(
        buffer: Arc<RwLock<AudioBuffer>>,
    ) -> mpsc::Sender<FillerCommand> {
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(buffer_handler::run_filler(rx, buffer));
        tx
    }
    #[allow(dead_code)]
    pub async fn ensure_and_wait(
        tx: &mpsc::Sender<FillerCommand>,
        book: BookKey,
        cursor: ChunkCursor,
    ) -> buffer_handler::SeekDecision {
        use tokio::sync::oneshot;

        let (decision_tx, decision_rx) = oneshot::channel();

        tx.send(FillerCommand::Ensure {
            book,
            start: cursor,
            respond_to: Some(decision_tx),
        }).await.unwrap();

        // Wait for the filler to make its decision
        let decision = decision_rx.await.unwrap();
        decision

    }


    #[allow(dead_code)]
    pub fn parse_place(place: &str) -> (u32, u32) {
        let mut parts = place.split(',');

        let chapter = parts
            .next()
            .expect("place missing chapter")
            .parse::<u32>()
            .expect("invalid chapter in place");

        let chunk = parts
            .next()
            .expect("place missing chunk")
            .parse::<u32>()
            .expect("invalid chunk in place");

        assert!(
            parts.next().is_none(),
            "place contains extra components"
        );

        (chapter, chunk)
    }

}
