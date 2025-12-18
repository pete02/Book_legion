#[cfg(test)]
mod audio_tests {
     mod positive_tests{
        use std::fs; 
        use crate::test::helpers::test_helpers;
        use tribune_logistica::{audio_handler::get_audio_chunk,audio_handler::get_audio_chunks_conf};
        #[test]
        fn positive_test_chunk(){
            let (_,b,_,map)=test_helpers::setup_test_book();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();

            assert!(get_audio_chunk(&b, &map,b.initial_chapter as usize, 1, &output, true).is_ok(), "Audiobook chunk not ok");
            assert!(fs::exists(output).is_ok());
            
        }

        #[test]
        fn positive_test_chunks(){
            let (_,b,_,map)=test_helpers::setup_test_book();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();
            let res=get_audio_chunks_conf(&b, &map,10, &output);
            assert!(res.is_ok());
            let vec=res.unwrap();
            assert!(vec.len() ==10)
        }
    }
    

    #[cfg(test)]
    mod negative_tests {
        use crate::test::helpers::test_helpers;
        use tribune_logistica::audio_handler::get_audio_chunks_conf;

        #[test]
        fn errors_on_invalid_chapter() {
            let (_dir, mut status, _data,map) = test_helpers::setup_test_book();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();

            status.chapter = 999; // chapter not in map
            let res = get_audio_chunks_conf(&status, &map,5,  &output);
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "no such chapter");
        }

        #[test]
        fn respects_max_chunk() {
            let (_dir, status, _data,map) = test_helpers::setup_test_book();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();
            let res = get_audio_chunks_conf(&status,&map, 20, &output).unwrap();
            let last_chunk = res.last().unwrap();
            assert!(last_chunk.reached_end);
            assert_eq!(last_chunk.place, format!("{},{}", status.chapter, status.chapter_to_chunk[&status.chapter]));
        }
    }

}



/*
mod sanity_checks{
    use std::fs;

    use crate::test::helpers::test_helpers::get_real_data;
    use tribune_logistica::{audio_handler::get_audio_chunk,audio_handler::get_audio_chunks_conf};

    #[test]
    fn sanity_check_chunk(){
        let t=get_real_data("mageling", "./data", "books.json");
        assert!(get_audio_chunk(&t, t.initial_chapter as usize, 1, "./test.mp3", true).is_ok());
        assert!(fs::exists("./test.mp3").is_ok());
        let _=fs::remove_file("./test.mp3");
    }

    #[test]
    fn sanity_check_chunks(){
        let t=get_real_data("mageling", "./data", "books.json");
        let res=get_audio_chunks_conf(&t, 10, "./test.mp3");

        assert!(res.is_ok());
        let vec=res.unwrap();
        assert!(vec.len() <=10);
    }
}
 */