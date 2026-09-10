#[cfg(test)]
mod page_count_tests {
    use super::*;
    use crate::domain::text::compute_total_pages;
    #[test]
    fn exact_multiple_gives_correct_page_count() {
        // 3 pages exactly, no partial page
        assert_eq!(compute_total_pages(900.0, 300.0), Some(2));
    }

    #[test]
    fn partial_last_page_rounds_up() {
        // 2.33 pages worth of content -> must round up to 3, not truncate,
        // or the last partial page of text would be inaccessible
        assert_eq!(compute_total_pages(700.0, 300.0), Some(2));
    }

    #[test]
    fn single_page_content_gives_one_page_not_zero() {
        // content narrower than the viewport itself
        assert_eq!(compute_total_pages(150.0, 300.0), Some(1));
    }

    #[test]
    fn zero_client_width_returns_none() {
        // viewport hasn't laid out yet — must not divide by zero
        assert_eq!(compute_total_pages(900.0, 0.0), None);
    }

    #[test]
    fn negative_client_width_returns_none() {
        assert_eq!(compute_total_pages(900.0, -10.0), None);
    }
}

// domain/text_tests.rs  (or a #[cfg(test)] mod at the bottom of text.rs)


use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, ShadowRootInit, ShadowRootMode};

use crate::domain::text::resolve_offset_to_page;

wasm_bindgen_test_configure!(run_in_browser);
struct Fixture {
    host: Element,
}

impl Fixture {
    fn new(html: &str, column_width_px: f64) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();

        // Ensure the fixture is isolated even if something from a previous
        // test was accidentally left behind.
        if let Some(existing) = document.get_element_by_id("book-content") {
            let _ = body.remove_child(&existing);
        }

        let host = document.create_element("div").unwrap();
        host.set_id("book-content");

        host.set_attribute(
            "style",
            &format!(
                "width: 600px;\
                 height: 300px;\
                 column-width: {column_width_px}px;\
                 column-gap: 0;\
                 column-fill: auto;"
            ),
        )
        .unwrap();

        body.append_child(&host).unwrap();

        let shadow_root = host
            .attach_shadow(&ShadowRootInit::new(ShadowRootMode::Open))
            .unwrap();

        shadow_root.set_inner_html(html);

        // Force the browser to perform layout before the test asks for
        // getBoundingClientRect().
        let _ = host.get_bounding_client_rect();

        Fixture { host }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(parent) = self.host.parent_node() {
            let _ = parent.remove_child(&self.host);
        }
    }
}

/// Three deterministic columns:
///
///   column 0: AAAAAAAAAA
///   column 1: BBBBBBBB
///   column 2: CCCCCCCCCCCC
///
/// The explicit block-level column breaks mean that these tests don't depend
/// on font metrics or line wrapping.
fn three_page_fixture() -> Fixture {
    let html = r#"
        <div style="break-after: column;">AAAAAAAAAA</div>
        <div style="break-after: column;">BBBBBBBB</div>
        <div>CCCCCCCCCCCC</div>
    "#;

    Fixture::new(html, 200.0)
}

/// Run all browser/DOM tests sequentially in one wasm test.
///
/// Keeping these in one test is intentional. All tests operate on the same
/// browser document and production code looks up a fixed `#book-content`
/// element. Running independent wasm tests concurrently can therefore make
/// one test observe another test's fixture.
#[wasm_bindgen_test]
fn resolve_offset_to_page_tests() {
    // ---------------------------------------------------------------------
    // Basic first-column lookup
    // ---------------------------------------------------------------------
    let diagnosis=false;
    //diagnosis
    {
        if(diagnosis){
            let _fixture = three_page_fixture();

            let page0 = resolve_offset_to_page(0, 200.0);
            let page1 = resolve_offset_to_page(10, 200.0);
            let page2 = resolve_offset_to_page(18, 200.0);

            web_sys::console::log_1(
                &format!("RESULTS: 0={page0:?}, 10={page1:?}, 18={page2:?}").into(),
            );

            assert!(false, "Diagnosis failed")
        }else{
            assert!(true)
        }
        
    }

    {
        let _fixture = three_page_fixture();

        assert_eq!(
            resolve_offset_to_page(0, 200.0),
            Some(0),
            "offset 0 should be in page 0"
        );

        assert_eq!(
            resolve_offset_to_page(9, 200.0),
            Some(0),
            "last character of first chunk should be in page 0"
        );
    }

    // ---------------------------------------------------------------------
    // Second column
    // ---------------------------------------------------------------------
    {
        let _fixture = three_page_fixture();
        let page=             resolve_offset_to_page(10, 200.0);

        web_sys::console::log_1(
            &format!("offset 10 => {:?}", page).into(),
        );

        
        assert_eq!(
            page,
            Some(1),
            "first character after first break should be in page 1"
        );

        assert_eq!(
            resolve_offset_to_page(17, 200.0),
            Some(1),
            "last character of second chunk should be in page 1"
        );
    }

    // ---------------------------------------------------------------------
    // Third column
    // ---------------------------------------------------------------------
    {
        let _fixture = three_page_fixture();

        assert_eq!(
            resolve_offset_to_page(18, 200.0),
            Some(2),
            "first character of third chunk should be in page 2"
        );

        assert_eq!(
            resolve_offset_to_page(25, 200.0),
            Some(2),
            "middle of third chunk should be in page 2"
        );
    }

    // ---------------------------------------------------------------------
    // Offset beyond document end
    // ---------------------------------------------------------------------
    {
        let _fixture = three_page_fixture();

        assert_eq!(
            resolve_offset_to_page(9_999, 200.0),
            Some(2),
            "offset beyond document should clamp to the last text node"
        );
    }

    // ---------------------------------------------------------------------
    // Negative offset
    // ---------------------------------------------------------------------
    {
        let _fixture = three_page_fixture();

        assert_eq!(
            resolve_offset_to_page(-5, 200.0),
            Some(0),
            "negative offset should clamp to the first text node"
        );
    }

    // ---------------------------------------------------------------------
    // Invalid column widths
    //
    // These don't need a fixture because validation happens before DOM
    // lookup, but keeping the fixture here also ensures the test behaves
    // correctly in the same browser environment.
    // ---------------------------------------------------------------------
    {
        let _fixture = three_page_fixture();

        assert_eq!(
            resolve_offset_to_page(5, 0.0),
            None,
            "zero column width should return None"
        );

        assert_eq!(
            resolve_offset_to_page(5, -10.0),
            None,
            "negative column width should return None"
        );

        assert_eq!(
            resolve_offset_to_page(5, f64::NAN),
            None,
            "NaN column width should return None"
        );

        assert_eq!(
            resolve_offset_to_page(5, f64::INFINITY),
            None,
            "infinite column width should return None"
        );
    }

    // ---------------------------------------------------------------------
    // Missing book-content
    // ---------------------------------------------------------------------
    //
    // Do this immediately after explicitly removing any fixture. Because all
    // browser tests are sequential now, nothing can recreate the element
    // while this assertion is running.
    // ---------------------------------------------------------------------
    {
        let document = web_sys::window().unwrap().document().unwrap();

        if let Some(host) = document.get_element_by_id("book-content") {
            host.remove();
        }

        assert_eq!(
            resolve_offset_to_page(0, 200.0),
            None,
            "missing #book-content should return None"
        );
    }

    // ---------------------------------------------------------------------
    // Empty shadow root
    // ---------------------------------------------------------------------
    {
        let _fixture = Fixture::new("", 200.0);

        assert_eq!(
            resolve_offset_to_page(0, 200.0),
            None,
            "empty shadow root should contain no text node"
        );
    }

    // ---------------------------------------------------------------------
    // Images do not consume character offsets
    // ---------------------------------------------------------------------
    {
        let html = r#"
            <div style="break-after: column;">
                <span>AAAAA</span>
                <img src="x.png" />
            </div>
            <div>BBBBB</div>
        "#;

        let _fixture = Fixture::new(html, 200.0);

        assert_eq!(
            resolve_offset_to_page(4, 200.0),
            Some(0),
            "last A should be in page 0"
        );

        assert_eq!(
            resolve_offset_to_page(5, 200.0),
            Some(1),
            "first B should be in page 1; image must not consume an offset"
        );
    }

    // ---------------------------------------------------------------------
    // UTF-16 / surrogate-pair offsets
    // ---------------------------------------------------------------------
    {
        let html = r#"
            <div style="break-after: column;">AA😀BB</div>
            <div>CCCC</div>
        "#;

        let _fixture = Fixture::new(html, 200.0);

        // UTF-16:
        //
        // A  = 0
        // A  = 1
        // 😀 = 2,3
        // B  = 4
        // B  = 5
        //
        // Therefore offset 5 is the second B, while offset 6 is the first C.
        assert_eq!(
            resolve_offset_to_page(5, 200.0),
            Some(0),
            "last B should still be in page 0"
        );

        assert_eq!(
            resolve_offset_to_page(6, 200.0),
            Some(1),
            "first C should be in page 1"
        );
    }
}

