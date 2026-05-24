use std::fmt::format;

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{window, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a `<div>` that acts as the viewport with explicit pixel dimensions.
/// The element is appended to `document.body` so it participates in layout,
/// and is returned together with a cleanup closure you should call at the end
/// of each test.
fn make_viewport(width_px: u32, height_px: u32) -> HtmlElement {
    let document = window().unwrap().document().unwrap();
    let viewport: HtmlElement = document
        .create_element("div")
        .unwrap()
        .dyn_into()
        .unwrap();

    let style = viewport.style();
    style.set_property("width",    &format!("{width_px}px")).unwrap();
    style.set_property("height",   &format!("{height_px}px")).unwrap();
    style.set_property("overflow", "hidden").unwrap();
    // Prevent surrounding content from interfering with the rect measurements.
    style.set_property("position", "fixed").unwrap();
    style.set_property("top",      "0").unwrap();
    style.set_property("left",     "0").unwrap();

    document.body().unwrap().append_child(&viewport).unwrap();
    viewport
}

/// Remove the viewport from the DOM after a test.
fn cleanup(viewport: &HtmlElement) {
    viewport.set_inner_html("");
    let document = window().unwrap().document().unwrap();
    document.body().unwrap().remove_child(viewport).ok();
}

/// Return the bottom edge (in px, relative to the viewport's own top) of the
/// *last* element that has a non-zero bounding rect inside `viewport`.
///
/// We query every element that is a descendant of `viewport`, compute its
/// rect relative to the viewport's rect, and track the maximum bottom edge.
fn last_child_bottom_relative(viewport: &HtmlElement) -> f64 {
    let viewport_rect = viewport.get_bounding_client_rect();
    let viewport_top  = viewport_rect.top();


    let viewport_el: &web_sys::Element = viewport.as_ref();
    let collection = viewport_el.get_elements_by_tag_name("*");
    let mut max_bottom: f64 = 0.0;

    for i in 0..collection.length() {
        if let Some(el) = collection.item(i) {
            let rect   = el.get_bounding_client_rect();
            // Ignore invisible / zero-size nodes.
            if rect.width() == 0.0 && rect.height() == 0.0 {
                continue;
            }
            let bottom_relative = rect.bottom() - viewport_top;
            if bottom_relative > max_bottom {
                max_bottom = bottom_relative;
            }
        }
    }
    max_bottom
}

// ---------------------------------------------------------------------------
// The function under test – adapt the import path to your actual module.
// ---------------------------------------------------------------------------
//
// Expected signature (adjust if yours differs):
//
//   pub fn load_chapter(viewport: &HtmlElement, raw_html: &str);
//
// It is responsible for:
//   • Parsing `raw_html`
//   • Inserting as much of it as fits inside `viewport` (height-wise)
//   • Cutting at a sensible boundary when content overflows
//   • Leaving the viewport unchanged when no cut point exists
//
use crate::renderer::{self, load_chapter}; // <-- change to your actual path, e.g. `super::load_chapter`

// ---------------------------------------------------------------------------
// Test 1 – Chapter fits entirely: no cut should occur
// ---------------------------------------------------------------------------
//
// Strategy: give the viewport a generous height and supply a short chapter.
// After loading we check that the very last semantic element of the chapter
// is still present inside the viewport (nothing was dropped).
// ---------------------------------------------------------------------------
#[wasm_bindgen_test]
fn test_chapter_fits_no_cut_needed() {
    // A tall viewport that can easily hold the short content below.
    let viewport = make_viewport(800, 2000);

    let raw_html = r#"<p id="p1">This is the first paragraph. It is short.</p><p id="p2">This is the second paragraph. Also short.</p><p id="last-paragraph">This is the final paragraph and it should still be visible.</p>"#;
    let res=load_chapter(&viewport, raw_html,0);

    assert!(res.is_some());
    assert_eq!(res.unwrap(),raw_html.len());

    let document = window().unwrap().document().unwrap();

    // The last paragraph must exist in the DOM …
    let last_p = document.get_element_by_id("last-paragraph");
    assert!(
        last_p.is_some(),
        "last-paragraph should be in the DOM when content fits"
    );

    // … and it must be within the viewport's height boundary.
    let viewport_height = viewport.get_bounding_client_rect().height();
    let bottom          = last_child_bottom_relative(&viewport);

    assert!(
        bottom <= viewport_height,
        "last element bottom ({bottom:.1}px) should be within viewport height ({viewport_height:.1}px)"
    );

    // Sanity: the viewport itself has the dimensions we asked for.
    let rect = viewport.get_bounding_client_rect();
    assert!(rect.width()  > 0.0, "viewport width  should be > 0");
    assert!(rect.height() > 0.0, "viewport height should be > 0");

    cleanup(&viewport);
}

// ---------------------------------------------------------------------------
// Test 2 – Chapter does not fit: a cut must occur
// ---------------------------------------------------------------------------
//
// Strategy: give the viewport a very small height and supply a long chapter
// made of many short paragraphs with IDs.  After loading we verify:
//   a) No rendered content extends beyond the viewport's bottom edge.
//   b) At least one paragraph from the *end* of the chapter is missing,
//      proving that the function actually cut rather than overflowed silently.
// ---------------------------------------------------------------------------
#[wasm_bindgen_test]
fn test_chapter_does_not_fit_cut_occurs() {
    // A deliberately small viewport – only ~80 px tall.
    let viewport = make_viewport(400, 80);

    // Build 20 paragraphs; only the first handful can possibly fit.
    let mut html = String::new();
    for i in 1..=200 {
        html.push_str(&format!(
            r#"<p id="para-{i}" style="font-size:16px;line-height:1.5;margin:0">Paragraph {i} with some text.</p>"#
        ));
    }

    assert!(load_chapter(&viewport, &html,0).is_some());

    // a) Nothing should overflow the viewport.
    let viewport_height = viewport.get_bounding_client_rect().height();
    let bottom          = last_child_bottom_relative(&viewport);

    assert!(
        bottom <= viewport_height + 1.0, // +1 px tolerance for sub-pixel rounding
        "rendered content bottom ({bottom:.1}px) must not exceed viewport height ({viewport_height:.1}px)"
    );

    // b) The last paragraph must be absent – if it were present the cut did
    //    not happen (or the viewport is too tall for this test to be meaningful).
    let document  = window().unwrap().document().unwrap();
    let last_para = document.get_element_by_id("para-20");
    assert!(
        last_para.is_none(),
        "para-200 should have been cut and must not be in the DOM"
    );

    cleanup(&viewport);
}

// ---------------------------------------------------------------------------
// Test 3 – Chapter cannot be cut: content is kept as-is
// ---------------------------------------------------------------------------
//
// Strategy: supply content that has no valid cut point.  We test two
// sub-cases that both represent "uncut-able" content:
//
//   3a. A single `<img>` element – an image cannot be split in the middle.
//   3b. A single run of continuous text with no spaces, punctuation, or child
//       elements – there is nowhere to insert a break.
//
// In both cases the function must leave the content intact (not discard it
// even though it overflows) and NOT crash.
//
// We confirm "left intact" by checking that the element / text node is still
// present in the viewport's DOM after the call.
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn test_chapter_cannot_be_cut_image() {
    // Small viewport so the image would overflow if cutting were tried.
    let viewport = make_viewport(200, 50);

    // A single image taller than the viewport.
    let raw_html = r#"<img id="big-image" src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" style="width:200px;height:300px;" alt="test">"#;

    load_chapter(&viewport, raw_html,0);

    let document = window().unwrap().document().unwrap();
    let img      = document.get_element_by_id("big-image");

    assert!(
        img.is_some(),
        "image must remain in the DOM because it cannot be cut"
    );

    cleanup(&viewport);
}

#[wasm_bindgen_test]
fn test_chapter_cannot_be_cut_continuous_text() {
    // Small viewport – content will overflow but has no cut point.
    let viewport = make_viewport(200, 40);

    // One paragraph of continuous text: no spaces, no punctuation, no child
    // elements – nowhere for the function to place a cut.
    let continuous = "A".repeat(500); // 500 consecutive letters
    let raw_html   = format!(r#"<p id="wall-of-text">{continuous}</p>"#);

    load_chapter(&viewport, &raw_html,0);

    let document = window().unwrap().document().unwrap();
    let wall     = document.get_element_by_id("wall-of-text");

    assert!(
        wall.is_some(),
        "continuous-text paragraph must remain in the DOM because it cannot be cut"
    );

    cleanup(&viewport);
}


use web_sys::console;
use wasm_bindgen::JsValue;

#[wasm_bindgen_test]
async fn test_problem_chapter_does_not_panic() {
    // Fetch the problem HTML file — path is relative to where wasm-pack serves files,
    // typically the crate root or a configured asset directory.
    let response = wasm_bindgen_futures::JsFuture::from(
        web_sys::window().unwrap().fetch_with_str("/assets/problem.html")
    ).await.unwrap();

    let response: web_sys::Response = response.dyn_into().unwrap();
    let text = wasm_bindgen_futures::JsFuture::from(response.text().unwrap())
        .await
        .unwrap();

    let html = text.as_string().unwrap();

    let viewport = make_viewport(800, 600);
    let result = load_chapter(&viewport, &html, 0);

    // It must not panic, and must return either Some(_) or None — both are valid.
    // The key assertion is just that we got here without unwinding.
    assert!(result.is_some());

    console::log_1(&JsValue::from_str(&format!("Result: {:?}, whole text length: {}", result, html.len())));


    match result {
        Some(n) => assert!(n <= html.len(), "cut position out of bounds: {n} > {}", html.len()),
        None    => { /* no cut point found — valid for this chapter */ }
    }

    cleanup(&viewport);
}


#[wasm_bindgen_test]
async fn test_problem_chapter_at_middle_does_not_panic() {
    // Fetch the problem HTML file — path is relative to where wasm-pack serves files,
    // typically the crate root or a configured asset directory.
    let response = wasm_bindgen_futures::JsFuture::from(
        web_sys::window().unwrap().fetch_with_str("/assets/problem.html")
    ).await.unwrap();

    let response: web_sys::Response = response.dyn_into().unwrap();
    let text = wasm_bindgen_futures::JsFuture::from(response.text().unwrap())
        .await
        .unwrap();

    let html = text.as_string().unwrap();


    let viewport = make_viewport(800, 600);
    let result = load_chapter(&viewport, &html, 3319);

    // It must not panic, and must return either Some(_) or None — both are valid.
    // The key assertion is just that we got here without unwinding.
    assert!(result.is_some());

    console::log_1(&JsValue::from_str(&format!("Result: {:?}, whole text length: {}", result, html.len())));


    match result {
        Some(n) => assert!(n <= html.len(), "cut position out of bounds: {n} > {}", html.len()),
        None    => { /* no cut point found — valid for this chapter */ }
    }

    cleanup(&viewport);
}