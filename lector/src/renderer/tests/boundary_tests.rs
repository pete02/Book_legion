
#[cfg(test)]
mod forward_boundary_tests {
    use crate::renderer::*;
    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn assert_boundary(
        input: &str,
        limit: usize,
        expected: Option<usize>,
    ) {
        let actual = sentence_boundaries::find_last_sentence_boundary(input, limit, 0);

        let visualize = |pos: Option<usize>| {
            match pos {
                Some(idx) if idx <= input.len() && input.is_char_boundary(idx) => {
                    format!(
                        "{}|{}",
                        &input[..idx],
                        &input[idx..]
                    )
                }
                Some(idx) => format!("<invalid boundary: {}>", idx),
                None => "<none>".to_string(),
            }
        };

        let visualize_limit = |limit: usize| {
            if limit <= input.len() && input.is_char_boundary(limit) {
                format!(
                    "{}^{}",
                    &input[..limit],
                    &input[limit..]
                )
            } else {
                format!("<invalid limit: {}>", limit)
            }
        };

        assert_eq!(
            actual,
            expected,
            concat!(
                "\ninput            : {:?}",
                "\nlimit            : {}",
                "\nlimit visual     : {}",
                "\nexpected         : {:?}",
                "\nexpected visual  : {}",
                "\nactual           : {:?}",
                "\nactual visual    : {}\n"
            ),
            input,
            limit,
            visualize_limit(limit),
            expected,
            visualize(expected),
            actual,
            visualize(actual),
        );
    }
        // -------------------------------------------------------------------------
    // Core behavior
    // -------------------------------------------------------------------------
 
    mod core{
        use super::*;
        #[test]
        fn finds_last_period_before_limit() {
            let s = "First sentence. Second sentence.";

            assert_boundary(s, s.len(), Some(s.len()));
        }

        #[test]
        fn returns_none_when_no_boundary_exists() {
            let s = "No sentence ending here";

            assert_boundary(s, s.len(), None);
        }

        #[test]
        fn respects_limit() {
            let s = "First sentence. Second sentence."; 

            // limit before '.'
            assert_boundary(s, 10, None);

            // limit exactly at '.'
            assert_boundary(s, 15, Some(15));

            // limit after '.'
            assert_boundary(s, 20, Some(16));
        }

    }
    // -------------------------------------------------------------------------
    // Sentence terminators
    // -------------------------------------------------------------------------
    mod sentence_terminators {
        use super::*;
        #[test]
        fn recognizes_standard_terminators() {
            assert_boundary("Hello!", 6, Some(6));
            assert_boundary("What?", 5, Some(5));
            assert_boundary("Done.", 5, Some(5));
        }

        #[test]
        fn handles_repeated_punctuation() {
            assert_boundary("What?! Really?", 8, Some(7));
            assert_boundary("Wow!!! Nice", 6, Some(6));
        }

        #[test]
        fn includes_closing_quotes_and_parens() {
            assert_boundary(r#"He said "Hello.""#, 17, Some(16));

            assert_boundary("He said (Hello!)", 17, Some(16));
        }

    }
    // -------------------------------------------------------------------------
    // Abbreviations
    // -------------------------------------------------------------------------
    mod abbreviations {
        use super::*;
       
        #[test]
        fn does_not_break_on_common_titles() {
            let s = "Dr. Smith went home.";

            assert_boundary(s, s.len(), Some(20));
        }

        #[test]
        fn does_not_break_on_multi_part_abbreviations() {
            let s = "The Ph.D. candidate graduated. Then he published.";

            assert_boundary(s, s.len()-4, Some(31));
        }

        #[test]
        fn does_not_break_on_latin_abbreviations() {
            let s = "Examples, e.g. apples, are useful.";

            assert_boundary(s, s.len(), Some(34));
        }

        #[test]
        fn abbreviations_are_case_insensitive() {
            let s = "DR. Smith left.";

            assert_boundary(s, s.len(), Some(15));
        }
    }

    // -------------------------------------------------------------------------
    // Numbers
    // -------------------------------------------------------------------------

   mod numbers{
        use super::*;
        #[test]
        fn ignores_decimal_points() {
            let s = "Value is 3.14. Next sentence.";

            assert_boundary(s, s.len(), Some(29));
        }

        #[test]
        fn ignores_version_numbers() {
            let s = "Using version 2.4.1. Deployment succeeded.";

            assert_boundary(s, s.len(), Some(42));
        }
   }

    // -------------------------------------------------------------------------
    // Ellipses
    // -------------------------------------------------------------------------

    #[test]
    fn handles_ellipses() {
        let s = "Wait... stop.";

        assert_boundary(s, s.len(), Some(13));
    }

    // -------------------------------------------------------------------------
    // HTML handling
    // -------------------------------------------------------------------------

    mod html {
        use super::*;
        #[test]
        fn includes_html_tags_after_boundary() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            assert_boundary(s, s.len(), Some(s.len()));
        }

        #[test]
        fn includes_only_html_tags_after_boundary() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            assert_boundary(s, s.len()-6, Some(19));
        }

                #[test]
        fn does_not_include_text_after_boundary_if_html_tag_is_not_set() {
            let s = "<p>Hello world. This is a very big test to see where it binds.</p><p>Next.</p>";
            assert_boundary(s, 51, Some(16));
        }
    }

    // -------------------------------------------------------------------------
    // Unicode
    // -------------------------------------------------------------------------

    mod unicode{
        use super::*;
        #[test]
        fn handles_unicode_quotes() {
            let s = "He said «Hello!»";

            assert_boundary(s, s.len(), Some(s.len()));
        }

        #[test]
        fn handles_unicode_whitespace() {
            let s = "Hello.\u{00A0}World.";

            assert_boundary(s, s.len()-2, Some(8));
        }

    }
    // -------------------------------------------------------------------------
    // Regression tests
    // -------------------------------------------------------------------------

    mod regression{
        use super::*;
        #[test]
        fn regression_et_al() {
            let s = "Smith et al. published results.";

            assert_boundary(s, s.len(), Some(31));
        }

        #[test]
        fn regression_st_louis() {
            let s = "They traveled to St. Louis.";

            assert_boundary(s, s.len(), Some(27));
        }
    }
    mod traling_tags {
        use super::*;
        #[test]
        fn includes_closing_p_after_sentence() {
            let html = "Hello world.</p><p>Next";
            
            assert_boundary(html, html.len(), Some(16));
        }
    }
}



#[cfg(test)]
mod backward_boundary_tests {
    use crate::renderer::*;
    

 fn assert_backward_boundary(
        input: &str,
        limit_from_end: usize,
        expected: Option<usize>,
    ) {
        // Convert limit from "from end" to "from start"
        let actual = sentence_boundaries::find_first_sentence_boundary(input, limit_from_end);

        let visualize = |pos: Option<usize>| {
            match pos {
                Some(idx) if idx <= input.len() && input.is_char_boundary(idx) => {
                    format!(
                        "{}|{}",
                        &input[..idx],
                        &input[idx..]
                    )
                }
                Some(idx) => format!("<invalid boundary: {}>", idx),
                None => "<none>".to_string(),
            }
        };

        let visualize_limit = |limit_from_end: usize| {
            let limit = input.len().saturating_sub(limit_from_end);
            if limit <= input.len() && input.is_char_boundary(limit) {
                format!(
                    "{}^{}",
                    &input[..limit],
                    &input[limit..]
                )
            } else {
                format!("<invalid limit: {}>", limit)
            }
        };

        assert_eq!(
            actual,
            expected,
            concat!(
                "\ninput            : {:?}",
                "\nlimit_from_end   : {}",
                "\nlimit_visual     : {}",
                "\nexpected         : {:?}",
                "\nexpected visual  : {}",
                "\nactual           : {:?}",
                "\nactual visual    : {}\n"
            ),
            input,
            limit_from_end,
            visualize_limit(limit_from_end),
            expected,
            visualize(expected),
            actual,
            visualize(actual),
        );
    }
     
    mod core{
        use super::*;

        #[test]
        fn whole_text_fits() {
            let s = "First sentence. Second sentence.";

            assert_backward_boundary(s, s.len(), Some(0));
        }

        #[test]
        fn finds_first_period_after_limit() {
            let s = "First sentence. Second sentence.";

            assert_backward_boundary(s, s.len()-3, Some(16));
        }

        #[test]
        fn returns_none_when_no_boundary_exists() {
            let s = "No sentence ending here";

            assert_backward_boundary(s, 0, None);
        }

        #[test]
        fn respects_limit() {
            let s = "First sentence. Second sentence."; 

            // limit before first '.'
            assert_backward_boundary(s, 10, None);

            // limit exactly at first '.'
            assert_backward_boundary(s, 19, Some(16));

            // limit after first '.'
            assert_backward_boundary(s, 20, Some(16));
        }

    }
    // -------------------------------------------------------------------------
    // Sentence terminators
    // -------------------------------------------------------------------------
    mod sentence_terminators {
        use super::*;
        #[test]
        fn recognizes_standard_terminators() {
            assert_backward_boundary("Hello! test.", 9, Some(7));
            assert_backward_boundary("What? test.", 9, Some(6));
            assert_backward_boundary("Done. test.", 9, Some(6));
        }

        #[test]
        fn handles_repeated_punctuation() {
            assert_backward_boundary("What?! Really?!", 10, Some(7));
            assert_backward_boundary("Wow!!! Nice!!", 11, Some(7));
        }

        #[test]
        fn includes_closing_quotes_and_parens() {
            assert_backward_boundary(r#"He said "Hello." Then this is."#, 20, Some(17));

            assert_backward_boundary("He said (Hello!) Then we test", 20, Some(17));
        }

    }
    // -------------------------------------------------------------------------
    // Abbreviations
    // -------------------------------------------------------------------------
    mod abbreviations {
        use super::*;
       
        #[test]
        fn does_not_break_on_common_titles() {
            let s = "test sentence. Dr. Smith went home.";

            assert_backward_boundary(s, 30, Some(15));
        }

        #[test]
        fn does_not_break_on_multi_part_abbreviations() {
            let s = "Then he published. The Ph.D. candidate graduated.";

            assert_backward_boundary(s, 33, Some(19));
        }

        #[test]
        fn does_not_break_on_latin_abbreviations() {
            let s = "test sentence. Examples, e.g. apples, are useful.";

            assert_backward_boundary(s, 44, Some(15));
        }

        #[test]
        fn abbreviations_are_case_insensitive() {
            let s = "DR. Smith left.";

            assert_backward_boundary(s, s.len()-2, None);
        }
    }

    // -------------------------------------------------------------------------
    // Numbers
    // -------------------------------------------------------------------------

   mod numbers{
        use super::*;
        #[test]
        fn ignores_decimal_points() {
            let s = "Value is 3.14. Next sentence.";

            assert_backward_boundary(s, s.len()-3, None);
        }

        #[test]
        fn ignores_version_numbers() {
            let s = "Using version 2.4.1. Deployment succeeded.";

            assert_backward_boundary(s, s.len()-3, None);
        }

        #[test]
        fn ignores_version_numbers_and_snaps_to_previous_sentence() {
            let s = "This is a test. Using version 2.4.1. Deployment succeeded.";

            assert_backward_boundary(s, 46, Some(16));
        }
   }

    // -------------------------------------------------------------------------
    // Ellipses
    // -------------------------------------------------------------------------
   //broken
    #[test]
    fn handles_ellipses() {
        let s = "Wait...stop.";

        assert_backward_boundary(s, 9, None);
    }

    // -------------------------------------------------------------------------
    // HTML handling
    // -------------------------------------------------------------------------

    mod html {
        use super::*;

        #[test]
        fn includes_fitting_html_tags_after_boundary() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            assert_backward_boundary(s, 18, Some(19));
        }

        #[test]
        fn includes_not_fitting_html_tags_after_boundary() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            assert_backward_boundary(s, 11, Some(19));
        }


        #[test]
        fn includes_only_html_tags_after_boundary() {
            let s = "<p>Hello world.</p><p>Next.</p>";
            assert_backward_boundary(s, 19, Some(19));
        }
    }

    // -------------------------------------------------------------------------
    // Unicode
    // -------------------------------------------------------------------------

    mod unicode{
        use super::*;
        #[test]
        fn handles_unicode_quotes() {
            let s = "He said «Hello!»";

            assert_backward_boundary(s, s.len(), Some(0));
        }

        #[test]
        fn handles_unicode_whitespace() {
            let s = "Hello.\u{00A0}World.";

            assert_backward_boundary(s, 9, Some(8));
        }

    }
    // -------------------------------------------------------------------------
    // Regression tests
    // -------------------------------------------------------------------------

    mod regression{
        use super::*;
        #[test]
        fn regression_et_al() {
            let s = "Smith et al. published results.";

            assert_backward_boundary(s, 0, None);
        }

        #[test]
        fn regression_st_louis() {
            let s = "They traveled to St. Louis.";

            assert_backward_boundary(s, 0, None);
        }
    }
    mod trailing_tags {
        use super::*;
        #[test]
        fn includes_closing_p_after_sentence() {
            let html = "Hello world.</p><p>Next";
            
            assert_backward_boundary(html, 16, Some(16));
        }
    }
}
