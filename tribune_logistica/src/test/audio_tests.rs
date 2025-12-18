#[cfg(test)]
mod audio_tests {
     mod positive_tests{
        use std::fs; 
        use crate::test::helpers::test_helpers;
        use tribune_logistica::{audio_handler::get_audio_chunk,audio_handler::get_audio_chunks_conf};
        #[test]
        fn positive_test_chunk(){
            let (a,b,c)=test_helpers::setup_test_book();
            let base: String=a.path().to_string_lossy().to_string();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();

            assert!(get_audio_chunk(&b, b.initial_chapter as usize, 1, &output, true, &base).is_ok());
            assert!(fs::exists(output).is_ok());
            
        }

        #[test]
        fn positive_test_chunks(){
            let (a,b,c)=test_helpers::setup_test_book();
            let base: String=a.path().to_string_lossy().to_string();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();
            let res=get_audio_chunks_conf(Some(&b), 10, &base,&output);
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
        fn errors_on_none_input() {
            let res = get_audio_chunks_conf(None, 5, ".","books.mp3");
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "No book");
        }

        #[test]
        fn errors_on_invalid_chapter() {
            let (_dir, mut status, _data) = test_helpers::setup_test_book();
            let base: String=_dir.path().to_string_lossy().to_string();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();

            status.chapter = 999; // chapter not in map
            let res = get_audio_chunks_conf(Some(&status), 5, &base, &output);
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "no such chapter");
        }

        #[test]
        fn respects_max_chunk() {
            let (_dir, status, _data) = test_helpers::setup_test_book();
            let base: String=_dir.path().to_string_lossy().to_string();
            let tmp_file = tempfile::NamedTempFile::new().unwrap();
            let output = tmp_file.path().to_string_lossy().to_string();
            let res = get_audio_chunks_conf(Some(&status), 20, &base,&output).unwrap();
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
        assert!(get_audio_chunk(&t, t.initial_chapter as usize, 1, "./test.mp3", true, "./data").is_ok());
        assert!(fs::exists("./test.mp3").is_ok());
        let _=fs::remove_file("./test.mp3");
    }

    #[test]
    fn sanity_check_chunks(){
        let t=get_real_data("mageling", "./data", "books.json");
        let res=get_audio_chunks_conf(Some(&t), 10, "./data");

        assert!(res.is_ok());
        let vec=res.unwrap();
        assert!(vec.len() <=10);
    }
}
*/