 
#[cfg(test)]
mod last_fitting_char_tests_no_children {
    use crate::renderer::find_page_boundary::*;
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;

    fn create_mock(text_len: u32, char_bottoms: Vec<f64>, children: Vec<LayoutQuery>) -> LayoutQuery {
        let char_bottoms = char_bottoms.to_vec(); // owned, no lifetime
        let mut layout=LayoutQuery::default();
        layout.text="x".repeat(text_len as usize);
        layout.bottom=char_bottoms.last().cloned().unwrap_or(0.0);
        layout.children=children;
        layout.get_char_bottom=Arc::new(move |offset| char_bottoms[offset as usize]);
        layout.get_char_top=Arc::new(|_| panic!("container must not call get_char_top"));
        return layout;
    }

    mod core{
        use super::*;
        #[test]
        fn returns_allfit_when_all_text_fits() {
            let layout = create_mock(
                5,
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom at 100 fits all characters
            let result = last_fitting_char(&layout, 100.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn handles_empty_text_and_children() {
            let layout = create_mock(0, vec![], vec![]);
            let result = last_fitting_char(&layout, 100.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn returns_nonefit_when_no_char_fits() {
            let layout = create_mock(
                5,
                vec![100.0, 110.0, 120.0, 130.0, 140.0],
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
                vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0],
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
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom exactly at last char's bottom
            let result = last_fitting_char(&layout, 50.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn handles_single_char() {
            let layout = create_mock(1, vec![50.0], vec![]);
            assert_eq!(last_fitting_char(&layout, 60.0), FitResult::AllFit); // fits all
            assert_eq!(last_fitting_char(&layout, 40.0), FitResult::NoneFit); // fits none
            assert_eq!(last_fitting_char(&layout, 50.0), FitResult::AllFit); // exact
        }


        #[test]
        fn local_index_independent_of_char_start() {
            let mut layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            layout.char_start = 100; // global offset
            let result = last_fitting_char(&layout, 35.0);
            assert_eq!(result, FitResult::LastFitting(2)); // still 2, not 102
        }
    }



    
    mod edge_cases{
        use super::*;
        #[test]
        fn handles_gaps_in_char_bottoms() {
            let layout = create_mock(
                5,
                vec![10.0, 100.0, 110.0, 120.0, 130.0],
                vec![]
            );

            // Only first char fits
            let result = last_fitting_char(&layout, 50.0);
            assert_eq!(result, FitResult::LastFitting(0));
        }

        #[test]
        fn handles_consecutive_fitting_chars() {
            let layout = create_mock(
                11,
                vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 15.0, 16.0, 17.0, 18.0, 19.0],
                vec![]
            );

            // viewport_bottom at 15 fits chars 0-5
            let result = last_fitting_char(&layout, 15.0);
            assert_eq!(result, FitResult::LastFitting(6));
        }

        #[test]
        fn handles_large_text() {
            let char_bottoms: Vec<f64> = (0..100).map(|i| (i + 1) as f64 * 10.0).collect();
            let layout = create_mock(100, char_bottoms, vec![]);

            // viewport_bottom at 500 should fit chars 0-49
            let result = last_fitting_char(&layout, 500.0);
            assert_eq!(result, FitResult::LastFitting(49));
        }

        #[test]
        fn handles_zero_viewport() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            let result = last_fitting_char(&layout, 0.0);
            assert_eq!(result, FitResult::NoneFit);
        }

        #[test]
        fn handles_negative_viewport() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            let result = last_fitting_char(&layout, -10.0);
            assert_eq!(result, FitResult::NoneFit);
        }


        #[test]
        fn single_oversized_char_returns_none_fit() {
            let layout = create_mock(1, vec![2000.0], vec![]);
            assert_eq!(last_fitting_char(&layout, 800.0), FitResult::NoneFit);
        }

    }


    mod boundaries{
        use super::*;
         #[test]
        fn does_not_return_char_with_bottom_exceeding_viewport() {
            let layout = create_mock(
                5,
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom at 49.999 should not fit char at 50
            let result = last_fitting_char(&layout, 49.999);
            assert_eq!(result, FitResult::LastFitting(3));
        }

        #[test]
        fn char_just_under_boundary_fits() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

            // one pixel under
            let result = last_fitting_char(&layout, 50.1);
            assert_eq!(result, FitResult::AllFit); // char at 50.0 doesn't fit, cutoff at index 4
        }

        #[test]
        fn char_just_over_boundary_does_not_fit() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

            // one pixel under
            let result = last_fitting_char(&layout, 49.9);
            assert_eq!(result, FitResult::LastFitting(3)); // char at 50.0 doesn't fit, cutoff at index 4
        }
    }

}


#[cfg(test)]
mod last_fitting_char_tests_with_children {
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;
    use crate::renderer::find_page_boundary::*;

    // =========================================================================
    // Test helpers
    // =========================================================================

    /// Leaf node: has text, no children.
    /// `char_start` is the global offset of this node's first character.
    /// `char_bottoms` is indexed locally (0 = first char of this node).
    fn make_leaf(char_start: u32, char_bottoms: Vec<f64>) -> LayoutQuery {
        let text_len = char_bottoms.len();
        let bottoms = char_bottoms.clone();
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = "x".repeat(text_len);
        layout.top = 0.0;
        layout.bottom = char_bottoms.last().cloned().unwrap_or(0.0);
        layout.children = vec![];
        layout.get_char_bottom = Arc::new(move |i| bottoms[i as usize]);
        layout.get_char_top = Arc::new(|_| panic!("container must not call get_char_top"));
        layout
    }

    /// Container node: has children, no text.
    /// `char_start` defaults to the char_start of the first child (typical case).
    fn make_container(children: Vec<LayoutQuery>) -> LayoutQuery {
        let bottom = children.iter().map(|c| c.bottom).fold(0.0_f64, f64::max);
        let char_start = children.first().map(|c| c.char_start).unwrap_or(0);
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = String::new();
        layout.top = 0.0;
        layout.bottom = bottom;
        layout.children = children;
        layout.get_char_bottom = Arc::new(|_| panic!("container must not call get_char_bottom"));
        layout.get_char_top = Arc::new(|_| panic!("container must not call get_char_top"));
        layout
    }

    mod core {
        use super::*;
        #[test]
        fn container_single_child_all_fit() {
            let child = make_leaf(0, vec![10.0, 20.0, 30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(last_fitting_char(&parent, 100.0), FitResult::AllFit);
        }

        #[test]
        fn container_single_child_no_fit() {
            let child = make_leaf(0, vec![100.0, 200.0]);
            let parent = make_container(vec![child]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::NoneFit);
        }

        #[test]
        fn container_single_child_partial_fit() {
            // child: char_start=10, chars 0-1 fit, char 2 does not.
            // parent: char_start=10.
            // Local offset from parent = (10 + 1) - 10 = 1. Expect LastFitting(1).
            let child = make_leaf(10, vec![10.0, 20.0, 60.0]);
            let parent = make_container(vec![child]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(1));
        }

        #[test]
        fn container_result_is_local_to_parent_not_child() {
            // child1: char_start=100, 3 chars, all fit.
            // child2: char_start=103, first char fits, second does not.
            // Last fitting global char: 103+0 = 103.
            // Local to parent (char_start=100): 103 - 100 = 3. Expect LastFitting(3).
            let child1 = make_leaf(100, vec![10.0, 20.0, 30.0]);
            let child2 = make_leaf(103, vec![40.0, 90.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn container_two_children_first_fits_second_does_not() {
            // child1 fully fits, child2 fully does not.
            // = (0 + 5 - 1) - 0 = 4. Expect LastFitting(4).
            let child1 = make_leaf(0, vec![10.0, 20.0, 30.0, 40.0, 50.0]);
            let child2 = make_leaf(5, vec![200.0, 210.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(last_fitting_char(&parent, 100.0), FitResult::LastFitting(4));
        }


    }

    mod boundary {
        use super::*;
        #[test]
        fn child_last_char_exactly_on_boundary_local_to_parent() {
            // child: char_start=20, bottoms [30.0, 50.0, 70.0].
            // page=50.0: chars 0 and 1 of child fit. Local to parent (char_start=20):
            // (20 + 1) - 20 = 1. Expect LastFitting(1).
            let child = make_leaf(20, vec![30.0, 50.0, 70.0]);
            let parent = make_container(vec![child]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(1));
        }

        #[test]
        fn first_child_end_exactly_on_boundary_local_to_parent() {
            // child1: char_start=0, 3 chars, last bottom=50.0.
            // child2: char_start=3, first bottom=60.0.
            // page=50.0: child1 fully fits exactly, child2 none fit.
            // Local to parent: last fitting = 2 (index of last char of child1). Expect LastFitting(2).
            let child1 = make_leaf(0, vec![20.0, 40.0, 50.0]);
            let child2 = make_leaf(3, vec![60.0, 70.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(2));
        }

        #[test]
        fn zero_page_height_is_none_fit() {
            let leaf = make_leaf(0, vec![10.0, 20.0]);
            assert_eq!(last_fitting_char(&leaf, 0.0), FitResult::NoneFit);
        }

        #[test]
        fn negative_page_height_is_none_fit() {
            let leaf = make_leaf(0, vec![10.0, 20.0]);
            assert_eq!(last_fitting_char(&leaf, -1.0), FitResult::NoneFit);
        }

        #[test]
        fn parent_with_empty_child_all_fit() {
            let leaf = make_leaf(0, vec![]);
            let parent=make_container(vec![leaf]);
            assert_eq!(last_fitting_char(&parent, 100.0), FitResult::AllFit);
        }
    }

    // =========================================================================
    // mod edge_cases — unusual but valid inputs
    // =========================================================================
    mod edge_cases {
        use super::*;
        #[test]
        fn deeply_nested_result_local_to_root() {
            // grandchild: char_start=5, bottoms [10.0, 70.0].
            // child wraps grandchild: char_start=5.
            // root wraps child: char_start=5.
            // page=50.0: only grandchild char 0 fits.
            // Local to root: (5 + 0) - 5 = 0. Expect LastFitting(0).
            let grandchild = make_leaf(5, vec![10.0, 70.0]);
            let child = make_container(vec![grandchild]);
            let root = make_container(vec![child]);
            assert_eq!(last_fitting_char(&root, 50.0), FitResult::LastFitting(0));
        }

        #[test]
        fn deeply_nested_nonzero_parent_char_start_local_index() {
            // grandchild: char_start=20, bottoms [10.0, 30.0, 60.0].
            // root: char_start=20.
            // page=40.0: chars 0 and 1 of grandchild fit.
            // Local to root: (20 + 1) - 20 = 1. Expect LastFitting(1).
            let grandchild = make_leaf(20, vec![10.0, 30.0, 60.0]);
            let child = make_container(vec![grandchild]);
            let root = make_container(vec![child]);
            assert_eq!(last_fitting_char(&root, 40.0), FitResult::LastFitting(1));
        }

        #[test]
        fn multiple_children_different_char_starts_local_to_parent() {
            // child1: char_start=10, 3 chars, all fit.
            // child2: char_start=13, 3 chars, partial: char 0 fits, chars 1-2 do not.
            // Last fitting global: 13. Local to parent (char_start=10): 13 - 10 = 3.
            // Expect LastFitting(3).
            let child1 = make_leaf(10, vec![10.0, 20.0, 30.0]);
            let child2 = make_leaf(13, vec![40.0, 80.0, 90.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn requesting_fit_on_child_directly_returns_local_to_that_child() {
            // Same setup as above, but we call last_fitting_char on child2 directly.
            // child2: char_start=13. char 0 fits (bottom=40.0 ≤ 50.0), char 1 does not.
            // Local to child2: 0. Expect LastFitting(0).
            let child2 = make_leaf(13, vec![40.0, 80.0, 90.0]);
            assert_eq!(last_fitting_char(&child2, 50.0), FitResult::LastFitting(0));
        }

        #[test]
        fn empty_container_no_children_all_fit() {
            // A container with no children and no text.
            let container = make_container(vec![]);
            assert_eq!(last_fitting_char(&container, 100.0), FitResult::AllFit);
        }

        #[test]
        fn three_children_middle_partially_fits() {
            // child1: char_start=0, 2 chars, both fit.
            // child2: char_start=2, 2 chars, first fits, second does not.
            // child3: char_start=4, 2 chars (never reached after child2 partial).
            // Last fitting global: 2. Local to parent (char_start=0): 2. Expect LastFitting(2).
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 80.0]);
            let child3 = make_leaf(4, vec![90.0, 100.0]);
            let parent = make_container(vec![child1, child2, child3]);
            assert_eq!(last_fitting_char(&parent, 50.0), FitResult::LastFitting(2));
        }
    }
}


#[cfg(test)]
mod last_fitting_char_layout_vec_tests{

}

 
#[cfg(test)]
mod first_fitting_char_tests_no_children {
    use crate::renderer::find_page_boundary::*;
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;

    fn create_mock(text_len: u32, char_tops: Vec<f64>, children: Vec<LayoutQuery>) -> LayoutQuery {
        let char_tops = char_tops.to_vec(); // owned, no lifetime
        let mut layout=LayoutQuery::default();
        layout.text="x".repeat(text_len as usize);
        layout.top=char_tops.last().cloned().unwrap_or(0.0);
        layout.children=children;
        layout.get_char_bottom=Arc::new(|_| panic!("container must not call get_char_bottom"));
        layout.get_char_top=Arc::new(move |offset: u32| char_tops[offset as usize]);
        return layout;
    }

    mod core{
        use super::*;
        #[test]
        fn returns_allfit_when_all_text_fits() {
            let layout = create_mock(
                5,
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom at 100 fits all characters
            let result = first_fitting_char(&layout, 0.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn handles_empty_text_and_children() {
            let layout = create_mock(0, vec![], vec![]);
            let result = first_fitting_char(&layout, 0.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn returns_nonefit_when_no_char_fits() {
            let layout = create_mock(
                5,
                vec![100.0, 110.0, 120.0, 130.0, 140.0],
                vec![]
            );

            // viewport_bottom at 50 fits no characters
            let result = first_fitting_char(&layout, 500.0);
            assert_eq!(result, FitResult::NoneFit);
        }


        #[test]
        fn finds_first_fitting_char() {
            let layout = create_mock(
                10,
                vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0],
                vec![]
            );

            // viewport_bottom at 55 should fit chars 0-4 (bottoms 10-50)
            let result = first_fitting_char(&layout, 55.0);
            assert_eq!(result, FitResult::LastFitting(5));
        }

        #[test]
        fn finds_exact_boundary() {
            let layout = create_mock(
                5,
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom exactly at last char's bottom
            let result = first_fitting_char(&layout, 10.0);
            assert_eq!(result, FitResult::AllFit);
        }

        #[test]
        fn handles_single_char() {
            let layout = create_mock(1, vec![50.0], vec![]);
            assert_eq!(first_fitting_char(&layout, 40.0), FitResult::AllFit); // fits all
            assert_eq!(first_fitting_char(&layout, 60.0), FitResult::NoneFit); // fits none
            assert_eq!(first_fitting_char(&layout, 50.0), FitResult::AllFit); // exact
        }


        #[test]
        fn local_index_independent_of_char_start() {
            let mut layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            layout.char_start = 100; // global offset
            let result = first_fitting_char(&layout, 35.0);
            assert_eq!(result, FitResult::LastFitting(3)); // still 2, not 102
        }
    }



    
    mod edge_cases{
        use super::*;
        #[test]
        fn handles_gaps_in_char_bottoms() {
            let layout = create_mock(
                5,
                vec![10.0, 10.0, 11.0, 12.0, 130.0],
                vec![]
            );

            // Only first char fits
            let result = first_fitting_char(&layout, 50.0);
            assert_eq!(result, FitResult::LastFitting(4));
        }

        #[test]
        fn handles_consecutive_fitting_chars() {
            let layout = create_mock(
                11,
                vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 15.0, 16.0, 17.0, 18.0, 19.0],
                vec![]
            );

            // viewport_bottom at 15 fits chars 0-5
            let result = first_fitting_char(&layout, 15.0);
            assert_eq!(result, FitResult::LastFitting(5));
        }

        #[test]
        fn handles_large_text() {
            let char_bottoms: Vec<f64> = (0..100).map(|i| (i + 1) as f64 * 10.0).collect();
            let layout = create_mock(100, char_bottoms, vec![]);

            // viewport_bottom at 500 should fit chars 50-100
            let result = first_fitting_char(&layout, 500.0);
            assert_eq!(result, FitResult::LastFitting(49));
        }

        #[test]
        fn handles_zero_viewport() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            let result = first_fitting_char(&layout, 100.0);
            assert_eq!(result, FitResult::NoneFit);
        }

        #[test]
        fn handles_negative_viewport() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);
            let result = first_fitting_char(&layout, -10.0);
            assert_eq!(result, FitResult::AllFit);
        }


        #[test]
        fn single_oversized_char_returns_none_fit() {
            let layout = create_mock(1, vec![-10.0], vec![]);
            assert_eq!(first_fitting_char(&layout, 800.0), FitResult::NoneFit);
        }

    }


    mod boundaries{
        use super::*;
         #[test]
        fn does_not_return_char_with_top_exceeding_viewport() {
            let layout = create_mock(
                5,
                vec![10.0, 20.0, 30.0, 40.0, 50.0],
                vec![]
            );

            // viewport_bottom at 49.999 should not fit char at 50
            let result = first_fitting_char(&layout, 15.0);
            assert_eq!(result, FitResult::LastFitting(1));
        }

        #[test]
        fn char_just_under_boundary_fits() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

            // one pixel under
            let result = first_fitting_char(&layout, 9.99);
            assert_eq!(result, FitResult::AllFit); // char at 50.0 doesn't fit, cutoff at index 4
        }

        #[test]
        fn char_just_over_boundary_does_not_fit() {
            let layout = create_mock(5, vec![10.0, 20.0, 30.0, 40.0, 50.0], vec![]);

            // one pixel over
            let result = first_fitting_char(&layout, 10.0001);
            assert_eq!(result, FitResult::LastFitting(1)); // char at 50.0 doesn't fit, cutoff at index 4
        }
    }

}

