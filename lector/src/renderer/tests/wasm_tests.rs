use std::sync::Arc;

// tests/dom_smoke_test.rs
use wasm_bindgen_test::*;
use web_sys::{window, HtmlElement,Element};
use wasm_bindgen::JsCast;
use crate::renderer::calculate_page_height::LayoutQuery;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn dom_element_has_nonzero_rect_when_attached() {
    let document = window().unwrap().document().unwrap();

    // 1. Create and style a span
    let span = document.create_element("span").unwrap();
    let span_html: &HtmlElement = span.dyn_ref().unwrap();
    
    // Use HtmlElement trait for style() method
    span_html.style().set_property("display", "inline-block").unwrap();
    span_html.style().set_property("font-size", "16px").unwrap();
    span_html.style().set_property("font-family", "monospace").unwrap();
    span.set_text_content(Some("hello"));

    // 2. Attach it — this is what gives it a real rect
    document.body().unwrap().append_child(&span).unwrap();

    // 3. Read back from DOM
    let rect = span.get_bounding_client_rect();

    // 4. Assert we got real dimensions
    assert!(rect.width() > 0.0,  "width should be > 0, got {}", rect.width());
    assert!(rect.height() > 0.0, "height should be > 0, got {}", rect.height());

    // Cleanup
    document.body().unwrap().remove_child(&span).unwrap();
}


use crate::renderer::layout_builder::build_layout;

#[wasm_bindgen_test]
fn build_layout_simple_text_element() {
    let document = window().unwrap().document().unwrap();
    
    let span = document.create_element("span").unwrap();
    span.set_text_content(Some("hello world"));
    document.body().unwrap().append_child(&span).unwrap();
    
    let layout = build_layout(&span,0);
    

    assert_eq!(layout.text, "<span>hello world</span>");
    assert_eq!(layout.text_len(), 24);
    assert!(layout.children.is_empty());
    
    document.body().unwrap().remove_child(&span).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_nested_elements() {
    let document = window().unwrap().document().unwrap();
    
    let parent = document.create_element("div").unwrap();
    let child = document.create_element("span").unwrap();
    child.set_text_content(Some("nested"));
    parent.append_child(&child).unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    
    let layout = build_layout(&parent,0);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].text, "<span>nested</span>");
    
    document.body().unwrap().remove_child(&parent).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_text_and_children() {
    let document = window().unwrap().document().unwrap();
    
    let p = document.create_element("p").unwrap();
    p.set_text_content(Some("test"));
    
    let b = document.create_element("b").unwrap();
    b.set_text_content(Some("bold"));
    p.append_child(&b).unwrap();
    
    document.body().unwrap().append_child(&p).unwrap();
    
    let layout = build_layout(&p, 0);
    
    // Parent has text "test" and one child
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 2);
    assert_eq!(layout.children[0].text, "test");
    assert_eq!(layout.children[1].text, "<b>bold</b>");
    
    document.body().unwrap().remove_child(&p).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_multiple_children() {
    let document = window().unwrap().document().unwrap();
    
    let div = document.create_element("div").unwrap();
    
    let span1 = document.create_element("span").unwrap();
    span1.set_text_content(Some("first"));
    div.append_child(&span1).unwrap();
    
    let span2 = document.create_element("span").unwrap();
    span2.set_text_content(Some("second"));
    div.append_child(&span2).unwrap();
    
    let span3 = document.create_element("span").unwrap();
    span3.set_text_content(Some("third"));
    div.append_child(&span3).unwrap();
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 3);
    assert_eq!(layout.children[0].text, "<span>first</span>");
    assert_eq!(layout.children[1].text, "<span>second</span>");
    assert_eq!(layout.children[2].text, "<span>third</span>");
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_deeply_nested() {
    let document = window().unwrap().document().unwrap();
    
    let outer = document.create_element("div").unwrap();
    let middle = document.create_element("div").unwrap();
    let inner = document.create_element("span").unwrap();
    
    inner.set_text_content(Some("deep"));
    middle.append_child(&inner).unwrap();
    outer.append_child(&middle).unwrap();
    document.body().unwrap().append_child(&outer).unwrap();
    
    let layout = build_layout(&outer, 0);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].text, "");
    assert_eq!(layout.children[0].children.len(), 1);
    assert_eq!(layout.children[0].children[0].text, "<span>deep</span>");
    
    document.body().unwrap().remove_child(&outer).unwrap();
}

// ... existing code ...


#[wasm_bindgen_test]
fn build_layout_measures_simple_element_height() {
    let document = window().unwrap().document().unwrap();
    
    let div_el = document.create_element("div").unwrap();
    let div: &HtmlElement = div_el.dyn_ref().unwrap();
    div.style().set_property("display", "block").unwrap();
    div.style().set_property("width", "100px").unwrap();
    div.style().set_property("height", "50px").unwrap();
    div.style().set_property("background-color", "red").unwrap();
    div.set_text_content(Some("test"));
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    // Verify structure is correct
    assert!(!layout.text.is_empty());
    assert!(layout.children.is_empty());
    
    // Get actual DOM rect for comparison
    let rect = div.get_bounding_client_rect();
    let expected_top = rect.top();
    let expected_bottom = rect.bottom(); // Assuming top is 0 for this test
    
    assert!(
        layout.top == expected_top,
        "LayoutQuery.top should be populated with actual DOM measurement. \
         Expected {}, got {}. \
         This test documents the requirement that build_layout must measure \
         element dimensions and set top/bottom fields.",
        expected_top,
        layout.top
    );
    assert!(
        layout.bottom-layout.top == rect.height(),
        "LayoutQuery.bottom - LayoutQuery.top should match actual DOM height. \
         Expected {}, got {}. \
         This test documents the requirement that build_layout must measure \
         element dimensions and set top/bottom fields.",
        rect.height(),
        layout.bottom - layout.top
    );

    assert!(
        layout.bottom == rect.bottom(),
        "LayoutQuery.bottom should be populated with actual DOM measurement. \
         Expected {}, got {}. \
         This test documents the requirement that build_layout must measure \
         element dimensions and set top/bottom fields.",
        rect.bottom(),
        layout.bottom
    );
    
    // Additional check: bottom should match the actual element height
    assert!(
        (layout.bottom - expected_bottom).abs() < 1.0,
        "LayoutQuery.bottom ({}) should match actual DOM height ({}) within 1px tolerance",
        layout.bottom,
        expected_bottom
    );
    
    document.body().unwrap().remove_child(&div).unwrap();
}
#[wasm_bindgen_test]
fn build_layout_text_only_element_has_no_children() {
    let document = window().unwrap().document().unwrap();
    
    let p = document.create_element("p").unwrap();
    p.set_text_content(Some("Just text content"));
    document.body().unwrap().append_child(&p).unwrap();
    
    let layout = build_layout(&p, 0);
    
    assert_eq!(layout.children.len(), 0);
    assert!(!layout.text.is_empty());
    
    document.body().unwrap().remove_child(&p).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_element_with_mixed_content() {
    let document = window().unwrap().document().unwrap();
    
    let div_el = document.create_element("div").unwrap();
    let div: &HtmlElement= div_el.dyn_ref().unwrap();
    div.style().set_property("font-size", "16px").unwrap();
    
    // Create text node directly (not via set_text_content)
    let text_node1 = document.create_text_node("Text before ");
    div.append_child(&text_node1).unwrap();
    
    // Add child element
    let span = document.create_element("span").unwrap();
    span.set_text_content(Some("Child"));
    div.append_child(&span).unwrap();
    
    // Add more text node
    let text_node2 = document.create_text_node(" and after");
    div.append_child(&text_node2).unwrap();
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    // Should have children for both text nodes and element
    assert_eq!(layout.children.len(), 3);
    assert_eq!(layout.text, "");
    assert!(layout.children[0].text.contains("Text before"));
    assert_eq!(layout.children[1].text, "<span>Child</span>");
    assert!(layout.children[2].text.contains("and after"));
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_text_node_extraction() {
    let document = window().unwrap().document().unwrap();
    
    let div_el = document.create_element("div").unwrap();
    let div: &HtmlElement= div_el.dyn_ref().unwrap();
    div.style().set_property("font-size", "16px").unwrap();
    
    // Create a text node
    let text_node = document.create_text_node("Hello World");
    div.append_child(&text_node).unwrap();
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    // Layout should capture the text content
    assert!(!layout.text.is_empty());
    assert!(layout.text.contains("Hello World"));
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_empty_element() {
    let document = window().unwrap().document().unwrap();
    
    let div = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    assert_eq!(layout.text, "<div></div>");
    assert!(layout.children.is_empty());
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_single_child_element() {
    let document = window().unwrap().document().unwrap();
    
    let parent_el = document.create_element("div").unwrap();
    let parent: &HtmlElement= parent_el.dyn_ref().unwrap();
    parent.style().set_property("width", "300px").unwrap();
    parent.style().set_property("height", "150px").unwrap();
    
    let child_el = document.create_element("span").unwrap();
    let child: &HtmlElement= child_el.dyn_ref().unwrap();
    child.style().set_property("font-size", "20px").unwrap();
    child.set_text_content(Some("Single child"));
    parent.append_child(&child).unwrap();
    
    document.body().unwrap().append_child(&parent).unwrap();
    
    let layout = build_layout(&parent, 0);
    
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].text, "<span style=\"font-size: 20px;\">Single child</span>");
    
    document.body().unwrap().remove_child(&parent).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_multiple_siblings() {
    let document = window().unwrap().document().unwrap();
    
    let container_el = document.create_element("div").unwrap();
    let container: &HtmlElement= container_el.dyn_ref().unwrap();
    container.style().set_property("width", "500px").unwrap();
    container.style().set_property("height", "300px").unwrap();
    
    for i in 0..5 {
        let span = document.create_element("span").unwrap();
        span.set_text_content(Some(format!("Item {}", i).as_str()));
        container.append_child(&span).unwrap();
    }
    
    document.body().unwrap().append_child(&container).unwrap();
    
    let layout = build_layout(&container, 0);
    
    assert_eq!(layout.children.len(), 5);
    for i in 0..5 {
        assert!(layout.children[i].text.contains(&format!("Item {}", i)));
    }
    
    document.body().unwrap().remove_child(&container).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_deeply_nested_structure() {
    let document = window().unwrap().document().unwrap();
    
    // Create 5 levels of nesting
    let level1 = document.create_element("div").unwrap();
    let level2 = document.create_element("div").unwrap();
    let level3 = document.create_element("div").unwrap();
    let level4 = document.create_element("div").unwrap();
    let level5 = document.create_element("span").unwrap();
    
    level5.set_text_content(Some("deepest"));
    level4.append_child(&level5).unwrap();
    level3.append_child(&level4).unwrap();
    level2.append_child(&level3).unwrap();
    level1.append_child(&level2).unwrap();
    
    document.body().unwrap().append_child(&level1).unwrap();
    
    let layout = build_layout(&level1, 0);
    
    // Verify depth
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].children.len(), 1);
    assert_eq!(layout.children[0].children[0].children.len(), 1);
    assert_eq!(layout.children[0].children[0].children[0].children.len(), 1);
    assert_eq!(layout.children[0].children[0].children[0].children[0].text, "<span>deepest</span>");
    
    document.body().unwrap().remove_child(&level1).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_with_styles_affects_rect() {
    let document = window().unwrap().document().unwrap();
    
    let div_el = document.create_element("div").unwrap();
    let div: &HtmlElement= div_el.dyn_ref().unwrap();
    
    // Different font sizes should produce different heights
    div.style().set_property("font-size", "12px").unwrap();
    div.set_text_content(Some("Small"));
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    let rect_small = div.get_bounding_client_rect();
    
    // Clear and create with larger font
    document.body().unwrap().remove_child(&div).unwrap();
    
    let div2_el = document.create_element("div").unwrap();
    let div2: &HtmlElement= div2_el.dyn_ref().unwrap();
    div2.style().set_property("font-size", "24px").unwrap();
    div2.set_text_content(Some("Small"));
    document.body().unwrap().append_child(&div2).unwrap();
    
    let layout2 = build_layout(&div2, 0);
    let rect_large = div2.get_bounding_client_rect();
    
    // Larger font should have larger height
    assert!(rect_large.height() >= rect_small.height());
    
    document.body().unwrap().remove_child(&div2).unwrap();
}

// -------------------------------------------------------------------------
// LAYOUT QUERY STRUCTURE VERIFICATION
// -------------------------------------------------------------------------

#[wasm_bindgen_test]
fn layout_query_has_correct_text_field() {
    let document = window().unwrap().document().unwrap();
    
    let span = document.create_element("span").unwrap();
    span.set_text_content(Some("Test content"));
    document.body().unwrap().append_child(&span).unwrap();
    
    let layout = build_layout(&span, 0);
    
    // Text field should contain full HTML markup
    assert!(layout.text.contains("span"));
    assert!(layout.text.contains("Test content"));
    
    document.body().unwrap().remove_child(&span).unwrap();
}

#[wasm_bindgen_test]
fn layout_query_text_len_matches_content() {
    let document = window().unwrap().document().unwrap();
    
    let p = document.create_element("p").unwrap();
    p.set_text_content(Some("Hello World"));
    document.body().unwrap().append_child(&p).unwrap();
    
    let layout = build_layout(&p, 0);
    
    // text_len() should count characters in text field
    let expected_len = layout.text.chars().count();
    assert_eq!(layout.text_len(), expected_len as u32);
    
    document.body().unwrap().remove_child(&p).unwrap();
}

#[wasm_bindgen_test]
fn layout_query_children_are_in_dom_order() {
    let document = window().unwrap().document().unwrap();
    
    let div = document.create_element("div").unwrap();
    
    let first = document.create_element("span").unwrap();
    first.set_text_content(Some("First"));
    div.append_child(&first).unwrap();
    
    let second = document.create_element("span").unwrap();
    second.set_text_content(Some("Second"));
    div.append_child(&second).unwrap();
    
    let third = document.create_element("span").unwrap();
    third.set_text_content(Some("Third"));
    div.append_child(&third).unwrap();
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div, 0);
    
    // Children should be in DOM order
    assert_eq!(layout.children[0].text, "<span>First</span>");
    assert_eq!(layout.children[1].text, "<span>Second</span>");
    assert_eq!(layout.children[2].text, "<span>Third</span>");
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::build_layout;

    // Run tests in browser, not node
    wasm_bindgen_test_configure!(run_in_browser);

    /// Inject an element into the document body, run a test closure, then clean up.
    fn with_element<F: FnOnce(&web_sys::Element)>(html: &str, f: F) {
        let document = window().unwrap().document().unwrap();
        let container = document.create_element("div").unwrap();
        container.set_inner_html(html);

        // Must be in the DOM for getBoundingClientRect / Range to return real values
        document.body().unwrap()
            .append_child(&container).unwrap();

        f(&container);

        // Cleanup
        document.body().unwrap()
            .remove_child(&container).unwrap();
    }

    // ── Basic sanity: top < bottom ───────────────────────────────────────────

    #[wasm_bindgen_test]
    fn char_top_is_above_char_bottom() {
        with_element(r#"<p style="font-size:16px;line-height:20px;">Hello</p>"#, |container| {
            let p = container.first_element_child().unwrap();
            let layout = build_layout(&p, 0);

            let top    = (layout.get_char_top)(0);
            let bottom = (layout.get_char_bottom)(0);

            assert!(top < bottom, "top ({top}) should be less than bottom ({bottom})");
        });
    }

    // ── Characters on the same line share the same top/bottom ────────────────

    #[wasm_bindgen_test]
    fn same_line_chars_have_same_top_and_bottom() {
        with_element(r#"<p style="font-size:16px;line-height:20px;width:400px;">Hello</p>"#, |container| {
            let p = container.first_element_child().unwrap();
            let layout = build_layout(&p, 0);

            let top_h = (layout.get_char_top)(0); // 'H'
            let top_e = (layout.get_char_top)(1); // 'e'
            let top_l = (layout.get_char_top)(2); // 'l'

            assert_eq!(top_h, top_e, "'H' and 'e' should be on the same line");
            assert_eq!(top_e, top_l, "'e' and 'l' should be on the same line");
        });
    }

    // ── Characters on different lines have different tops ────────────────────

    #[wasm_bindgen_test]
    fn wrapped_chars_have_different_tops() {
        // Very narrow width forces a line break between the two words
        with_element(
            r#"<p style="font-size:16px;line-height:20px;width:20px;">A B</p>"#,
            |container| {
                let p = container.first_element_child().unwrap();
                let layout = build_layout(&p, 0);

                let top_a     = (layout.get_char_top)(0); // 'A' — line 1
                let top_space = (layout.get_char_top)(1); // ' ' — line 1
                let top_b     = (layout.get_char_top)(2); // 'B' — line 2

                assert_eq!(top_a, top_space, "'A' and ' ' should be on the same line");
                assert!(
                    top_b > top_a,
                    "'B' (top={top_b}) should be below 'A' (top={top_a}) after wrapping"
                );
            },
        );
    }

    // ── char_start offset threading ──────────────────────────────────────────

    #[wasm_bindgen_test]
    fn char_start_offsets_are_correct_across_children() {
        with_element(
            r#"<p style="font-size:16px;">Hello <span>World</span></p>"#,
            |container| {
                let p = container.first_element_child().unwrap();
                let layout = build_layout(&p, 0);

                // "Hello " is the first text node → chars 0..5
                // <span>World</span> starts at char 6
                let span_child = layout.children.iter()
                    .find(|c| c.char_start == 6)
                    .expect("span child should have char_start=6");
                println!("Span child char_start: {}", span_child.char_start);
                assert_eq!(span_child.char_start, 6);
                assert_eq!(span_child.text, "<span>World</span>");
                assert_eq!(span_child.text_len(), 18); // "World"

                // char 6 ('W') should be queryable via the parent too
                let top_via_parent = (layout.get_char_top)(6);
                let top_via_child  = (span_child.get_char_top)(6);
                assert_eq!(top_via_parent, top_via_child);
            },
        );
    }

    // ── Nested elements ──────────────────────────────────────────────────────

    #[wasm_bindgen_test]
    fn nested_elements_produce_correct_char_starts() {
        with_element(
            r#"<div style="font-size:16px;"><p>AB</p><p>CD</p></div>"#,
            |container| {
                let div = container.first_element_child().unwrap();
                let layout = build_layout(&div, 0);

                assert_eq!(layout.children.len(), 2);

                let first  = &layout.children[0]; // <p>AB</p>
                let second = &layout.children[1]; // <p>CD</p>

                assert_eq!(first.char_start,  0);
                assert_eq!(second.char_start, 9); // "AB" = 2 chars
            },
        );
    }

    // ── global char_start parameter is respected ─────────────────────────────

    #[wasm_bindgen_test]
    fn build_layout_respects_nonzero_char_start() {
        with_element(r#"<p style="font-size:16px;">Hi</p>"#, |container| {
            let p = container.first_element_child().unwrap();
            let layout = build_layout(&p, 100);

            assert_eq!(layout.char_start, 100);

            // Querying char 100 should give valid coordinates, not 0.0
            let top = (layout.get_char_top)(100);
            assert!(top > 0.0, "char 100 should map to a real y position");
        });
    }

    #[wasm_bindgen_test]
    fn build_layout_problem_html_structure() {
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        let html_content = r#"
            <div id="toc3_CHAPTER_ONE_Hugh_of_Emblin" class="class54">CHAPTER ONE</div>
            <div class="class56">Hugh of Emblin</div>
            <div class="class58">Hugh of Emblin wasn't good at much, but he was very, very good at hiding. Which was good, because he really needed to be.</div>
            <div class="class60">"Where are you hiding, sheepherder? The longer it takes us to find you, the worse it will be for you!"</div>
            <div class="class60">Hugh slid farther back into the space behind the bookshelf. Rhodes and his friends might have chosen him as their favorite victim, but their attention span usually wasn't too long. If he stayed hidden long enough, they'd eventually get bored and find something else to amuse themselves.</div>
            <div class="class95">Hugh, thankfully enough, didn't run into Rhodes and his lackeys on the way to his next class.</div>
        "#;
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        let layout = build_layout(&container, 0);
        
        // Verify structure
        assert_eq!(layout.children.len(), 6, "Should have 6 child div elements");
        
        // First child: CHAPTER ONE header
        assert!(layout.children[0].text.contains("CHAPTER ONE"));
        
        // Second child: Title
        assert!(layout.children[1].text.contains("Hugh of Emblin"));
        
        // Third child: Opening paragraph
        assert!(layout.children[2].text.contains("wasn't good at much"));
        
        // Fourth child: Dialogue
        assert!(layout.children[3].text.contains("sheepherder"));
        
        // Fifth child: Section break   
        assert!(layout.children[4].text.contains("slid farther back"));

        // Sixth child: Section break
        assert!(layout.children[5].text.contains("thankfully enough"));
        
        document.body().unwrap().remove_child(&container_el).unwrap();
    }

    #[wasm_bindgen_test]
    fn build_layout_problem_html_text_extraction() {
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        let html_content = r#"
            <div class="class60">Hugh slid farther back into the space behind the bookshelf.</div>
            <div class="class60">Rhodes and his friends might have chosen him as their favorite victim.</div>
        "#;
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        let layout = build_layout(&container, 0);
        
        // Verify all text content is extracted
        let total_text_len: u32 = layout.children.iter()
            .map(|c| c.text_len())
            .sum();
        
        assert!(total_text_len > 100, "Should have significant text content");
        
        // Verify char_start offsets are correct across siblings
        assert_eq!(layout.children[0].char_start, 0);
        assert!(layout.children[1].char_start > layout.children[0].char_start);
        
        document.body().unwrap().remove_child(&container_el).unwrap();
    }

    #[wasm_bindgen_test]
    fn build_layout_problem_html_nested_structure() {
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // Test nested dialogue with potential inline elements
        let html_content = r#"
            <div class="class60">
                "Where are you hiding, sheepherder?" a voice asked.
                <span class="class60">Hugh took a deep breath and looked up.</span>
            </div>
        "#;
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        let layout = build_layout(&container, 0);
        
        // Should have nested structure
        assert_eq!(layout.children.len(), 1);
        assert!(layout.children[0].children.len() >= 1);
        
        // Verify nested element is captured
        let has_span = layout.children[0].children.iter()
            .any(|c| c.text.contains("span"));
        assert!(has_span, "Should have nested span element");
        
        document.body().unwrap().remove_child(&container_el).unwrap();
    }

    #[wasm_bindgen_test]
    fn build_layout_problem_html_char_coordinates() {
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.style().set_property("line-height", "20px").unwrap();
        
        let html_content = r#"
            <div class="class60">Hello World</div>
        "#;
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        let layout = build_layout(&container, 0);
        
        // Verify char coordinates are queryable
        let top_h = (layout.get_char_top)(0);
        let top_o = (layout.get_char_top)(1);
        
        // Same line should have same top
        assert_eq!(top_h, top_o, "Characters on same line should have same top");
        
        // Top should be > 0 (not defaulting to 0)
        assert!(top_h > 0.0, "Character coordinates should be populated");
        
        document.body().unwrap().remove_child(&container_el).unwrap();
    }

    #[wasm_bindgen_test]
    fn layout_len_shoud_match_html(){
                let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        let html_content = r#"
            <div id="toc3_CHAPTER_ONE_Hugh_of_Emblin" class="class54">CHAPTER ONE</div>
            <div class="class56">Hugh of Emblin</div>
            <div class="class58">Hugh of Emblin wasn't good at much, but he was very, very good at hiding. Which was good, because he really needed to be.</div>
            <div class="class60">"Where are you hiding, sheepherder? The longer it takes us to find you, the worse it will be for you!"</div>
            <div class="class60">Hugh slid farther back into the space behind the bookshelf. Rhodes and his friends might have chosen him as their favorite victim, but their attention span usually wasn't too long. If he stayed hidden long enough, they'd eventually get bored and find something else to amuse themselves.</div>
            <div class="class95">Hugh, thankfully enough, didn't run into Rhodes and his lackeys on the way to his next class.</div>
        "#;
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        let layout = build_layout(&container, 0);

        assert_eq!(layout.text_len(), html_content.len() as u32);
    }
}




