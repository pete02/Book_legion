mod test_nested_layout{
    use crate::renderer::layout_builder;
    use std::alloc::Layout;

    const html_content: &str= r#"<p class="THis is a big test"><span class="class56">Hugh of Emblin</span></p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::{find_page_boundary::LayoutQuery, layout_builder::build_layout};

    fn setup()->(LayoutQuery, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(html_content);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout(&container, 0, html_content), container.outer_html())
    }

    

    #[wasm_bindgen_test]
    fn layout_parent_starts_at_zero(){
        let (layout,_)=setup();
        assert!(layout.char_start == 0, "first layout char_start should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_parent_does_not_have_text(){
        let (layout,_)=setup();
        assert!(layout.text.len() == 0, "first layout text length should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_text_length(){
        let (layout,text)=setup();
        assert!(layout.text_len() != text.len() as u32, "Layout text length should not match viewport html, {} != {}, text assumed: {}", layout.text_len(), text.len() as u32, text  );
        assert!(layout.text_len() != html_content.len() as u32, "Layout text length should match HTML content length, {} != {}, text assumed: {}", layout.text_len(), text.len() as u32, html_content  );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_children(){
        let (layout,_)=setup();
        assert!(layout.children.len() > 0, "must have children" );
        assert!(layout.children.len() == 1, "must have one child, had {}", layout.children.len() );
    }
    #[wasm_bindgen_test]
    fn layout_start_differs_from_child(){
        let (layout,_)=setup();
        assert!(layout.char_start != layout.children[0].char_start, "child should have advanced char_start, char starts: {}, {}", layout.char_start, layout.children[0].char_start );
    }

    #[wasm_bindgen_test]
    fn layout_child_start_at_correct_place(){
        let (layout,_)=setup();
        assert!(layout.children[0].char_start ==30, "child char_start should be 30, {}", layout.children[0].char_start );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_text(){
        let (layout,_)=setup();
        assert!(layout.children[0].text.len() > 0, "child should have text" );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_correct_length(){
        let (layout,_)=setup();
        assert!(layout.children[0].text_len() ==30+43, "child text_len should be 73, {}, for text: {}", layout.children[0].text_len(), layout.children[0].text );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_no_children(){
        let (layout,_)=setup();
        assert!(layout.children[0].children.len() == 0, "child should have no children, had {:?}, {}", layout.children[0], layout.children[0].text );
    }
}

