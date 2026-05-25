use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

mod test_no_layout{
        const HTML_CONTENT: &str= r#""#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::{layout_builder::LayoutQuery, layout_builder::build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }
    #[wasm_bindgen_test]
    fn layout_vec_returns_only_no_element(){
        let (layout,_)=setup();
        assert!(layout.len() == 0, "should only have no elements" );
    }
}

mod test_non_nested_layout{
    const HTML_CONTENT: &str= r#"<p class="THis is a big test">Hugh of Emblin</p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::{layout_builder::LayoutQuery, layout_builder::build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }
    #[wasm_bindgen_test]
    fn layout_vec_returns_only_one_element(){
        let (layout,_)=setup();
        assert!(layout.len() == 1, "should only have one element" );
    }

    #[wasm_bindgen_test]
    fn layout_starts_at_zero(){
        let (layout,_)=setup();
        assert!(layout[0].char_start == 0, "first layout char_start should be 0" );
    }
    #[wasm_bindgen_test]
    fn layout_shoud_have_nop_children(){
        let (layout,_)=setup();
        assert!(layout[0].children.len() == 0, "first layout should have no children" );
    }
    #[wasm_bindgen_test]
    fn layout_shoud_have_text(){
        let (layout,_)=setup();
        assert!(layout[0].text.len() > 0, "first layout should have text" );
    }
    #[wasm_bindgen_test]
    fn layout_should_have_correct_len(){
        let (layout,_)=setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, "Layout text length should match text length, {} != {}", layout[0].text_len(), HTML_CONTENT.len() as u32 );
    }

}

mod test_single_nested_layout_singe_child{
    const HTML_CONTENT: &str= r#"<p class="THis is a big test"><span class="class56">Hugh of Emblin</span></p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_parent_should_be_only_element(){
        let (layout,_)=setup();
        assert!(layout.len() == 1, "should only have one element" );
    }

    #[wasm_bindgen_test]
    fn layout_parent_starts_at_zero(){
        let (layout,_)=setup();
        assert!(layout[0].char_start == 0, "first layout char_start should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_parent_does_not_have_text(){
        let (layout,_)=setup();
        assert!(layout[0].text.len() == 0, "first layout text length should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_text_length(){
        let (layout,text)=setup();
        assert!(layout[0].text_len() != text.len() as u32, "Layout text length should not match viewport html, {} != {}, text assumed: {}", layout[0].text_len(), text.len() as u32, text  );
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, "Layout text length should match HTML content length, {} != {}, text assumed: {}", layout[0].text_len(), HTML_CONTENT.len() as u32, HTML_CONTENT  );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_children(){
        let (layout,_)=setup();
        assert!(layout[0].children.len() > 0, "must have children" );
        assert!(layout[0].children.len() == 1, "must have one child, had {}", layout[0].children.len() );
    }
    #[wasm_bindgen_test]
    fn layout_start_differs_from_child(){
        let (layout,_)=setup();
        assert!(layout[0].char_start != layout[0].children[0].char_start, "child should have advanced char_start, char starts: {}, {}", layout[0].char_start, layout[0].children[0].char_start );
    }

    #[wasm_bindgen_test]
    fn layout_child_start_at_correct_place(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].char_start ==30, "child char_start should be 30, {}", layout[0].children[0].char_start );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_text(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].text.len() > 0, "child should have text" );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_correct_length(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].text_len() ==layout[0].children[0].text.len() as u32,
         "child text_len should be {}, with start {}, got {}, for text: {}", 
            layout[0].children[0].char_start+layout[0].children[0].text.len() as u32,
            layout[0].children[0].char_start, 
            layout[0].children[0].text_len(),
            layout[0].children[0].text 
        );
    }

    #[wasm_bindgen_test]
    fn layout_child_has_no_children(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].children.len() == 0, "child should have no children, had {:?}, {}", layout[0].children[0], layout[0].children[0].text );
    }
}


mod test_non_nested_layout_two_elements{
    const HTML_CONTENT: &str= r#"<p class="THis is a big test">Hugh of Emblin</p><p class="THis is a big test 2">was an interesing char</p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::{layout_builder::LayoutQuery, layout_builder::build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }
    #[wasm_bindgen_test]
    fn first_layout_vec_returns_only_one_element(){
        let (layout,_)=setup();
        assert!(layout.len() == 2, "should only have two elements" );
    }

    #[wasm_bindgen_test]
    fn first_layout_starts_at_zero(){
        let (layout,_)=setup();
        assert!(layout[0].char_start == 0, "first layout char_start should be 0" );
    }
    #[wasm_bindgen_test]
    fn first_layout_shoud_have_nop_children(){
        let (layout,_)=setup();
        assert!(layout[0].children.len() == 0, "first layout should have no children" );
    }
    #[wasm_bindgen_test]
    fn first_layout_shoud_have_text(){
        let (layout,_)=setup();
        assert!(layout[0].text.len() > 0, "first layout should have text" );
    }
    #[wasm_bindgen_test]
    fn first_layout_should_have_correct_len(){
        let (layout,_)=setup();
        assert!(layout[0].text_len() == layout[0].text.len() as u32, "Layout text length should match text length, {} != {}", layout[0].text_len(), HTML_CONTENT.len() as u32 );
    }

    #[wasm_bindgen_test]
    fn second_layout_starts_at_first_layout_len(){
        let (layout,_)=setup();
        assert!(layout[1].char_start == layout[0].text_len(), "second layout char_start should be at the end of the first layout" );
    }
    #[wasm_bindgen_test]
    fn second_layout_shoud_have_nop_children(){
        let (layout,_)=setup();
        assert!(layout[1].children.len() == 0, "second layout should have no children" );
    }
    #[wasm_bindgen_test]
    fn second_layout_shoud_have_text(){
        let (layout,_)=setup();
        assert!(layout[1].text.len() > 0, "second layout should have text" );
    }
    #[wasm_bindgen_test]
    fn second_layout_should_have_correct_len(){
        let (layout,_)=setup();
        assert!(layout[1].text_len() == layout[1].text.len() as u32, "Layout text length should match text length, {} != {}", layout[1].text_len(), layout[1].text.len() as u32 );
    }
    #[wasm_bindgen_test]
    fn second_layout_len_and_start_shoud_equal_html(){
        let (layout,_)=setup();
        assert!(layout[1].text_len()+layout[1].char_start == HTML_CONTENT.len() as u32, "Layout text length should match text length, {} != {}", layout[1].text_len()+layout[1].char_start, HTML_CONTENT.len() as u32 );
    }
}


mod test_single_nested_layout_two_children{
    const HTML_CONTENT: &str= r#"<p class="THis is a big test"><span class="class56">Hugh of Emblin</span><span>was very interested</span></p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        // Create a container and inject the problem.html content
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        
        // HTML content from problem.html (truncated for test)
        
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_parent_should_be_only_element(){
        let (layout,_)=setup();
        assert!(layout.len() == 1, "should only have one element" );
    }

    #[wasm_bindgen_test]
    fn layout_parent_starts_at_zero(){
        let (layout,_)=setup();
        assert!(layout[0].char_start == 0, "first layout char_start should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_parent_does_not_have_text(){
        let (layout,_)=setup();
        assert!(layout[0].text.len() == 0, "first layout text length should be 0" );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_text_length(){
        let (layout,text)=setup();
        assert!(layout[0].text_len() != text.len() as u32, "Layout text length should not match viewport html, {} != {}, text assumed: {}", layout[0].text_len(), text.len() as u32, text  );
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, "Layout text length should match HTML content length, {} != {}, text assumed: {}", layout[0].text_len(), HTML_CONTENT.len() as u32, HTML_CONTENT  );
    }

    #[wasm_bindgen_test]
    fn layout_has_correct_children(){
        let (layout,_)=setup();
        assert!(layout[0].children.len() > 0, "must have children" );
        assert!(layout[0].children.len() == 2, "must have two children, had {}", layout[0].children.len() );
    }
    #[wasm_bindgen_test]
    fn layout_start_differs_from_child(){
        let (layout,_)=setup();
        assert!(layout[0].char_start != layout[0].children[0].char_start, "child should have advanced char_start, char starts: {}, {}", layout[0].char_start, layout[0].children[0].char_start );
        assert!(layout[0].char_start != layout[0].children[1].char_start, "child should have advanced char_start, char starts: {}, {}", layout[0].char_start, layout[0].children[1].char_start );
    }


    #[wasm_bindgen_test]
    fn first_layout_child_start_at_correct_place(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].char_start ==30, "child char_start should be 30, {}", layout[0].children[0].char_start );
    }

    #[wasm_bindgen_test]
    fn first_layout_child_has_text(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].text.len() > 0, "child should have text" );
    }

    #[wasm_bindgen_test]
    fn first_layout_child_has_correct_length(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].text_len() ==layout[0].children[0].text.len() as u32,
         "child text_len should be {}, with start {}, got {}, for text: {}", 
            layout[0].children[0].char_start+layout[0].children[0].text.len() as u32,
            layout[0].children[0].char_start, 
            layout[0].children[0].text_len(),
            layout[0].children[0].text 
        );
    }

    #[wasm_bindgen_test]
    fn first_layout_child_has_no_children(){
        let (layout,_)=setup();
        assert!(layout[0].children[0].children.len() == 0, "child should have no children, had {:?}, {}", layout[0].children[0], layout[0].children[0].text );
    }


    #[wasm_bindgen_test]
    fn second_layout_child_start_at_correct_place(){
        let (layout,_)=setup();
        assert!(layout[0].children[1].char_start ==layout[0].children[0].char_start+layout[0].children[0].text.len() as u32, "child char_start should be 30, {}", layout[1].children[1].char_start );
    }

    #[wasm_bindgen_test]
    fn second_layout_child_has_text(){
        let (layout,_)=setup();
        assert!(layout[0].children[1].text.len() > 0, "child should have text" );
    }

    #[wasm_bindgen_test]
    fn second_layout_child_has_correct_length(){
        let (layout,_)=setup();
        assert!(layout[0].children[1].text_len() ==layout[0].children[1].text.len() as u32,
         "child text_len should be {}, with start {}, got {}, for text: {}", 
            layout[0].children[1].char_start+layout[0].children[1].text.len() as u32,
            layout[0].children[1].char_start, 
            layout[0].children[1].text_len(),
            layout[0].children[1].text 
        );
    }

    #[wasm_bindgen_test]
    fn second_layout_child_has_no_children(){
        let (layout,_)=setup();
        assert!(layout[0].children[1].children.len() == 0, "child should have no children, had {:?}, {}", layout[0].children[1], layout[0].children[1].text );
    }
}


mod test_grandparent_one_child_one_grandchild{
    const HTML_CONTENT: &str= r#"<div class="grandparent"><p class="child"><span class="grandchild">Deep text</span></p></div>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_has_one_root_element(){
        let (layout,_) = setup();
        assert!(layout.len() == 1, "should have one root element" );
    }

    #[wasm_bindgen_test]
    fn root_is_grandparent(){
        let (layout,_) = setup();
        assert!(layout[0].children.len() == 1, "grandparent should have one child (parent)" );
    }

    #[wasm_bindgen_test]
    fn grandparent_has_no_text(){
        let (layout,_) = setup();
        assert!(layout[0].text.is_empty(), "grandparent should have no direct text" );
    }

    #[wasm_bindgen_test]
    fn child_is_parent(){
        let (layout,_) = setup();
        let child = &layout[0].children[0];
        assert!(child.children.len() == 1, "parent should have one child (grandchild)" );
    }

    #[wasm_bindgen_test]
    fn grandchild_has_text(){
        let (layout,_) = setup();
        let grandchild = &layout[0].children[0].children[0];
        assert!(grandchild.text.len() > 0, "grandchild should have text" );
    }

    #[wasm_bindgen_test]
    fn grandchild_has_no_children(){
        let (layout,_) = setup();
        let grandchild = &layout[0].children[0].children[0];
        assert!(grandchild.children.len() == 0, "grandchild should have no children" );
    }

    #[wasm_bindgen_test]
    fn total_text_len_matches_html(){
        let (layout,_) = setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, 
            "Layout text length should match HTML content length, {} != {}", 
            layout[0].text_len(), HTML_CONTENT.len() as u32);
    }

    #[wasm_bindgen_test]
    fn char_start_chain_is_correct(){
        let (layout,_) = setup();
        let grandchild = &layout[0].children[0].children[0];
        assert!(grandchild.char_start == 42, "grandchild char_start should be 42, was {}", grandchild.char_start );
    }
}

mod test_grandparent_one_child_two_grandchildren{
    const HTML_CONTENT: &str= r#"<div class="grandparent"><p class="child"><span class="grandchild1">Text 1</span><span class="grandchild2">Text 2</span></p></div>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_has_one_root_element(){
        let (layout,_) = setup();
        assert!(layout.len() == 1, "should have one root element" );
    }

    #[wasm_bindgen_test]
    fn grandparent_has_one_child(){
        let (layout,_) = setup();
        assert!(layout[0].children.len() == 1, "grandparent should have one child" );
    }

    #[wasm_bindgen_test]
    fn child_has_two_grandchildren(){
        let (layout,_) = setup();
        let child = &layout[0].children[0];
        assert!(child.children.len() == 2, "child should have two grandchildren, had {}", child.children.len() );
    }

    #[wasm_bindgen_test]
    fn first_grandchild_has_correct_start(){
        let (layout,_) = setup();
        let child = &layout[0].children[0];
        let gc1 = &child.children[0];
        assert!(gc1.char_start > layout[0].char_start, "first grandchild should have advanced char_start" );
    }

    #[wasm_bindgen_test]
    fn second_grandchild_starts_after_first(){
        let (layout,_) = setup();
        let child = &layout[0].children[0];
        let gc1 = &child.children[0];
        let gc2 = &child.children[1];
        assert!(gc2.char_start >= gc1.char_start + gc1.text.len() as u32, "second grandchild should start after first grandchild ends" );
    }

    #[wasm_bindgen_test]
    fn both_grandchildren_have_text(){
        let (layout,_) = setup();
        let child = &layout[0].children[0];
        assert!(child.children[0].text.len() > 0, "first grandchild should have text" );
        assert!(child.children[1].text.len() > 0, "second grandchild should have text" );
    }

    #[wasm_bindgen_test]
    fn total_text_len_matches_html(){
        let (layout,_) = setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, 
            "Layout text length should match HTML content length, {} != {}", 
            layout[0].text_len(), HTML_CONTENT.len() as u32);
    }
}

mod test_grandparent_two_children_one_grandchild_each{
    const HTML_CONTENT: &str= r#"<div class="grandparent"><p class="child1"><span class="grandchild1a">Text 1A</span></p><p class="child2"><span class="grandchild2a">Text 2A</span></p></div>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_has_one_root_element(){
        let (layout,_) = setup();
        assert!(layout.len() == 1, "should have one root element" );
    }

    #[wasm_bindgen_test]
    fn grandparent_has_two_children(){
        let (layout,_) = setup();
        assert!(layout[0].children.len() == 2, "grandparent should have two children, had {}", layout[0].children.len() );
    }

    #[wasm_bindgen_test]
    fn first_child_has_one_grandchild(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        assert!(child1.children.len() == 1, "first child should have one grandchild" );
    }

    #[wasm_bindgen_test]
    fn second_child_has_one_grandchild(){
        let (layout,_) = setup();
        let child2 = &layout[0].children[1];
        assert!(child2.children.len() == 1, "second child should have one grandchild" );
    }

    #[wasm_bindgen_test]
    fn second_child_starts_after_first(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        let child2 = &layout[0].children[1];
        assert!(child2.char_start >= child1.char_start + child1.text.len() as u32, "second child should start after first child ends" );
    }

    #[wasm_bindgen_test]
    fn all_grandchildren_have_text(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        let child2 = &layout[0].children[1];
        assert!(child1.children[0].text.len() > 0, "first grandchild should have text" );
        assert!(child2.children[0].text.len() > 0, "second grandchild should have text" );
    }

    #[wasm_bindgen_test]
    fn total_text_len_matches_html(){
        let (layout,_) = setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, 
            "Layout text length should match HTML content length, {} != {}", 
            layout[0].text_len(), HTML_CONTENT.len() as u32);
    }
}

mod test_grandparent_two_children_two_grandchildren_each{
    const HTML_CONTENT: &str= r#"<div class="grandparent"><p class="child1"><span class="grandchild1a">Text 1A</span><span class="grandchild1b">Text 1B</span></p><p class="child2"><span class="grandchild2a">Text 2A</span><span class="grandchild2b">Text 2B</span></p></div>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn layout_has_one_root_element(){
        let (layout,_) = setup();
        assert!(layout.len() == 1, "should have one root element" );
    }

    #[wasm_bindgen_test]
    fn grandparent_has_two_children(){
        let (layout,_) = setup();
        assert!(layout[0].children.len() == 2, "grandparent should have two children, had {}", layout[0].children.len() );
    }

    #[wasm_bindgen_test]
    fn first_child_has_two_grandchildren(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        assert!(child1.children.len() == 2, "first child should have two grandchildren, had {}", child1.children.len() );
    }

    #[wasm_bindgen_test]
    fn second_child_has_two_grandchildren(){
        let (layout,_) = setup();
        let child2 = &layout[0].children[1];
        assert!(child2.children.len() == 2, "second child should have two grandchildren, had {}", child2.children.len() );
    }

    #[wasm_bindgen_test]
    fn second_child_starts_after_first(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        let child2 = &layout[0].children[1];
        assert!(child2.char_start >= child1.char_start + child1.text.len() as u32, 
            "second child should start after first child ends" );
    }

    #[wasm_bindgen_test]
    fn all_four_grandchildren_have_text(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        let child2 = &layout[0].children[1];
        assert!(child1.children[0].text.len() > 0, "grandchild 1a should have text" );
        assert!(child1.children[1].text.len() > 0, "grandchild 1b should have text" );
        assert!(child2.children[0].text.len() > 0, "grandchild 2a should have text" );
        assert!(child2.children[1].text.len() > 0, "grandchild 2b should have text" );
    }

    #[wasm_bindgen_test]
    fn first_child_grandchildren_are_consecutive(){
        let (layout,_) = setup();
        let child1 = &layout[0].children[0];
        let gc1 = &child1.children[0];
        let gc2 = &child1.children[1];
        assert!(gc2.char_start >= gc1.char_start + gc1.text.len() as u32, 
            "second grandchild of first child should start after first ends" );
    }

    #[wasm_bindgen_test]
    fn second_child_grandchildren_are_consecutive(){
        let (layout,_) = setup();
        let child2 = &layout[0].children[1];
        let gc1 = &child2.children[0];
        let gc2 = &child2.children[1];
        assert!(gc2.char_start >= gc1.char_start + gc1.text.len() as u32, 
            "second grandchild of second child should start after first ends" );
    }

    #[wasm_bindgen_test]
    fn total_text_len_matches_html(){
        let (layout,_) = setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, 
            "Layout text length should match HTML content length, {} != {}", 
            layout[0].text_len(), HTML_CONTENT.len() as u32);
    }
}


mod test_mixed_content{
    const HTML_CONTENT: &str= r#"<p class="THis is a big test">Hugh of Emblin<span>was an interesing char</span></p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn mixed_content_layout(){
        let (layout, _) = setup();
        assert!(layout.len() == 1, "Expected 1 layout item for mixed content, {}" , layout.len() );
    }

    #[wasm_bindgen_test]
    fn mixed_content_has_two_children(){
        let (layout, _) = setup();
        assert!(layout[0].children.len() == 2, "Expected 2 children for mixed content, {}" , layout[0].children.len() );
    }

    #[wasm_bindgen_test]
    fn mixed_content_has_correct_len(){
        let (layout, _) = setup();
        assert!(layout[0].text_len() == HTML_CONTENT.len() as u32, "Expected text length for mixed content, {} != {}", layout[0].text_len(), HTML_CONTENT.len() as u32);
    }

    #[wasm_bindgen_test]
    fn mixed_content_first_child_has_text(){
        let (layout, _) = setup();
        assert!(layout[0].children[0].text.len() > 0, "First child should have text, {}" , layout[0].children[0].text.len() );
    }

    #[wasm_bindgen_test]
    fn mixed_content_second_child_has_text(){
        let (layout, _) = setup();
        assert!(layout[0].children[1].text.len() > 0, "Second child should have text, {}" , layout[0].children[1].text.len() );
    }

    #[wasm_bindgen_test]
    fn mixed_content_first_child_has_no_children(){
        let (layout, _) = setup();
        assert!(layout[0].children[0].children.is_empty(), "First child should have no children, {}" , layout[0].children[0].children.len() );
    }
    #[wasm_bindgen_test]
    fn mixed_content_second_child_has_no_children(){
        let (layout, _) = setup();
        assert!(layout[0].children[1].children.is_empty(), "Second child should have no children, {}" , layout[0].children[1].children.len() );
    }
    #[wasm_bindgen_test]
    fn mixed_content_first_child_has_correct_len(){
        let (layout, _) = setup();
        assert!(layout[0].children[0].text.len() == 14, "First child should have correct text length, {}" , layout[0].children[0].text.len() );
    }
    #[wasm_bindgen_test]
    fn mixed_content_second_child_has_correct_len(){
        let (layout, _) = setup();
        assert!(layout[0].children[1].text.len() == 35, "Second child should have correct text length, {}" , layout[0].children[1].text.len() );
    }
    #[wasm_bindgen_test]
    fn mixed_content_first_child_has_correct_start(){
        let (layout, _) = setup();
        assert!(layout[0].children[0].char_start == 30, "First child should have correct start position, {} != {}", layout[0].children[0].char_start, 30);
    }
    #[wasm_bindgen_test]
    fn mixed_content_second_child_has_correct_start(){
        let (layout, _) = setup();
        assert!(layout[0].children[1].char_start == 44, "Second child should have correct start position, {} != {}", layout[0].children[1].char_start, 44);
    }
}



mod test_self_closing_tags{
    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};


    const HTML_CONTENT: &str= r#"<p>Text<br>More text</p>"#;
    // Test that <br> doesn't break the builder
    

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn self_closing_tag_does_not_break_builder(){
        let (layout, _) = setup();
        assert!(layout.len() == 1, "Should have one layout item, {}" , layout.len() );
    }

    #[wasm_bindgen_test]
    fn parent_has_three_children(){
        let (layout, _) = setup();
        assert!(layout[0].children.len() == 3, 
            "Parent should have 3 children (2 text + 1 br), {}" , layout[0].children.len() );
    }

    #[wasm_bindgen_test]
    fn br_element_is_empty(){
        let (layout, _) = setup();
        let br = &layout[0].children[1];
        assert!(br.text=="<br>", "Self-closing tag should have <br>, {}" , br.text );
    }
}


mod test_text_between_elements{
    const HTML_CONTENT: &str= r#"<p>Text 1</p>Text 2<p>Text 3</p>"#;

    use wasm_bindgen_test::*;
    use web_sys::{window, HtmlElement};
    use wasm_bindgen::JsCast;
    use crate::renderer::layout_builder::{LayoutQuery, build_layout_vec};

    fn setup()->(Vec<LayoutQuery>, String){
        let document = window().unwrap().document().unwrap();
        
        let container_el = document.create_element("div").unwrap();
        let container: &HtmlElement = container_el.dyn_ref().unwrap();
        container.style().set_property("font-size", "16px").unwrap();
        container.set_inner_html(HTML_CONTENT);
        document.body().unwrap().append_child(&container_el).unwrap();
        
        (build_layout_vec(&container, HTML_CONTENT), container.outer_html())
    }

    #[wasm_bindgen_test]
    fn produces_three_layouts(){
        // build_layout_vec only creates layouts for element children, not text nodes
        let (layout, _) = setup();
        assert_eq!(layout.len(), 3, "Should have 3 layouts (2 <p> elements + 1 text node), got {}", layout.len() );
    }

    #[wasm_bindgen_test]
    fn first_layout_is_first_p(){
        let (layout, _) = setup();
        assert!(layout[0].text.len() > 0, "First layout should have text" );
        assert!(layout[0].children.is_empty(), "First layout should have no children" );
    }

    #[wasm_bindgen_test]
    fn second_layout_is_text(){
        let (layout, _) = setup();
        assert!(layout[1].text.len() > 0, "Second layout should have text" );
        assert!(layout[1].children.is_empty(), "Second layout should have no children" );
}
    #[wasm_bindgen_test]
    fn second_layout_is_second_p(){
        let (layout, _) = setup();
        assert!(layout[1].text.len() > 0, "Second layout should have text" );
        assert!(layout[1].children.is_empty(), "Second layout should have no children" );
    }


    #[wasm_bindgen_test]
    fn first_layout_starts_at_zero(){
        let (layout, _) = setup();
        assert_eq!(layout[0].char_start, 0, "First layout should start at 0" );
    }

    #[wasm_bindgen_test]
    fn second_layout_starts_after_first(){
        let (layout, _) = setup();
        let first_end = layout[0].char_start + layout[0].text.len() as u32;
        assert!(layout[1].char_start >= first_end, 
            "Second layout should start after first ends, {} >= {}", 
            layout[1].char_start, first_end);
    }

    #[wasm_bindgen_test]
    fn total_text_len_matches_html(){
        let (layout, _) = setup();
        assert_eq!(layout[2].char_start + layout[2].text_len(), 
            HTML_CONTENT.len() as u32, 
            "Total text length should match HTML content length, {} != {}", 
            layout[0].text_len() + layout[1].text_len(), HTML_CONTENT.len() as u32);
    }

    #[wasm_bindgen_test]
    fn first_layout_text_is_text_1(){
        let (layout, _) = setup();
        assert!(layout[0].text.contains("Text 1"), 
            "First layout text should contain 'Text 1', got: {}", layout[0].text);
    }

    #[wasm_bindgen_test]
    fn second_layout_text_is_text_2(){
        let (layout, _) = setup();
        assert!(layout[1].text.contains("Text 2"), 
            "Second layout text should contain 'Text 2', got: {}", layout[1].text);
    }
    #[wasm_bindgen_test]
    fn third_layout_text_is_text_3(){
        let (layout, _) = setup();
        assert!(layout[2].text.contains("Text 3"), 
            "Third layout text should contain 'Text 3', got: {}", layout[2].text);
    }
}