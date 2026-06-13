use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{window, HtmlElement};
use crate::renderer::*;

wasm_bindgen_test_configure!(run_in_browser);

// =============================================================================
// Helpers
// =============================================================================

/// Creates a viewport div of fixed pixel height, appended to document.body.
/// Uses monospace + fixed font metrics to maximise cross-platform determinism.
/// Returns the element; caller is responsible for removing it after the test.
fn make_viewport(height_px: u32) -> HtmlElement {
    let document = window().unwrap().document().unwrap();
    let el = document.create_element("div").unwrap();
    el.set_attribute("style", &format!(
        "position:absolute;\
         top:0;left:0;\
         width:400px;\
         height:{height_px}px;\
         overflow:hidden;\
         font-family:monospace;\
         font-size:16px;\
         line-height:20px;\
         visibility:hidden;"
    )).unwrap();
    document.body().unwrap().append_child(&el).unwrap();
    el.dyn_into().unwrap()
}

/// Removes the viewport from the DOM after a test.
fn cleanup(el: &HtmlElement) {
    el.remove();
}



mod cut_forward{
    use super::*;
    /// Asserts that every descendant element of `root` has its bottom <=
    /// root.bottom and its top >= root.top, using getBoundingClientRect.
    /// Panics with the offending element's outerHTML on failure.
    fn assert_no_overflow(viewport: &HtmlElement) {
        let vr = viewport.get_bounding_client_rect();
        let v_top = vr.top();
        let v_bottom = vr.bottom();
        assert_all_descendants_fit(viewport, v_top, v_bottom);
    }

    fn assert_all_descendants_fit(el: &HtmlElement, v_top: f64, v_bottom: f64) {
        let children = el.children();
        for i in 0..children.length() {
            let child = children
                .item(i).unwrap()
                .dyn_into::<HtmlElement>().unwrap();
            let r = child.get_bounding_client_rect();
            assert!(
                r.top() >= v_top - 0.5,
                "Element top {:.1} is above viewport top {:.1}: {}",
                r.top(), v_top, child.outer_html()
            );
            assert!(
                r.bottom() <= v_bottom + 0.5,
                "Element bottom {:.1} exceeds viewport bottom {:.1}: {}",
                r.bottom(), v_bottom, child.outer_html()
            );
            assert_all_descendants_fit(&child, v_top, v_bottom);
        }
    }

    // =============================================================================
    // mod core — fundamental visual invariant
    // =============================================================================
    mod core {
        use super::*;

        #[wasm_bindgen_test]
        fn short_text_fits_without_cut_forward() {
            // Text so short it fits entirely; cut should be a no-op or cut at the end.
            let viewport = make_viewport(400);
            let html = r#"<p style="margin:0">Hello world</p>"#;
            cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn long_text_is_cut_to_fit() {
            // Enough paragraphs that they cannot all fit in 200px at 20px line-height.
            let viewport = make_viewport(200);
            let html = (0..30)
                .map(|i| format!(r#"<p style="margin:0;line-height:20px">Line {i}</p>"#))
                .collect::<Vec<_>>()
                .join("");
            cut_forward(&viewport, &html, 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn nested_elements_do_not_overflow() {
            // Deeply nested structure; every descendant must stay within viewport.
            let viewport = make_viewport(100);
            let html = r#"
                <div style="margin:0">
                    <div style="margin:0">
                        <p style="margin:0;line-height:20px">Line 1</p>
                        <p style="margin:0;line-height:20px">Line 2</p>
                        <p style="margin:0;line-height:20px">Line 3</p>
                        <p style="margin:0;line-height:20px">Line 4</p>
                        <p style="margin:0;line-height:20px">Line 5</p>
                        <p style="margin:0;line-height:20px">Line 6</p>
                        <p style="margin:0;line-height:20px">Line 7</p>
                        <p style="margin:0;line-height:20px">Line 8</p>
                    </div>
                </div>"#;
            cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn single_oversized_element_does_not_crash() {
            // One element taller than the viewport; acceptable-cut fallback kicks in.
            // We only assert no crash and valid DOM — overflow is permitted in this case.
            let viewport = make_viewport(100);
            let html = r#"<p style="margin:0;line-height:20px;padding:200px 0">Tall paragraph</p>"#;
            cut_forward(&viewport, html, 0); // must not panic
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn empty_html_does_not_crash() {
            let viewport = make_viewport(200);
            cut_forward(&viewport, "", 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn viewport_content_is_valid_html_after_cut_forward() {
            // After cut, innerHTML must be parseable — no unclosed tags.
            // We verify this by re-setting innerHTML to itself, which would throw on bad HTML.
            let viewport = make_viewport(100);
            let html = r#"
                <p style="margin:0;line-height:20px"><b>Bold <i>italic text</i> end</b></p>
                <p style="margin:0;line-height:20px">Second paragraph</p>
                <p style="margin:0;line-height:20px">Third paragraph</p>
                <p style="margin:0;line-height:20px">Fourth paragraph</p>
                <p style="margin:0;line-height:20px">Fifth paragraph</p>"#;
            cut_forward(&viewport, html, 0);
            let inner = viewport.inner_html();
            // Re-parse by setting on a detached div — throws if malformed.
            let document = window().unwrap().document().unwrap();
            let probe = document.create_element("div").unwrap();
            probe.set_inner_html(&inner); // must not throw
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    }

    // =============================================================================
    // mod boundary — edge cases around the cut point
    // =============================================================================
    mod boundary {
        use super::*;

        #[wasm_bindgen_test]
        fn cut_exactly_at_paragraph_boundary() {
            // Two paragraphs that together exactly fill the viewport.
            // Each paragraph is exactly 50px tall (line-height:50px, margin:0).
            // viewport=100px. Both should fit; no cut needed inside either paragraph.
            let viewport = make_viewport(100);
            let html = r#"
                <p style="margin:0;line-height:50px">First</p>
                <p style="margin:0;line-height:50px">Second</p>
                <p style="margin:0;line-height:50px">Third</p>"#;
            cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn single_char_over_boundary_is_cut_forward() {
            // Fill viewport almost completely, then one more character that just overflows.
            let viewport = make_viewport(100);
            // 5 lines at 20px = 100px exactly, then a 6th.
            let mut html = (0..5)
                .map(|i| format!(r#"<p style="margin:0;line-height:20px">Line {i}</p>"#))
                .collect::<Vec<_>>()
                .join("");
            html.push_str(r#"<p style="margin:0;line-height:20px">Overflow</p>"#);
            cut_forward(&viewport, &html, 0);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn zero_height_viewport_does_not_crash() {
            let viewport = make_viewport(0);
            let html = r#"<p style="margin:0;line-height:20px">Some text</p>"#;
            cut_forward(&viewport, html, 0);
            cleanup(&viewport);
        }
    }

    // =============================================================================
    // mod unambiguous — content and index assertions where cut point cannot vary
    // =============================================================================
    mod unambiguous {
        use super::*;

        /// These tests use a large explicit gap between blocks so the cut point is
        /// forced to fall at the paragraph boundary regardless of font rendering.
        /// We assert both the visual invariant AND what text is present/absent.

        #[wasm_bindgen_test]
        fn content_before_cut_is_preserved() {
            // Block A: 40px tall. Gap of 500px. Block B: 40px tall.
            // Viewport: 100px. Block B cannot possibly fit → cut after block A.
            // After cut, viewport textContent must contain "Block A" and not "Block B".
            let viewport = make_viewport(100);
            let html = r#"
                <p style="margin:0;line-height:40px">Block A</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Block B</p>"#;
            let cut_index = cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            let content = viewport.text_content().unwrap_or_default();
            assert!(content.contains("Block A"), "Expected 'Block A' in viewport after cut");
            assert!(!content.contains("Block B"), "Expected 'Block B' to be cut off");
            // Everything fits → Some(cut_index), not None.
            let cut_index = cut_index.expect("Expected a cut index, got None");
            // Cut index must be before "Block B" — only assert it's positive and
            // less than the full HTML length, not an exact value.
            assert!(cut_index > 0, "Cut index should be positive");
            assert!(cut_index < html.len(), "Cut index should not be at the very end");
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn multiple_blocks_only_fitting_ones_present() {
            // Three blocks separated by large gaps. Viewport fits only the first.
            let viewport = make_viewport(100);
            let html = r#"
                <p style="margin:0;line-height:40px">Alpha</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Beta</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Gamma</p>"#;
            cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            let content = viewport.text_content().unwrap_or_default();
            assert!(content.contains("Alpha"));
            assert!(!content.contains("Beta"));
            assert!(!content.contains("Gamma"));
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn cut_index_falls_at_paragraph_boundary_not_mid_word() {
            // Block A fits. Block B does not. Cut must fall at the end of block A,
            // which is a clean paragraph boundary — not mid-word inside block A.
            // We verify by checking the returned index points to a '>' or whitespace,
            // indicating the cut is at a tag boundary.
            let viewport = make_viewport(100);
            let html = r#"<p style="margin:0;line-height:40px">Hello world</p><p style="margin:0;margin-top:500px;line-height:40px">Goodbye</p>"#;
            let cut_index = cut_forward(&viewport, html, 0)
                .expect("Expected a cut index, got None");
            assert_no_overflow(&viewport);
            // The character at cut_index in the original HTML should be at a clean boundary.
            let cut_char = html.as_bytes().get(cut_index).copied().map(|b| b as char);
            assert!(
                matches!(cut_char, Some('<') | Some('>') | Some(' ') | Some('\n') | None),
                "Cut index {cut_index} landed mid-word at char {:?}", cut_char
            );
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn returns_length_when_everything_fits() {
            // Content short enough to fully fit — no cut needed.
            // cut_forward() should return Some(html.len()), meaning "everything from char_start onward fit".
            let viewport = make_viewport(400);
            let html = r#"<p style="margin:0;line-height:40px">Only line</p>"#;
            let result = cut_forward(&viewport, html, 0);
            assert_eq!(result, Some(html.len()), "Expected length when all content fits, got {:?}", result);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn nonzero_char_start_skips_content_before_it() {
            // Full HTML has Block A and Block B separated by a large gap.
            // We pass char_start pointing to the start of Block B, so Block A
            // is skipped entirely. Viewport should contain Block B, not Block A.
            let viewport = make_viewport(600);
            let block_a = r#"<p style="margin:0;line-height:40px">Block A</p>"#;
            let block_b = r#"<p style="margin:0;margin-top:500px;line-height:40px">Block B</p>"#;
            let html = format!("{block_a}{block_b}");
            let char_start = block_a.len(); // skip Block A entirely
            cut_forward(&viewport, &html, char_start);
            assert_no_overflow(&viewport);
            let content = viewport.text_content().unwrap_or_default();
            assert!(content.contains("Block B"), "Expected 'Block B' when starting from block_b offset");
            assert!(!content.contains("Block A"), "Expected 'Block A' to be skipped via char_start");
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn healed_html_closes_open_tags() {
            // Cut falls inside a nested tag structure.
            // After cut, all opened tags must be closed — verified by checking
            // that every opening tag in innerHTML has a corresponding closing tag.
            let viewport = make_viewport(60);
            let html = r#"
                <div style="margin:0">
                    <p style="margin:0;line-height:20px"><b>Line one</b></p>
                    <p style="margin:0;line-height:20px"><b>Line two</b></p>
                    <p style="margin:0;line-height:20px"><b>Line three</b></p>
                </div>
                <div style="margin:0;margin-top:500px">
                    <p style="margin:0;line-height:20px"><b>Line four</b></p>
                </div>"#;
            cut_forward(&viewport, html, 0);
            assert_no_overflow(&viewport);
            let inner = viewport.inner_html();
            // Count opening vs closing <b> tags — they must balance.
            let open_b = inner.matches("<b>").count();
            let close_b = inner.matches("</b>").count();
            assert_eq!(open_b, close_b, "Unbalanced <b> tags after cut: {inner}");
            let open_p = inner.matches("<p").count();
            let close_p = inner.matches("</p>").count();
            assert_eq!(open_p, close_p, "Unbalanced <p> tags after cut: {inner}");
            cleanup(&viewport);
        }
    }



    // =============================================================================
    // mod char_start — nonzero char_start with a cut needed
    // All returned indices are global (relative to the full HTML string, not char_start).
    // =============================================================================
    mod char_start {
        use super::*;
    
        #[wasm_bindgen_test]
        fn cut_index_is_global_not_relative_to_char_start() {
            // Block A (skipped), Block B (rendered, fits), Block C (cut off via large gap).
            // char_start points to Block B. Cut happens somewhere before Block C.
            // The returned index must be > block_a.len(), proving it is global.
            let block_a = r#"<p style="margin:0;line-height:40px">Block A</p>"#;
            let block_b = r#"<p style="margin:0;line-height:40px">Block B</p>"#;
            let block_c = r#"<p style="margin:0;margin-top:500px;line-height:40px">Block C</p>"#;
            let html = format!("{block_a}{block_b}{block_c}");
            let char_start = block_a.len();
            let viewport = make_viewport(100);
            let result = cut_forward(&viewport, &html, char_start)
                .expect("Expected a cut index, got None");
            assert!(
                result > char_start,
                "Cut index {result} should be greater than char_start {char_start} — must be global"
            );
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    
        #[wasm_bindgen_test]
        fn content_from_char_start_is_rendered_and_cut_forward() {
            // Block A is skipped via char_start. Block B and Block C follow.
            // Viewport fits Block B but not Block C (large gap).
            // Viewport must contain Block B, not Block A or Block C.
            let block_a = r#"<p style="margin:0;line-height:40px">Block A</p>"#;
            let block_b = r#"<p style="margin:0;line-height:40px">Block B</p>"#;
            let block_c = r#"<p style="margin:0;margin-top:500px;line-height:40px">Block C</p>"#;
            let html = format!("{block_a}{block_b}{block_c}");
            let char_start = block_a.len();
            let viewport = make_viewport(100);
            cut_forward(&viewport, &html, char_start);
            assert_no_overflow(&viewport);
            let content = viewport.text_content().unwrap_or_default();
            assert!(content.contains("Block B"), "Expected 'Block B' to be rendered");
            assert!(!content.contains("Block A"), "Expected 'Block A' to be skipped via char_start");
            assert!(!content.contains("Block C"), "Expected 'Block C' to be cut off");
            cleanup(&viewport);
        }
    
        #[wasm_bindgen_test]
        fn cut_index_advances_correctly_across_two_forward_steps() {
            // Simulates two drive_forward calls on the same HTML.
            // Step 1: char_start=0, cut after Block A → Some(idx1).
            // Step 2: char_start=idx1, cut after Block B → Some(idx2).
            // idx2 must be > idx1, and viewport must contain Block B not Block A or Block C.
            let block_a = r#"<p style="margin:0;line-height:40px">Block A</p>"#;
            let block_b = r#"<p style="margin:0;line-height:40px">Block B</p>"#;
            let block_c = r#"<p style="margin:0;margin-top:40px;line-height:40px">Block C</p>"#;
            let block_d = r#"<p style="margin:0;margin-top:100px;line-height:40px">Block d</p>"#;
            let html = format!("{block_a}{block_b}{block_c}{block_d}");
            let viewport = make_viewport(100);
    
            // Step 1
            let idx1 = cut_forward(&viewport, &html, 0)
                .expect("Step 1: expected a cut index");
            assert!(idx1 > 0, "Step 1: cut index should be positive");
    
            // Step 2
            let idx2 = cut_forward(&viewport, &html, idx1)
                .expect("Step 2: expected a cut index");
            assert!(idx2 > idx1, "Step 2: cut index {idx2} should be greater than step 1 index {idx1}");
    
            assert_no_overflow(&viewport);
            let content = viewport.text_content().unwrap_or_default();
            assert!(content.contains("Block C"), "Expected 'Block C' on second page");
            assert!(!content.contains("Block A"), "Expected 'Block A' to be behind char_start");
            assert!(!content.contains("Block D"), "Expected 'Block D' to be cut off");
            cleanup(&viewport);
        }
    
        #[wasm_bindgen_test]
        fn char_start_at_last_block_returns_length_when_it_fits() {
            // char_start points to the last block, which fits entirely.
            // No cut needed → Some(html.len()).
            let block_a = r#"<p style="margin:0;line-height:40px">Block A</p>"#;
            let block_b = r#"<p style="margin:0;line-height:40px">Block B</p>"#;
            let html = format!("{block_a}{block_b}");
            let char_start = block_a.len();
            let viewport = make_viewport(200);
            let result = cut_forward(&viewport, &html, char_start);
            assert_eq!(result, Some(html.len()), "Expected length when last block fits entirely, got {:?}", result);
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    
        #[wasm_bindgen_test]
        fn many_blocks_cut_from_middle_global_index_correct() {
            // 10 blocks. char_start skips the first 5.
            // Viewport fits only 2 of the remaining blocks (each 40px, viewport 100px).
            // Cut must happen after block 7 (0-indexed), and returned index must be
            // greater than the start of block 5.
            let blocks: Vec<String> = (0..100)
                .map(|i| format!(r#"<p style="margin:0;font-size:40px">Block {i}</p>"#))
                .collect();
            let html = blocks.join("");
            let char_start: usize = blocks[..5].iter().map(|b| b.len()).sum();
            let viewport = make_viewport(100);
            let result = cut_forward(&viewport, &html, char_start)
                .expect("Expected a cut index for partial fit");
            assert!(
                result > char_start,
                "Cut index {result} must be global (> char_start {char_start})"
            );
            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    }
}


mod cut_backward {
    use super::*;


    /// Asserts that every descendant element of `root` has its top >=
    /// root.top, using getBoundingClientRect.
    ///
    /// Traversal is performed bottom-up because cut_backward keeps content
    /// anchored to the viewport bottom and removes overflowing content above.
    ///
    /// Panics with the offending element's outerHTML on failure.
    fn assert_no_overflow(viewport: &HtmlElement) {
        let vr = viewport.get_bounding_client_rect();
        let v_top = vr.top();
        assert_all_descendants_fit_bottom_up(viewport, v_top);
    }

    fn assert_all_descendants_fit_bottom_up(el: &HtmlElement, v_top: f64) {
        let children = el.children();

        // Bottom-up traversal.
        for i in (0..children.length()).rev() {
            let child = children
                .item(i).unwrap()
                .dyn_into::<HtmlElement>().unwrap();

            assert_all_descendants_fit_bottom_up(&child, v_top);

            let r = child.get_bounding_client_rect();

            assert!(
                r.top() >= v_top - 0.5,
                "Element top {:.1} is above viewport top {:.1}: {}",
                r.top(),
                v_top,
                child.outer_html()
            );
        }
    }

    // =============================================================================
    // mod core — fundamental visual invariant
    // =============================================================================
    mod core {
        use super::*;

        #[wasm_bindgen_test]
        fn short_text_fits_without_cut_backward() {
            // Text so short it fits entirely; cut should be a no-op or cut at the beginning.
            let viewport = make_viewport(400);
            let html = r#"<p style="margin:0">Hello world</p>"#;

            cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn long_text_is_cut_to_fit_from_bottom() {
            // Enough paragraphs that they cannot all fit in 200px at 20px line-height.
            // cut_backward should preserve the latest content and trim from the top.
            let viewport = make_viewport(200);

            let html = (0..30)
                .map(|i| {
                    format!(r#"<p style="margin:0;line-height:20px">Line {i}</p>"#)
                })
                .collect::<Vec<_>>()
                .join("");

            cut_backward(&viewport, &html, html.len());

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn nested_elements_do_not_overflow_top() {
            let viewport = make_viewport(100);

            let html = r#"
                <div style="margin:0">
                    <div style="margin:0">
                        <p style="margin:0;line-height:20px">Line 1</p>
                        <p style="margin:0;line-height:20px">Line 2</p>
                        <p style="margin:0;line-height:20px">Line 3</p>
                        <p style="margin:0;line-height:20px">Line 4</p>
                        <p style="margin:0;line-height:20px">Line 5</p>
                        <p style="margin:0;line-height:20px">Line 6</p>
                        <p style="margin:0;line-height:20px">Line 7</p>
                        <p style="margin:0;line-height:20px">Line 8</p>
                    </div>
                </div>"#;

            cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn single_oversized_element_does_not_crash() {
            // One element taller than the viewport; acceptable fallback.
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:20px;padding:200px 0">
                    Tall paragraph
                </p>"#;

            cut_backward(&viewport, html, html.len());

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn empty_html_does_not_crash() {
            let viewport = make_viewport(200);

            cut_backward(&viewport, "", 0);

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn viewport_content_is_valid_html_after_cut_backward() {
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:20px">
                    <b>Bold <i>italic text</i> end</b>
                </p>
                <p style="margin:0;line-height:20px">Second paragraph</p>
                <p style="margin:0;line-height:20px">Third paragraph</p>
                <p style="margin:0;line-height:20px">Fourth paragraph</p>
                <p style="margin:0;line-height:20px">Fifth paragraph</p>"#;

            cut_backward(&viewport, html, html.len());

            let inner = viewport.inner_html();

            let document = window().unwrap().document().unwrap();
            let probe = document.create_element("div").unwrap();

            probe.set_inner_html(&inner);

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    }

    // =============================================================================
    // mod boundary — edge cases around the cut point
    // =============================================================================
    mod boundary {
        use super::*;

        #[wasm_bindgen_test]
        fn cut_exactly_at_paragraph_boundary_from_bottom() {
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:50px">First</p>
                <p style="margin:0;line-height:50px">Second</p>
                <p style="margin:0;line-height:50px">Third</p>"#;

            cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn single_char_over_boundary_is_cut_backward() {
            let viewport = make_viewport(100);

            let html = (0..6)
                .map(|i| {
                    format!(r#"<p style="margin:0;line-height:20px">Line {i}</p>"#)
                })
                .collect::<Vec<_>>()
                .join("");

            cut_backward(&viewport, &html, html.len());

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn zero_height_viewport_does_not_crash() {
            let viewport = make_viewport(0);

            let html = r#"
                <p style="margin:0;line-height:20px">Some text</p>"#;

            cut_backward(&viewport, html, html.len());

            cleanup(&viewport);
        }
    }

    // =============================================================================
    // mod unambiguous — content and index assertions where cut point cannot vary
    // =============================================================================
    mod unambiguous {
        use super::*;

        #[wasm_bindgen_test]
        fn content_after_cut_is_preserved() {
            // Block A cannot fit because of huge gap.
            // cut_backward should preserve Block B.
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:40px">Block A</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Block B</p>"#;

            let cut_index = cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);

            let content = viewport.text_content().unwrap_or_default();

            assert!(!content.contains("Block A"));
            assert!(content.contains("Block B"));

            let cut_index = cut_index.expect("Expected a cut index");

            assert!(cut_index > 0);
            assert!(cut_index < html.len());

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn multiple_blocks_only_latest_fitting_ones_present() {
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:40px">Alpha</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Beta</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Gamma</p>"#;

            cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);

            let content = viewport.text_content().unwrap_or_default();

            assert!(!content.contains("Alpha"));
            assert!(!content.contains("Beta"));
            assert!(content.contains("Gamma"));

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn cut_index_falls_at_paragraph_boundary_not_mid_word() {
            let viewport = make_viewport(100);

            let html = r#"
                <p style="margin:0;line-height:40px">Hello world</p>
                <p style="margin:0;margin-top:500px;line-height:40px">Goodbye</p>"#;

            let cut_index = cut_backward(&viewport, html, html.len())
                .expect("Expected a cut index");

            assert_no_overflow(&viewport);

            let cut_char = html
                .as_bytes()
                .get(cut_index)
                .copied()
                .map(|b| b as char);

            assert!(
                matches!(
                    cut_char,
                    Some('<') | Some('>') | Some(' ') | Some('\n') | None
                ),
                "Cut index {cut_index} landed mid-word at char {:?}",
                cut_char
            );

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn returns_none_when_everything_fits() {
            let viewport = make_viewport(400);

            let html = r#"
                <p style="margin:0;line-height:40px">Only line</p>"#;

            let result = cut_backward(&viewport, html, html.len());

            assert!(
                result == None,
                "Expected None when all content fits, got {:?}",
                result
            );

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn nonzero_char_end_skips_content_after_it() {
            // cut_backward renders content BEFORE char_end.
            let viewport = make_viewport(600);

            let block_a =
                r#"<p style="margin:0;line-height:40px">Block A</p>"#;

            let block_b =
                r#"<p style="margin:0;margin-top:500px;line-height:40px">Block B</p>"#;

            let html = format!("{block_a}{block_b}");

            let char_end = block_a.len();

            cut_backward(&viewport, &html, char_end);

            assert_no_overflow(&viewport);

            let content = viewport.text_content().unwrap_or_default();

            assert!(content.contains("Block A"));
            assert!(!content.contains("Block B"));

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn healed_html_closes_open_tags() {
            let viewport = make_viewport(60);

            let html = r#"
                <div style="margin:0;margin-top:500px">
                    <p style="margin:0;line-height:20px"><b>Line one</b></p>
                </div>

                <div style="margin:0">
                    <p style="margin:0;line-height:20px"><b>Line two</b></p>
                    <p style="margin:0;line-height:20px"><b>Line three</b></p>
                    <p style="margin:0;line-height:20px"><b>Line four</b></p>
                </div>"#;

            cut_backward(&viewport, html, html.len());

            assert_no_overflow(&viewport);

            let inner = viewport.inner_html();

            let open_b = inner.matches("<b>").count();
            let close_b = inner.matches("</b>").count();

            assert_eq!(open_b, close_b);

            let open_p = inner.matches("<p").count();
            let close_p = inner.matches("</p>").count();

            assert_eq!(open_p, close_p);

            cleanup(&viewport);
        }
    }

    // =============================================================================
    // mod char_end — nonzero char_end with a cut needed
    // All returned indices are global.
    // =============================================================================
    mod char_end {
        use super::*;

        #[wasm_bindgen_test]
        fn cut_index_is_global_not_relative_to_char_end() {
            let block_a =
                r#"<p style="margin:0;margin-top:500px;line-height:40px">Block A</p>"#;

            let block_b =
                r#"<p style="margin:0;line-height:80px">Block B</p>"#;

            let block_c =
                r#"<p style="margin:0;line-height:40px">Block C</p>"#;

            let html = format!("{block_a}{block_b}{block_c}");

            let char_end = html.len() - block_c.len();

            let viewport = make_viewport(100);

            let result = cut_backward(&viewport, &html, char_end)
                .expect("Expected a cut index");

            assert!(
                result < char_end,
                "Cut index {result} should be less than char_end {char_end}"
            );

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn content_before_char_end_is_rendered_and_cut_backward() {
            let block_a =
                r#"<p style="margin:0;margin-top:500px;line-height:40px">Block A</p>"#;

            let block_b =
                r#"<p style="margin:0;line-height:80px">Block B</p>"#;

            let block_c =
                r#"<p style="margin:0;line-height:40px">Block C</p>"#;

            let html = format!("{block_a}{block_b}{block_c}");

            let char_end = html.len() - block_c.len();

            let viewport = make_viewport(100);

            cut_backward(&viewport, &html, char_end);

            assert_no_overflow(&viewport);

            let content = viewport.text_content().unwrap_or_default();

            assert!(!content.contains("Block A"));
            assert!(content.contains("Block B"));
            assert!(!content.contains("Block C"));

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn cut_index_moves_backward_across_two_steps() {
            let block_a =
                r#"<p style="margin:0;margin-top:100px;line-height:40px">Block A</p>"#;

            let block_b =
                r#"<p style="margin:0;margin-top:100px;line-height:40px">Block B</p>"#;

            let block_c =
                r#"<p style="margin:0;line-height:40px">Block C</p>"#;

            let block_d =
                r#"<p style="margin:0;line-height:40px">Block D</p>"#;

            let html = format!("{block_a}{block_b}{block_c}{block_d}");

            let viewport = make_viewport(100);

            // Step 1
            let idx1 = cut_backward(&viewport, &html, html.len())
                .expect("Expected step 1 cut index");

            console(&format!("Step 1 cut index: {}, rebdered  {}", idx1, &html[idx1..]));
            // Step 2
            let idx2 = cut_backward(&viewport, &html, idx1)
                .expect("Expected step 2 cut index");

            assert!(
                idx2 < idx1,
                "Expected idx2 ({idx2}) < idx1 ({idx1})"
            );

            assert_no_overflow(&viewport);

            let content = viewport.text_content().unwrap_or_default();

            assert!(content.contains("Block B"));
            assert!(!content.contains("Block D"));

            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn char_end_at_first_block_returns_none_when_it_fits() {
            let block_a =
                r#"<p style="margin:0;line-height:40px">Block A</p>"#;

            let block_b =
                r#"<p style="margin:0;line-height:40px">Block B</p>"#;

            let html = format!("{block_a}{block_b}");

            let char_end = block_a.len();

            let viewport = make_viewport(200);

            let result = cut_backward(&viewport, &html, char_end);

            assert!(
                result.is_none(),
                "Expected None when first block fits"
            );

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }

        #[wasm_bindgen_test]
        fn many_blocks_cut_from_middle_global_index_correct() {
            let blocks: Vec<String> = (0..10)
                .map(|i| {
                    format!(
                        r#"<p style="margin:0;line-height:40px">Block {i}</p>"#
                    )
                })
                .collect();

            let html = blocks.join("");

            let char_end: usize =
                blocks[..8].iter().map(|b| b.len()).sum();

            let viewport = make_viewport(100);

            let result = cut_backward(&viewport, &html, char_end)
                .expect("Expected cut index");

            assert!(
                result < char_end,
                "Cut index {result} should be before char_end {char_end}"
            );

            assert_no_overflow(&viewport);
            cleanup(&viewport);
        }
    }
}