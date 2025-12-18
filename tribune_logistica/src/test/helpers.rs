pub mod test_helpers {
    use std::{collections::HashMap, path::Path};
    use std::fs::{self, File};
    
    use tempfile::TempDir;
    use tribune_logistica::db_handlers::load_books;
    use tribune_logistica::models::*;

    #[allow(dead_code)]
    fn generate_temp_book_data(name:&str,)->BookData{
        let mut g=HashMap::new();
        g.insert(2, 10);
        BookData{
            path: name.to_owned(),
            initial_chapter: 2,
            duration: 100.0,
            current_chunk:1,
            current_chapter: 2,
            current_time: 0.0,
            max_chapter: 2,
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
    fn gen_audio_map(book:&BookStatus,path:&Path){
        let mut h:HashMap<String,AudioMapEntry>=HashMap::new();

        for i in 1..=10{
            h.insert(format!("{},{}",book.chapter,i), AudioMapEntry { chapter_number: book.chapter as usize, chunk_number: i, start_time: 0.0, duration: 10.0 });
        }

        fs::create_dir_all(path).unwrap();
        let ap=format!("{}/{}.json",path.to_string_lossy(), book.name);
        

        serde_json::to_writer_pretty(File::create(ap).unwrap(),&AudioMap{name:book.name.clone(), map:h}).unwrap();

    }

    #[allow(dead_code)]
    fn generate_temp_manifest(bookdir:&Path, data:&BookData, name:&str){
        let mut h=HashMap::new();
        h.insert(name.to_owned(), data);
        serde_json::to_writer_pretty(File::create(bookdir).unwrap(),&h).unwrap();
    }

#[allow(dead_code)]
    pub fn setup_test_book()->(TempDir,BookStatus,BookData) {
        let book_name = "testbook";
        let dir=TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(book_name)).unwrap();
        let manifest_path=dir.path().join("books.json");
        let book_path=dir.path().join(book_name);
        let bookdata=generate_temp_book_data(book_name);
        let bookstatus=gen_book_status(book_name, &bookdata,&book_path, &manifest_path);

        let mp3_file = book_path.join(format!("{}.mp3", book_name));
        std::fs::write(mp3_file, vec![1u8; 100]).unwrap();
        generate_temp_manifest(&manifest_path, &bookdata, book_name);
        gen_audio_map(&bookstatus,&book_path);
        (dir,bookstatus, bookdata)
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
}
