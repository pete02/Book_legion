 
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
mod last_fitting_char_vec_tests {
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;
    use crate::renderer::find_page_boundary::*;
 
    // =========================================================================
    // Test helpers
    // =========================================================================
 
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
        layout.get_char_top = Arc::new(|_| 0.0);
        layout
    }
 
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
 
    // =========================================================================
    // mod core
    // =========================================================================
    mod core {
        use super::*;
 
        #[test]
        fn single_layout_all_fit() {
            let layouts = vec![make_leaf(0, vec![10.0, 20.0, 30.0])];
            assert_eq!(last_fitting_char_vec(&layouts, 100.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn single_layout_none_fit() {
            let layouts = vec![make_leaf(0, vec![100.0, 200.0])];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 0));
        }
 
        #[test]
        fn single_layout_partial_fit() {
            // Chars 0-1 fit, char 2 does not. LastFitting(1) local to layout[0].
            let layouts = vec![make_leaf(0, vec![10.0, 20.0, 60.0])];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 0));
        }
 
        #[test]
        fn multiple_layouts_all_fit() {
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
                make_leaf(4, vec![50.0, 60.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 100.0), (FitResult::AllFit, 2));
        }
 
        #[test]
        fn multiple_layouts_first_none_fit() {
            // First layout already doesn't fit. NoneFit is legal at idx=0.
            let layouts = vec![
                make_leaf(0, vec![100.0, 200.0]),
                make_leaf(2, vec![300.0, 400.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 0));
        }
 
        #[test]
        fn multiple_layouts_last_partially_fits() {
            // layouts[0] and [1] fully fit, layouts[2] partially fits.
            // LastFitting(1) local to layouts[2].
            let layouts = vec![
                make_leaf(0,  vec![10.0, 20.0]),
                make_leaf(2,  vec![30.0, 40.0]),
                make_leaf(4,  vec![50.0, 60.0, 90.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 70.0), (FitResult::LastFitting(1), 2));
        }
 
        #[test]
        fn first_fits_second_none_fit_returns_all_fit_not_none_fit() {
            // layouts[0] fully fits, layouts[1] none fit.
            // NoneFit at idx=1 is illegal → must return (AllFit, 0).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![200.0, 300.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn result_local_to_the_layout_at_returned_index() {
            // layouts[1] has char_start=10. Partial fit: char 0 fits, char 1 does not.
            // LastFitting(0) — local to layouts[1], not global index 10.
            let layouts = vec![
                make_leaf(0,  vec![10.0, 20.0]),
                make_leaf(10, vec![30.0, 80.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(0), 1));
        }
 
        #[test]
        fn container_layouts_respected() {
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 40.0]);
            let layouts = vec![
                make_container(vec![child1, child2]),
                make_leaf(4, vec![50.0, 90.0]),
            ];
            // layouts[0] fully fits, layouts[1] partial: char 0 fits, char 1 does not.
            assert_eq!(last_fitting_char_vec(&layouts, 70.0), (FitResult::LastFitting(0), 1));
        }
    }
 
    // =========================================================================
    // mod boundary
    // =========================================================================
    mod boundary {
        use super::*;
 
        #[test]
        fn layout_bottom_exactly_on_page_is_all_fit() {
            // Last char bottom == page_height → AllFit for that layout.
            let layouts = vec![make_leaf(0, vec![10.0, 20.0, 50.0])];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn last_char_of_layout_exactly_on_boundary() {
            // layouts[0] last char bottom == page_height exactly → AllFit at idx 0.
            // layouts[1] first char already exceeds → would be NoneFit at idx 1,
            // which is illegal → result is (AllFit, 0).
            let layouts = vec![
                make_leaf(0, vec![10.0, 50.0]),
                make_leaf(2, vec![60.0, 70.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn first_char_of_second_layout_exactly_on_boundary_fits() {
            // layouts[1] first char bottom == page_height → that char fits,
            // but no further chars exist → AllFit at idx 1.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![50.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }
 
        #[test]
        fn partial_fit_one_epsilon_under_boundary() {
            // layouts[1]: char 0 bottom=49.999 fits, char 1 bottom=50.001 does not.
            // LastFitting(0) at idx 1.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![49.999, 50.001]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(0), 1));
        }
 
        #[test]
        fn none_fit_only_legal_at_idx_zero() {
            // layouts[0] fits, layouts[1] none fit → (AllFit, 0), not (NoneFit, 1).
            let layouts = vec![
                make_leaf(0, vec![10.0]),
                make_leaf(1, vec![200.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn zero_page_height_none_fit_at_zero() {
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 0.0), (FitResult::NoneFit, 0));
        }
 
        #[test]
        fn negative_page_height_none_fit_at_zero() {
            let layouts = vec![make_leaf(0, vec![10.0, 20.0])];
            assert_eq!(last_fitting_char_vec(&layouts, -1.0), (FitResult::NoneFit, 0));
        }
    }
 
    // =========================================================================
    // mod edge_cases
    // =========================================================================
    mod edge_cases {
        use super::*;
 
        #[test]
        fn empty_vec_all_fit_at_zero() {
            // Vacuously everything in an empty vec fits.
            let layouts: Vec<LayoutQuery> = vec![];
            assert_eq!(last_fitting_char_vec(&layouts, 100.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn single_oversized_layout_none_fit() {
            let layouts = vec![make_leaf(0, vec![2000.0])];
            assert_eq!(last_fitting_char_vec(&layouts, 800.0), (FitResult::NoneFit, 0));
        }
 
        #[test]
        fn many_layouts_all_fit_returns_last_index() {
            let layouts: Vec<LayoutQuery> = (0..10)
                .map(|i| make_leaf(i * 3, vec![(i * 3 + 1) as f64 * 10.0, (i * 3 + 2) as f64 * 10.0, (i * 3 + 3) as f64 * 10.0]))
                .collect();
            assert_eq!(last_fitting_char_vec(&layouts, 10000.0), (FitResult::AllFit, 9));
        }
 
        #[test]
        fn partial_fit_in_middle_layout_correct_index() {
            // layouts[0] and [1] all fit, layouts[2] partial, layouts[3] not reached.
            let layouts = vec![
                make_leaf(0,  vec![10.0, 20.0]),
                make_leaf(2,  vec![30.0, 40.0]),
                make_leaf(4,  vec![50.0, 90.0]),
                make_leaf(6,  vec![100.0, 110.0]),
            ];
            // layouts[2]: char 0 fits (bottom=50.0), char 1 does not. LastFitting(0) at idx 2.
            assert_eq!(last_fitting_char_vec(&layouts, 70.0), (FitResult::LastFitting(0), 2));
        }
 
        #[test]
        fn large_char_start_result_still_local() {
            // layouts[1] has char_start=1000. Partial: char 0 fits, char 1 does not.
            // Result must be LastFitting(0) — local to layouts[1] — not LastFitting(1000).
            let layouts = vec![
                make_leaf(0,    vec![10.0, 20.0]),
                make_leaf(1000, vec![30.0, 80.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(0), 1));
        }
 
        #[test]
        fn mixed_container_and_leaf_layouts() {
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 40.0]);
            let container = make_container(vec![child1, child2]);
            let leaf = make_leaf(4, vec![200.0, 300.0]);
            // container fully fits, leaf none fits → (AllFit, 0), not (NoneFit, 1).
            let layouts = vec![container, leaf];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }
 
        #[test]
        fn all_layouts_none_fit_returns_none_fit_at_zero() {
            let layouts = vec![
                make_leaf(0, vec![200.0, 300.0]),
                make_leaf(2, vec![400.0, 500.0]),
            ];
            assert_eq!(last_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 0));
        }
    }
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


#[cfg(test)]
mod first_fitting_char_tests_with_children {
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;
    use crate::renderer::find_page_boundary::*;

    // =========================================================================
    // Test helpers
    // =========================================================================

    /// Leaf node: has text, no children.
    /// `char_start` is the global offset of this node's first character.
    /// `char_tops` is indexed locally (0 = first char of this node).
    fn make_leaf(char_start: u32, char_tops: Vec<f64>) -> LayoutQuery {
        let text_len = char_tops.len();
        let tops = char_tops.clone();
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = "x".repeat(text_len);
        layout.top = char_tops.first().cloned().unwrap_or(0.0);
        layout.bottom = char_tops.last().cloned().unwrap_or(0.0);
        layout.children = vec![];
        layout.get_char_bottom = Arc::new(|_| 0.0);
        layout.get_char_top = Arc::new(move |i| tops[i as usize]);
        layout
    }

    /// Container node: has children, no text.
    /// `char_start` mirrors the first child's char_start (typical case).
    fn make_container(children: Vec<LayoutQuery>) -> LayoutQuery {
        let top = children.iter().map(|c| c.top).fold(f64::MAX, f64::min);
        let bottom = children.iter().map(|c| c.bottom).fold(0.0_f64, f64::max);
        let char_start = children.first().map(|c| c.char_start).unwrap_or(0);
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = String::new();
        layout.top = top;
        layout.bottom = bottom;
        layout.children = children;
        layout.get_char_bottom = Arc::new(|_| panic!("container must not call get_char_bottom"));
        layout.get_char_top = Arc::new(|_| panic!("container must not call get_char_top"));
        layout
    }

    // =========================================================================
    // mod core — fundamental contract
    // =========================================================================
    mod core {
        use super::*;

        #[test]
        fn container_with_single_child_full_fit() {
            // All char tops >= page_top=0.0; everything fits.
            let child = make_leaf(0, vec![10.0, 20.0, 30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 0.0), FitResult::AllFit);
        }

        #[test]
        fn container_with_single_child_no_fit() {
            // All char tops below page_top=100.0; nothing fits.
            let child = make_leaf(0, vec![10.0, 20.0, 30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 100.0), FitResult::NoneFit);
        }

        #[test]
        fn container_single_child_partial_fit() {
            // child: char_start=10, tops [10.0, 20.0, 60.0].
            // page_top=50.0: chars 0-1 are above the cut, char 2 is the first to fit.
            // Local to parent (char_start=10): (10 + 2) - 10 = 2. Expect LastFitting(2).
            let child = make_leaf(10, vec![10.0, 20.0, 60.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(2));
        }

        #[test]
        fn container_result_is_local_to_parent_not_child() {
            // child1: char_start=100, 3 chars, all tops below page_top.
            // child2: char_start=103, first char top >= page_top, rest also fit.
            // First fitting global char: 103. Local to parent (char_start=100): 103 - 100 = 3.
            // Expect LastFitting(3).
            let child1 = make_leaf(100, vec![10.0, 20.0, 30.0]);
            let child2 = make_leaf(103, vec![60.0, 70.0, 80.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn container_two_children_first_none_second_all_fit() {
            // child1: all tops below page_top. child2: all tops at or above page_top.
            // First fitting = first char of child2.
            // Local to parent (char_start=0): (5 + 0) - 0 = 5. Expect LastFitting(5).
            let child1 = make_leaf(0, vec![10.0, 20.0, 30.0, 40.0, 45.0]);
            let child2 = make_leaf(5, vec![60.0, 70.0, 80.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(5));
        }

        #[test]
        fn empty_layout_is_all_fit() {
            let container = make_container(vec![]);
            assert_eq!(first_fitting_char(&container, 100.0), FitResult::AllFit);
        }

        #[test]
        fn requesting_fit_on_child_directly_returns_local_to_that_child() {
            // Same setup as container_result_is_local_to_parent_not_child,
            // but calling directly on child2. char_start=103, first top=60.0 >= 50.0.
            // Local to child2: 0. Expect AllFit (char 0 already fits → all fit from 0).
            let child2 = make_leaf(103, vec![60.0, 70.0, 80.0]);
            assert_eq!(first_fitting_char(&child2, 50.0), FitResult::AllFit);
        }
    }

    // =========================================================================
    // mod boundary — exact edge values
    // =========================================================================
    mod boundary {
        use super::*;

        #[test]
        fn char_top_exactly_equals_page_top_fits() {
            // top == page_top counts as fitting.
            // child: char_start=0, tops [10.0, 20.0, 30.0]. page_top=20.0.
            // Chars 0 is above cut, char 1 top == page_top → first fitting.
            // Local to parent: 1. Expect LastFitting(1).
            let child = make_leaf(0, vec![10.0, 20.0, 30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 20.0), FitResult::LastFitting(1));
        }

        #[test]
        fn char_top_one_epsilon_below_page_does_not_fit() {
            // char top 19.9999 < 20.0: char 1 does not fit, char 2 does.
            // Expect LastFitting(2).
            let child = make_leaf(0, vec![10.0, 19.9999, 30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 20.0), FitResult::LastFitting(2));
        }

        #[test]
        fn char_top_one_epsilon_above_page_gives_all_fit_for_that_char() {
            // All tops >= page_top → AllFit.
            let child = make_leaf(0, vec![20.0001, 30.0, 40.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 20.0), FitResult::AllFit);
        }

        #[test]
        fn first_child_all_above_cut_second_child_exactly_on_boundary() {
            // child1: all tops < page_top.
            // child2: first top exactly == page_top.
            // First fitting global: child2.char_start + 0 = 3.
            // Local to parent (char_start=0): 3. Expect LastFitting(3).
            let child1 = make_leaf(0, vec![10.0, 20.0, 30.0]);
            let child2 = make_leaf(3, vec![50.0, 60.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn first_child_partially_fits_at_exact_boundary() {
            // child: char_start=20, tops [30.0, 50.0, 70.0]. page_top=50.0.
            // Char 0 top=30.0 < 50.0 (above cut), char 1 top=50.0 == page_top → first fitting.
            // Local to parent (char_start=20): (20 + 1) - 20 = 1. Expect LastFitting(1).
            let child = make_leaf(20, vec![30.0, 50.0, 70.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(1));
        }

        #[test]
        fn single_char_exactly_on_boundary_all_fit() {
            // Single char, top == page_top → AllFit (char 0 is the first and it fits).
            let child = make_leaf(0, vec![50.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::AllFit);
        }

        #[test]
        fn single_char_above_page_top_none_fit() {
            let child = make_leaf(0, vec![30.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::NoneFit);
        }

        #[test]
        fn single_char_below_page_top_all_fit() {
            let child = make_leaf(0, vec![70.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::AllFit);
        }
    }

    // =========================================================================
    // mod edge_cases — unusual but valid inputs
    // =========================================================================
    mod edge_cases {
        use super::*;

        #[test]
        fn deeply_nested_result_local_to_root() {
            // grandchild: char_start=5, tops [10.0, 70.0].
            // page_top=50.0: char 0 top=10.0 < 50.0 (above cut), char 1 top=70.0 >= 50.0.
            // First fitting global: 5+1=6. Local to root (char_start=5): 6 - 5 = 1.
            // Expect LastFitting(1).
            let grandchild = make_leaf(5, vec![10.0, 70.0]);
            let child = make_container(vec![grandchild]);
            let root = make_container(vec![child]);
            assert_eq!(first_fitting_char(&root, 50.0), FitResult::LastFitting(1));
        }

        #[test]
        fn deeply_nested_nonzero_parent_char_start_local_index() {
            // grandchild: char_start=20, tops [10.0, 30.0, 60.0].
            // page_top=40.0: chars 0-1 above cut, char 2 top=60.0 is first fitting.
            // Local to root (char_start=20): (20 + 2) - 20 = 2. Expect LastFitting(2).
            let grandchild = make_leaf(20, vec![10.0, 30.0, 60.0]);
            let child = make_container(vec![grandchild]);
            let root = make_container(vec![child]);
            assert_eq!(first_fitting_char(&root, 40.0), FitResult::LastFitting(2));
        }

        #[test]
        fn multiple_children_different_char_starts_local_to_parent() {
            // child1: char_start=10, all tops below page_top.
            // child2: char_start=13, first top >= page_top.
            // First fitting global: 13. Local to parent (char_start=10): 13 - 10 = 3.
            // Expect LastFitting(3).
            let child1 = make_leaf(10, vec![10.0, 20.0, 30.0]);
            let child2 = make_leaf(13, vec![60.0, 70.0, 80.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn three_children_first_none_second_partial_third_not_reached() {
            // child1: char_start=0, all tops below page_top.
            // child2: char_start=2, first top below, second top >= page_top.
            // child3: char_start=4, not reached (first fitting already found in child2).
            // First fitting global: 2+1=3. Local to parent (char_start=0): 3.
            // Expect LastFitting(3).
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 60.0]);
            let child3 = make_leaf(4, vec![70.0, 80.0]);
            let parent = make_container(vec![child1, child2, child3]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(3));
        }

        #[test]
        fn gap_in_char_tops_first_fitting_skips_ahead() {
            // Char tops jump sharply; the first char >= page_top is not char 1.
            // child: char_start=0, tops [10.0, 100.0, 110.0]. page_top=50.0.
            // Char 0 top=10.0 < 50.0, char 1 top=100.0 >= 50.0 → first fitting = 1.
            // Expect LastFitting(1).
            let child = make_leaf(0, vec![10.0, 100.0, 110.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(1));
        }

        #[test]
        fn all_children_none_fit() {
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 40.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 100.0), FitResult::NoneFit);
        }

        #[test]
        fn all_children_all_fit() {
            let child1 = make_leaf(0, vec![60.0, 70.0]);
            let child2 = make_leaf(2, vec![80.0, 90.0]);
            let parent = make_container(vec![child1, child2]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::AllFit);
        }

        #[test]
        fn consecutive_chars_same_top_first_of_group_is_first_fitting() {
            // Multiple chars share the same top value that crosses the boundary.
            // child: char_start=0, tops [10.0, 10.0, 60.0, 60.0]. page_top=50.0.
            // Chars 0-1 top=10.0 < 50.0, char 2 top=60.0 is first fitting.
            // Expect LastFitting(2).
            let child = make_leaf(0, vec![10.0, 10.0, 60.0, 60.0]);
            let parent = make_container(vec![child]);
            assert_eq!(first_fitting_char(&parent, 50.0), FitResult::LastFitting(2));
        }
    }
}

#[cfg(test)]
mod first_fitting_char_vec_tests {
    use std::sync::Arc;
    use crate::renderer::layout_builder::LayoutQuery;
    use crate::renderer::find_page_boundary::*;

    // =========================================================================
    // Test helpers
    // =========================================================================

    fn make_leaf(char_start: u32, char_tops: Vec<f64>) -> LayoutQuery {
        let text_len = char_tops.len();
        let tops = char_tops.clone();
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = "x".repeat(text_len);
        layout.top = char_tops.first().cloned().unwrap_or(0.0);
        layout.bottom = char_tops.last().cloned().unwrap_or(0.0);
        layout.children = vec![];
        layout.get_char_bottom = Arc::new(|_| 0.0);
        layout.get_char_top = Arc::new(move |i| tops[i as usize]);
        layout
    }

    fn make_container(children: Vec<LayoutQuery>) -> LayoutQuery {
        let top = children.iter().map(|c| c.top).fold(f64::MAX, f64::min);
        let bottom = children.iter().map(|c| c.bottom).fold(0.0_f64, f64::max);
        let char_start = children.first().map(|c| c.char_start).unwrap_or(0);
        let mut layout = LayoutQuery::default();
        layout.char_start = char_start;
        layout.text = String::new();
        layout.top = top;
        layout.bottom = bottom;
        layout.children = children;
        layout.get_char_bottom = Arc::new(|_| panic!("container must not call get_char_bottom"));
        layout.get_char_top = Arc::new(|_| panic!("container must not call get_char_top"));
        layout
    }

    // =========================================================================
    // mod core
    // =========================================================================
    mod core {
        use super::*;

        #[test]
        fn single_layout_all_fit() {
            // Only layout, all tops >= page_top → (AllFit, 0).
            let layouts = vec![make_leaf(0, vec![60.0, 70.0, 80.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn single_layout_none_fit() {
            // Only layout, all tops < page_top → (NoneFit, 0) which equals len-1=0, legal.
            let layouts = vec![make_leaf(0, vec![10.0, 20.0, 30.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 0));
        }

        #[test]
        fn single_layout_partial_fit() {
            // Char 0 top < page_top, char 1 top >= page_top.
            // FirstFitting(1) local to layout[0]. Returns (LastFitting(1), 0).
            let layouts = vec![make_leaf(0, vec![10.0, 60.0, 70.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 0));
        }

        #[test]
        fn multiple_layouts_all_fit() {
            // Every layout fully fits → (AllFit, 0).
            let layouts = vec![
                make_leaf(0, vec![60.0, 70.0]),
                make_leaf(2, vec![80.0, 90.0]),
                make_leaf(4, vec![100.0, 110.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn multiple_layouts_last_none_fit() {
            // Last layout (checked first) already none fit → (NoneFit, len-1=2).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![60.0, 70.0]),
                make_leaf(4, vec![80.0, 90.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }

        #[test]
        fn last_fits_second_to_last_none_fit_returns_all_fit_not_none_fit() {
            // layouts[2] all fit, layouts[1] none fit.
            // NoneFit at idx=1 is illegal (1 < len-1=2) → must return (AllFit, 2).
            let layouts = vec![
                make_leaf(0, vec![60.0, 70.0]),
                make_leaf(2, vec![10.0, 20.0]),
                make_leaf(4, vec![80.0, 90.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 2));
        }

        #[test]
        fn multiple_layouts_first_fitting_in_middle() {
            // layouts[0] none fit, layouts[1] none fit, layouts[2] all fit, layouts[3] all fit.
            // Iterating from end: layouts[3] all fit, layouts[2] all fit, layouts[1] none fit.
            // NoneFit at idx=1 is illegal → (AllFit, 2).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
                make_leaf(4, vec![60.0, 70.0]),
                make_leaf(6, vec![80.0, 90.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 2));
        }

        #[test]
        fn partial_fit_in_middle_layout() {
            // layouts[0] none fit, layouts[1] partial, layouts[2] all fit, layouts[3] all fit.
            // Iterating from end: [3] all fit, [2] all fit, [1] partial → (LastFitting(1), 1).
            // k=1 local to layouts[1]: char 0 top < page_top, char 1 top >= page_top.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 60.0]),
                make_leaf(4, vec![70.0, 80.0]),
                make_leaf(6, vec![90.0, 100.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 1));
        }

        #[test]
        fn result_local_to_layout_at_returned_index() {
            // layouts[1] has char_start=10. Partial: char 0 top < page_top, char 1 top >= page_top.
            // LastFitting(1) — local to layouts[1], not global index 11.
            let layouts = vec![
                make_leaf(0,  vec![10.0, 20.0]),
                make_leaf(10, vec![30.0, 60.0]),
                make_leaf(12, vec![70.0, 80.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 1));
        }

        #[test]
        fn container_layouts_respected() {
            let child1 = make_leaf(0, vec![10.0, 20.0]);
            let child2 = make_leaf(2, vec![30.0, 40.0]);
            let container = make_container(vec![child1, child2]);
            // container: all tops < page_top → none fit.
            // leaf: all tops >= page_top → all fit.
            // NoneFit at container (idx=0) is illegal (0 < len-1=1) → (AllFit, 1).
            let layouts = vec![
                container,
                make_leaf(4, vec![60.0, 70.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }
    }

    // =========================================================================
    // mod boundary
    // =========================================================================
    mod boundary {
        use super::*;

        #[test]
        fn char_top_exactly_on_page_top_fits() {
            // Last layout: char 0 top == page_top → fits → AllFit at last idx.
            // Previous layout none fit → NoneFit at idx=0 illegal → (AllFit, 1).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![50.0, 60.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }

        #[test]
        fn char_top_one_epsilon_below_page_top_does_not_fit() {
            // Last layout: char 0 top=49.999 < 50.0 → doesn't fit → NoneFit at len-1.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![49.999]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 1));
        }

        #[test]
        fn char_top_one_epsilon_above_page_top_fits() {
            // Last layout: char 0 top=50.001 > 50.0 → fits.
            // Previous layout none fit → (AllFit, 1).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![50.001, 60.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }

        #[test]
        fn last_layout_partial_exactly_on_boundary() {
            // Last layout: char 0 top < page_top, char 1 top == page_top → partial.
            // LastFitting(1) local to last layout.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 50.0, 70.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 1));
        }

        #[test]
        fn none_fit_only_legal_at_last_index() {
            // layouts[1] (last) all fit, layouts[0] none fit.
            // NoneFit at idx=0 < len-1=1 is illegal → (AllFit, 1).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![60.0, 70.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }

        #[test]
        fn single_char_top_exactly_on_boundary_all_fit() {
            let layouts = vec![make_leaf(0, vec![50.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn single_char_top_below_boundary_none_fit() {
            let layouts = vec![make_leaf(0, vec![30.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 0));
        }

        #[test]
        fn zero_page_top_all_layouts_fit() {
            // page_top=0.0: all tops >= 0.0 → everything fits → (AllFit, 0).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 0.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn negative_page_top_all_layouts_fit() {
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, -10.0), (FitResult::AllFit, 0));
        }
    }

    // =========================================================================
    // mod edge_cases
    // =========================================================================
    mod edge_cases {
        use super::*;

        #[test]
        fn empty_vec_all_fit_at_zero() {
            let layouts: Vec<LayoutQuery> = vec![];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn single_oversized_layout_all_fit() {
            // A layout far below page_top still fits from the top perspective.
            let layouts = vec![make_leaf(0, vec![2000.0, 2010.0])];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn all_layouts_none_fit_returns_none_fit_at_last_index() {
            // All tops < page_top. Only legal NoneFit is at len-1.
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
                make_leaf(4, vec![45.0, 49.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::NoneFit, 2));
        }

        #[test]
        fn large_char_start_result_still_local() {
            // layouts[1] has char_start=1000. Partial: char 0 top < page_top, char 1 top >= page_top.
            // Result must be LastFitting(1) — local to layouts[1] — not LastFitting(1001).
            let layouts = vec![
                make_leaf(0,    vec![10.0, 20.0]),
                make_leaf(1000, vec![30.0, 60.0]),
                make_leaf(1002, vec![70.0, 80.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 1));
        }

        #[test]
        fn many_layouts_all_fit_returns_index_zero() {
            let layouts: Vec<LayoutQuery> = (0..10)
                .map(|i| make_leaf(i * 2, vec![(i * 2 + 1) as f64 * 10.0 + 100.0, (i * 2 + 2) as f64 * 10.0 + 100.0]))
                .collect();
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 0));
        }

        #[test]
        fn partial_fit_at_last_layout() {
            // Last layout partial: char 0 top < page_top, char 1 top >= page_top.
            // No need to check further back. Returns (LastFitting(1), last_idx).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 40.0]),
                make_leaf(4, vec![45.0, 60.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 2));
        }

        #[test]
        fn mixed_container_and_leaf_layouts() {
            let child1 = make_leaf(0, vec![60.0, 70.0]);
            let child2 = make_leaf(2, vec![80.0, 90.0]);
            let container = make_container(vec![child1, child2]);
            // leaf: none fit. container: all fit.
            // NoneFit at leaf (idx=0) is illegal (0 < len-1=1) → (AllFit, 1).
            let layouts = vec![
                make_leaf(4, vec![10.0, 20.0]),
                container,
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::AllFit, 1));
        }

        #[test]
        fn consecutive_chars_same_top_crossing_boundary() {
            // Last layout: chars 0-1 share top=30.0 < page_top, chars 2-3 share top=60.0 >= page_top.
            // First fitting local index = 2. Returns (LastFitting(2), last_idx).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),
                make_leaf(2, vec![30.0, 30.0, 60.0, 60.0]),
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(2), 1));
        }

        #[test]
        fn three_layouts_middle_partial_earlier_irrelevant() {
            // Iterating from end: layouts[2] all fit, layouts[1] partial → stop.
            // layouts[0] is never checked.
            // Returns (LastFitting(1), 1).
            let layouts = vec![
                make_leaf(0, vec![10.0, 20.0]),   // never checked
                make_leaf(2, vec![30.0, 60.0]),   // partial
                make_leaf(4, vec![70.0, 80.0]),   // all fit
            ];
            assert_eq!(first_fitting_char_vec(&layouts, 50.0), (FitResult::LastFitting(1), 1));
        }
    }
}