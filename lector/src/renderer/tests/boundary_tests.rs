mod boundary_tests {
    use super::*;
    mod basic_tests{
        use crate::renderer::sentence_boundaries::*;

        #[test]
        fn valid_simple_sentence_end() {
            let s = "Hello world.";
            let pos = s.find('.').unwrap();
            assert!(is_valid_boundary(s, pos));
        }


        #[test]
        fn invalid_valid_simple_sentence_end() {
            let s = "Hello world.";
            let pos = s.find('.').unwrap();
            assert!(!is_valid_boundary(s, pos-2));
        }

        #[test]
        fn valid_exclamation_mark() {
            let s = "Hello!";
            let pos = s.find('!').unwrap();
            assert!(is_valid_boundary(s, pos));
        }

        #[test]
        fn valid_question_mark() {
            let s = "What?";
            let pos = s.find('?').unwrap();
            assert!(is_valid_boundary(s, pos));
        }
    }

    mod abbriviations{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn invalid_abbreviation_dr() {
            let s = "Dr. Smith arrived.";
            let pos = s.find('.').unwrap(); // after "Dr"
            assert!(!is_valid_boundary(s, pos));
        }

        #[test]
        fn invalid_abbreviation_us() {
            let s = "The U.S. economy grew.";
            let pos = s.find("U.S.").unwrap() + 1; // first dot
            assert!(!is_valid_boundary(s, pos));
        }
    }


    mod ellipses{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn invalid_ellipsis() {
            let s = "Wait...";
            let pos = s.find('.').unwrap();
            assert!(!is_valid_boundary(s, pos));
        }
    }

    mod numbers{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn invalid_decimal_number() {
            let s = "Value is 3.14 today.";
            let pos = s.find('.').unwrap(); // middle dot
            assert!(!is_valid_boundary(s, pos));
        }

        #[test]
        fn valid_number_sentence_end() {
            let s = "It costs 3.";
            let pos = s.rfind('.').unwrap();
            assert!(is_valid_boundary(s, pos));
        }


        #[test]
        fn invalid_version_number() {
            let s = "Using version 2.4.1. Done.";
            let pos = s.find("2.4.1.").unwrap() + "2.4.1".len();
            assert!(!is_valid_boundary(s, pos));
        }
    }

    mod html{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn invalid_inside_html_tag() {
            let s = "orld</p>Hello w";
            let pos = s.find('p').unwrap();

            assert!(!is_valid_boundary(s, pos));
        }

        #[test]
        fn valid_between_html_tag() {
            let s = "orld</p><p>Hello w";
            assert!(!is_valid_boundary(s, 8));
        }


        #[test]
        fn valid_after_html_content() {
            let s = "<p>Hello world.</p>";
            let pos = s.find("world.").unwrap() + "world".len();
            assert!(is_valid_boundary(s, pos));
        }

        #[test]
        fn invalid_mixed_html_and_number() {
            let s = "<p>Version 2.4.1.</p>";
            let pos = s.find("2.4.1.").unwrap() + "2.4.1".len();
            assert!(!is_valid_boundary(s, pos));
        }


    }
    mod test_valid_cut{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn valid_between_html() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            let a=s[0..19].to_string();
            println!("s: {a}");
            assert!(is_valid_cut(s, 19));
        }

        #[test]
        fn valid_inside_html() {
            let s = "<p>Hello world.Next</p>";
            let pos = s.find("world.").unwrap() + "world".len();
            let a=s[0..pos].to_string();
            println!("s: {a}");
            assert!(is_valid_cut(s, pos));
        }
    }
    mod edge_cases{
        use crate::renderer::sentence_boundaries::*;
        #[test]
        fn punctuation_is_only_candidate_not_guarantee() {
            let s = "Hello world. Dr. Smith arrived. Value is 3.14.";

            for (i, c) in s.char_indices() {
                if matches!(c, '.' | '!' | '?') {
                    // we only assert "safe behavior", not correctness
                    let _ = is_valid_boundary(&s, i);
                }
            }
        }
    }
}




mod move_boundary_tests{
    use super::*;
    use crate::renderer::sentence_boundaries::*;

    pub fn visualize_boundary(text: &str, pos: usize) -> String {
        let mut out = String::new();

        let mut i = 0;

        while i < text.len() {
            if i == pos {
                out.push('|');
            }

            let c = text[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }

        if pos == text.len() {
            out.push('|');
        }

        out
    }


    pub fn assert_boundary_eq(
        text: &str,
        expected: Option<usize>,
        actual: Option<usize>,
    ) {
        if expected == actual {
            return;
        }

        let mut msg = String::new();

        msg.push_str("input:\n");
        msg.push_str(text);
        msg.push_str("\n\n");

        msg.push_str("expected:\n");
        match expected {
            Some(p) => msg.push_str(&visualize_boundary(text, p)),
            None => msg.push_str("<none>"),
        }

        msg.push_str("\n\nactual:\n");
        match actual {
            Some(p) => msg.push_str(&visualize_boundary(text, p)),
            None => msg.push_str("<none>"),
        }

        panic!("{msg}");
    }

    #[test]
    fn resolution_moves_past_quotes() {
        let s = "Hello world.\" Next sentence.";
        let pos = s.find('.').unwrap();

        let resolved = resolve_boundary(s, pos);

        assert_boundary_eq(s, Some(s.find("Next").unwrap()), Some(resolved));
    }

    #[test]
    fn resolution_moves_past_html() {
        let s = "<p>Hello world.</p> Next sentence.";
        let pos = s.find('.').unwrap();

        let resolved = resolve_boundary(s, pos);

        assert_boundary_eq(s, Some(s.find(" Next").unwrap()), Some(resolved));
    }

    #[test]
    fn resolution_collapses_multiple_punctuation() {
        let s = "Wait!!! Next.";
        let pos = s.find('!').unwrap();

        let resolved = resolve_boundary(s, pos);

        assert_boundary_eq(s, Some(s.find("Next").unwrap()), Some(resolved));
    }

    #[test]
    fn resolution_is_direction_invariant() {
        let s = "Hello world. Next sentence.";

        let fwd = resolve_boundary(s, s.find('.').unwrap());
        let bwd = resolve_boundary(s, s.find("Next").unwrap() - 1);

        assert_boundary_eq(s, Some(fwd), Some(bwd));
    }

    #[test]
    fn resolution_nested_html() {
        let s = "<p>Hello <b>world.</b></p> Next.";
        let pos = s.find('.').unwrap();

        let resolved = resolve_boundary(s, pos);

        assert_boundary_eq(s, Some(s.find(" Next").unwrap()), Some(resolved));
    }
    
}