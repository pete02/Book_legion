#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use crate::models::*;
    use crate::book_handler;
    use serde_json::json;
    use tempfile::NamedTempFile;
    use std::io::Write;


    fn write_books_temp(data: serde_json::Value) -> NamedTempFile {
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "{}", data.to_string()).unwrap();
        tmpfile
    }


    #[test]
    fn test_incorret_initbook_entires(){

        let tmp=write_books_temp( json!({
            "non_existent": {
                "path": "doo",
                "initial_chapter": 4,
                "current_chunk": 0,
                "current_chapter": 4,
                "duration": 1560.1995,
                "max_chapter": 100,
                "chapter_to_chunk":{
                    "4":142
                    }
            }
        }));
        let path=tmp.path().to_str().unwrap().to_string();
        assert_eq!(book_handler::init_book("missing","text","missing.json").unwrap_err(),json!({
            "status":"missing library manifest: missing.json",
            "chapter":-1,
            "chunk":-1
        }));

        assert_eq!(book_handler::init_book("missing","taika", &path).unwrap_err(),json!({
            "status":"incorrect format",
            "chapter":-1,
            "chunk":-1
        }));

        assert_eq!(book_handler::init_book("missing","text",&path).unwrap_err(),json!({
            "status":"not in library",
            "chapter":-1,
            "chunk":-1
        }));

        assert_eq!(book_handler::init_book("non_existent","text",&path).unwrap_err(),json!({
            "status":"missing book",
            "chapter":-1,
            "chunk":-1
        }));

        assert_eq!(book_handler::init_book("non_existent","audio",&path).unwrap_err(),json!({
            "status":"missing audiobook",
            "chapter":-1,
            "chunk":-1
        }));


    }

    #[test]
    fn test_correct_initbook_entry(){
        let tmpfile =write_books_temp( json!({
            "mageling": {
                "path": "mageling",
                "initial_chapter":4,
                "current_chunk": 0,
                "duration": 1560.1995,
                "current_chapter": 4,
                "max_chapter": 10,
                "chapter_to_chunk":{
                    "4":142
                    }
            }
        }));
        let path = tmpfile.path().to_str().unwrap();
        println!("{}",path);
        assert_eq!(book_handler::init_book("mageling","text",path).unwrap(),BookStatus{
            time: 0.0,
            name: "mageling".to_string(),
            path: "mageling".to_string(),
            chapter:4,
            chunk:0,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        });


        assert_eq!(book_handler::init_book("mageling","audio",path).unwrap(),BookStatus{
            time: 0.0,
            name: "mageling".to_string(),
            path: "mageling".to_string(),
            chapter:4,
            chunk:0,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        });
    }

    #[test]
    fn test_incorrect_update_progress(){
        let tmpfile =write_books_temp( json!({
            "mageling": {
                "path": "mageling",
                "initial_chapter":4,
                "current_chunk": 0,
                "duration": 1560.1995,
                "current_chapter": 4,
                "max_chapter": 10,
                "chapter_to_chunk":{
                    "4":142
                    }
            }
        }));
        let path=tmpfile.path().to_str().unwrap();
        assert_eq!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 3,
            chunk: 3,
            json: "missing".to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(),"missing manifest");

        assert_eq!(book_handler::update_progress(None).unwrap_err(),"no initialized book");
        assert_eq!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"missing".to_string(),
            path: "nonsense".to_string(),
            chapter: 3,
            chunk: 3,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(),"not in library");

        assert_eq!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 100,
            chunk: 3,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(),"chapter overflow");

        assert_eq!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 3,
            chunk: 1000,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(),"invalid chapter number");

        assert_eq!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 1000,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(),"chunk overflow");

    }

    #[test]
    fn test_correct_update_progress(){
        let tmpfile =write_books_temp( json!({
            "mageling": {
                "path": "mageling",
                "initial_chapter":4,
                "current_chunk": 0,
                "current_chapter": 4,
                "duration": 1560.1995,
                "max_chapter": 10,
                "chapter_to_chunk":{
                    "4":142
                    }
            }
        }));
        let path=tmpfile.path().to_str().unwrap();
        assert!(book_handler::update_progress(Some(BookStatus{
            time: 0.0,
            name:"mageling".to_string(),
            path: "nonsense".to_string(),
            chapter: 4,
            chunk: 5,
            json: path.to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).is_ok());
        
        let json_str=json!({
            "mageling": {
                "path": "mageling",
                "initial_chapter":4,
                "current_chunk": 5,
                "current_chapter": 4,
                "duration": 1560.1995,
                "max_chapter": 10,
                "chapter_to_chunk":{
                    "4":142
                    }
            }}).to_string();

        let test_books: HashMap<String, BookData>=serde_json::from_str(&json_str).unwrap();
        let books=book_handler::load_books(path).unwrap();

        let test_book: &BookData=test_books.get("mageling").unwrap();
        let book: &BookData=books.get("mageling").unwrap();
        assert_eq!(test_book.current_chunk,book.current_chunk);
        assert_eq!(test_book.current_chapter,book.current_chapter);

    }

    #[test]
    fn test_get_chapter(){
        assert_eq!(book_handler::get_chapter(None).unwrap_err(),"no initialized book");
        assert_eq!(book_handler::get_chapter(Some(BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "nonsense".to_string(),
            chapter: 4,
            chunk: 5,
            json: "books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(), "Failed to open EPUB in the path");
        
        assert_eq!(book_handler::get_chapter(Some(BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 1000000000,
            chunk: 5,
            json: "books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).unwrap_err(), "chapter too large");


        assert!(book_handler::get_chapter(Some(BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 5,
            json: "books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        })).is_ok());
    }

    

    #[test]
    fn test_create_temp_mp3(){
        assert!(book_handler::get_audio_chunk(None, 10,20).is_err());

        assert!(book_handler::get_audio_chunk(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 30,
            json: "books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,20).is_err());

        assert!(!book_handler::get_audio_chunk(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 10,
            json: "../data/books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,20).unwrap().reached_end);


        assert!(book_handler::get_audio_chunk(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 100000000,
            json: "books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,100000001).is_err());


        assert!(book_handler::get_audio_chunk(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 10,
            json: "../data/books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,20).is_ok());

        assert!(book_handler::get_audio_chunk(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 10,
            json: "../data/books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,100000).unwrap().reached_end);


        assert!(book_handler::get_audio_chunk_config(Some(&BookStatus{
            time: 0.0, 
            name:"mageling".to_string(),
            path: "mageling".to_string(),
            chapter: 4,
            chunk: 10,
            json: "../data/books.json".to_owned(),
            max_chapter: 25,
            duration: 100.0
        }), 4,20,true,"test.mp3".to_owned()).is_ok());
    }
}
