
#[cfg(test)]
mod last_fitting_char_tests_no_children {
    use super::*;
    use crate::renderer::calculate_page_height::*;
    use std::sync::Arc;

    fn create_mock(text_len: u32, char_bottoms: &[f64], children: Vec<LayoutQuery>) ->LayoutQuery {
        let char_bottoms = char_bottoms.to_vec(); // owned, no lifetime

        LayoutQuery {
            text_len,
            char_start: 0,
            top: 0.0,
            bottom: char_bottoms.last().cloned().unwrap_or(0.0),
            children,
            get_char_bottom: Arc::new(move |offset| char_bottoms[offset as usize]),
        }
    }

    // -------------------------------------------------------------------------
    // Core behavior
    // -------------------------------------------------------------------------
    #[test]
    fn handles_empty_text_and_children() {
        let layout = create_mock(0, &[], vec![]);
        let result = last_fitting_char(&layout, 100.0);
        assert_eq!(result, FitResult::AllFit);
    }

    #[test]
    fn returns_none_when_all_text_fits() {
        let layout = create_mock(
            5,
            &[10.0, 20.0, 30.0, 40.0, 50.0],
            vec![]
        );

        // viewport_bottom at 100 fits all characters
        let result = last_fitting_char(&layout, 100.0);
        assert_eq!(result, FitResult::AllFit);
    }

    #[test]
    fn returns_none_when_no_char_fits() {
        let layout = create_mock(
            5,
            &[100.0, 110.0, 120.0, 130.0, 140.0],
            vec![]
        );

        // viewport_bottom at 50 fits no characters
        let result = last_fitting_char(&layout, 50.0);
        assert_eq!(result, FitResult::NoneFit);
    }


    #[test]
    fn finds_last_fitting_char() {
        let layout = create_mock(
            10,
            &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0],
            vec![]
        );

        // viewport_bottom at 55 should fit chars 0-4 (bottoms 10-50)
        let result = last_fitting_char(&layout, 55.0);
        assert_eq!(result, FitResult::LastFitting(4));
    }

    #[test]
    fn finds_exact_boundary() {
        let layout = create_mock(
            5,
            &[10.0, 20.0, 30.0, 40.0, 50.0],
            vec![]
        );

        // viewport_bottom exactly at last char's bottom
        let result = last_fitting_char(&layout, 50.0);
        assert_eq!(result, FitResult::LastFitting(4));
    }

    #[test]
    fn handles_single_char() {
        let layout = create_mock(1, &[50.0], vec![]);
        assert_eq!(last_fitting_char(&layout, 60.0), FitResult::AllFit); // fits all
        assert_eq!(last_fitting_char(&layout, 40.0), FitResult::NoneFit); // fits none
        assert_eq!(last_fitting_char(&layout, 50.0), FitResult::LastFitting(0)); // exact
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    

    #[test]
    fn handles_gaps_in_char_bottoms() {
        let layout = create_mock(
            5,
            &[10.0, 100.0, 110.0, 120.0, 130.0],
            vec![]
        );

        // Only first char fits
        let result = last_fitting_char(&layout, 50.0);
        assert_eq!(result, FitResult::LastFitting(0));
    }

    #[test]
    fn handles_consecutive_fitting_chars() {
        let layout = create_mock(
            10,
            &[10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0],
            vec![]
        );

        // viewport_bottom at 15 fits chars 0-5
        let result = last_fitting_char(&layout, 15.0);
        assert_eq!(result, FitResult::LastFitting(5));
    }

    #[test]
    fn handles_unicode_characters() {
        let layout = create_mock(
            3,
            &[10.0, 20.0, 30.0],
            vec![]
        );

        // Unicode chars should work the same as ASCII
        let result = last_fitting_char(&layout, 25.0);
        assert_eq!(result, FitResult::LastFitting(1));
    }

    #[test]
    fn handles_large_text() {
        let char_bottoms: Vec<f64> = (0..100).map(|i| (i + 1) as f64 * 10.0).collect();
        let layout = create_mock(100, &char_bottoms, vec![]);

        // viewport_bottom at 500 should fit chars 0-49
        let result = last_fitting_char(&layout, 500.0);
        assert_eq!(result, FitResult::LastFitting(49));
    }

    #[test]
    fn handles_zero_viewport() {
        let layout = create_mock(5, &[10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
        let result = last_fitting_char(&layout, 0.0);
        assert_eq!(result, FitResult::NoneFit);
    }

    #[test]
    fn handles_negative_viewport() {
        let layout = create_mock(5, &[10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
        let result = last_fitting_char(&layout, -10.0);
        assert_eq!(result, FitResult::NoneFit);
    }

    // -------------------------------------------------------------------------
    // Boundary conditions
    // -------------------------------------------------------------------------

    #[test]
    fn returns_none_when_char_bottom_exceeds_viewport() {
        let layout = create_mock(
            5,
            &[10.0, 20.0, 30.0, 40.0, 50.0],
            vec![]
        );

        // viewport_bottom at 49.999 should not fit char at 50
        let result = last_fitting_char(&layout, 49.999);
        assert_eq!(result, FitResult::LastFitting(3));
    }

    #[test]
    fn char_exactly_on_boundary_fits() {
        let layout = create_mock(5, &[10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

        // char bottom == viewport bottom, should count as fitting
        let result = last_fitting_char(&layout, 50.0);
        assert_eq!(result, FitResult::LastFitting(4)); // everything fits
    }

        #[test]
    fn char_just_under_boundary_fits() {
        let layout = create_mock(5, &[10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

        // one pixel under
        let result = last_fitting_char(&layout, 50.1);
        assert_eq!(result, FitResult::AllFit); // char at 50.0 doesn't fit, cutoff at index 4
    }

    #[test]
    fn char_just_over_boundary_does_not_fit() {
        let layout = create_mock(5, &[10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

        // one pixel under
        let result = last_fitting_char(&layout, 49.9);
        assert_eq!(result, FitResult::LastFitting(3)); // char at 50.0 doesn't fit, cutoff at index 4
    }
        // -------------------------------------------------------------------------
    // Regression tests
    // -------------------------------------------------------------------------

    #[test]
    fn regression_single_line() {
        let layout = create_mock(
            1,
            &[50.0],
            vec![]
        );

        // Single line that fits
        let result = last_fitting_char(&layout, 100.0);
        assert_eq!(result, FitResult::AllFit);
    }

    #[test]
    fn regression_multiple_lines() {
        let layout = create_mock(
            20,
            &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0,
              110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0],
            vec![]
        );

        // viewport_bottom at 150 should fit chars 0-14
        let result = last_fitting_char(&layout, 150.0);
        assert_eq!(result, FitResult::LastFitting(14));
    }


    #[test]
    fn single_oversized_char_returns_none_fit() {
        let layout = create_mock(1, &[2000.0], vec![]);
        assert_eq!(last_fitting_char(&layout, 800.0), FitResult::NoneFit);
    }
}

// ... existing code ...
#[cfg(test)]
mod last_fitting_char_tests_with_children {
    use std::sync::Arc;
    use super::*;
    use crate::renderer::calculate_page_height::*;

    /// Leaf node (text only)
    fn create_text_leaf(
        text_len: u32,
        char_bottoms: &[f64],
        char_start: u32,
    ) -> LayoutQuery {
        let char_bottoms = char_bottoms.to_vec();

        LayoutQuery {
            text_len,
            char_start,
            top: 0.0,
            bottom: char_bottoms.last().cloned().unwrap_or(0.0),
            children: vec![],
            get_char_bottom: Arc::new(move |offset| char_bottoms[offset as usize]),
        }
    }

    /// Container node (children only, no text)
    fn create_container(
        children: Vec<LayoutQuery>,
    ) -> LayoutQuery {
        let max_child_bottom = children
            .iter()
            .map(|c| c.bottom)
            .fold(0.0, f64::max);

        LayoutQuery {
            text_len: 0,
            char_start: 0,
            top: 0.0,
            bottom: max_child_bottom,
            children,
            get_char_bottom: Arc::new(|_| {
                panic!("Container node must not access char_bottoms")
            }),
        }
    }

    // ------------------------------------------------------------
    // CHILD CONTAINER BEHAVIOR
    // ------------------------------------------------------------

    #[test]
    fn container_with_single_child_full_fit() {
        let child = create_text_leaf(
            3,
            &[10.0, 20.0, 30.0],
            0,
        );

        let parent = create_container(vec![child]);

        let result = last_fitting_char(&parent, 100.0);

        assert_eq!(result, FitResult::AllFit);
    }

    #[test]
    fn child_truncates_parent_layout() {
        let child = create_text_leaf(
            3,
            &[10.0, 20.0, 60.0],
            0,
        );

        let parent = create_container(vec![child]);

        let result = last_fitting_char(&parent, 55.0);

        // child exceeds viewport → truncation happens
        assert_eq!(result, FitResult::LastFitting(1));
    }

    #[test]
    fn multiple_children_respect_max_bottom() {
        let child1 = create_text_leaf(2, &[10.0, 20.0], 0);
        let child2 = create_text_leaf(2, &[30.0, 40.0], 2);

        let parent = create_container(vec![child1, child2]);

        let result = last_fitting_char(&parent, 35.0);

        // child2 bottom = 40 exceeds viewport → partial fit
        assert_eq!(result, FitResult::LastFitting(2));
    }

    #[test]
    fn container_with_children_all_fit() {
        let child1 = create_text_leaf(2, &[10.0, 20.0], 0);
        let child2 = create_text_leaf(2, &[30.0, 40.0], 0);

        let parent = create_container(vec![child1, child2]);

        let result = last_fitting_char(&parent, 100.0);

        assert_eq!(result, FitResult::AllFit);
    }

    #[test]
    fn deeply_nested_children_are_respected() {
        let grandchild = create_text_leaf(2, &[10.0, 70.0], 0);
        let child = create_container(vec![grandchild]);
        let parent = create_container(vec![child]);

        let result = last_fitting_char(&parent, 50.0);

        // grandchild exceeds → truncation
        assert_eq!(result, FitResult::LastFitting(0));
    }

    // ------------------------------------------------------------
    // INVARIANT TESTS
    // ------------------------------------------------------------

    #[test]
    fn container_has_no_text() {
        let child = create_text_leaf(2, &[10.0, 20.0], 0);
        let container = create_container(vec![child]);

        assert_eq!(container.text_len, 0);
    }

    #[test]
    fn leaf_has_no_children() {
        let leaf = create_text_leaf(2, &[10.0, 20.0], 0);

        assert!(leaf.children.is_empty());
    }

    #[test]
    fn container_bottom_is_max_of_children() {
        let child1 = create_text_leaf(2, &[10.0, 20.0], 0);
        let child2 = create_text_leaf(2, &[30.0, 80.0], 0);

        let container = create_container(vec![child1.clone(), child2.clone()]);

        assert!(container.bottom >= child1.bottom);
        assert!(container.bottom >= child2.bottom);
        assert_eq!(container.bottom, 80.0);
    }

    
}


mod inside_html_tests{
    use crate::renderer::sentence_boundaries::*;

    #[test]
    fn inside_simple_open_tag() {
        let s = "<p>Hello world</p>";

        // position inside "<p"
        assert!(is_inside_html_tag_boundary(s, 1));
    }

    #[test]
    fn outside_tag_in_text() {
        let s = "<p>Hello world</p>";

        let pos = s.find("Hello").unwrap();

        assert!(!is_inside_html_tag_boundary(s, pos));
    }
    #[test]
    fn inside_closing_tag() {
        let s = "<p>Hello</p>";

        // position inside "</p>"
        let pos = s.find("</p>").unwrap() + 2;

        assert!(is_inside_html_tag_boundary(s, pos));
    }
    #[test]
    fn after_closing_tag_is_safe() {
        let s = "<p>Hello</p>Next";

        let pos = s.find("Next").unwrap();

        assert!(!is_inside_html_tag_boundary(s, pos));
    }

    #[test]
    fn between_adjacent_tags_is_inside_tag_context() {
        let s = "<p>Hello</p><p>Next</p>";

        // position between </p> and <p>
        let pos = s.find("</p>").unwrap() + "</p>".len();

        assert!(!is_inside_html_tag_boundary(s, pos));
    }
    #[test]
    fn inside_nested_tags() {
        let s = "<div><p>Hello</p></div>";

        let pos = s.find("<p").unwrap() + 1;

        assert!(is_inside_html_tag_boundary(s, pos));
    }
    #[test]
    fn outside_nested_tags_text() {
        let s = "<div><p>Hello</p></div>";

        let pos = s.find("Hello").unwrap();

        assert!(!is_inside_html_tag_boundary(s, pos));
    }

    #[test]
    fn incomplete_open_tag() {
        let s = "<p>Hello world";

        let pos = s.find("<p").unwrap() + 1;

        assert!(is_inside_html_tag_boundary(s, pos));
    }
    #[test]
    fn plain_text_is_never_inside_tag() {
        let s = "Hello world.";

        for i in 0..s.len() {
            assert!(!is_inside_html_tag_boundary(s, i));
        }
    }
}