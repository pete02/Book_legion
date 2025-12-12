use std::vec;

use dioxus::{prelude::*};
#[component]
pub fn BookRenderer(visible_chunks:Signal<Vec<String>>) -> Element {
    rsx!(
        {
                visible_chunks.iter().map(|chunk| rsx!(
                    div {
                        class: "chapter-chunk mb-2",
                        style: "
                            width: 100%;
                            box-sizing: border-box;
                            word-break: break-word;
                        ",
                        dangerous_inner_html: "{chunk}",
                    }
                ))
            }
    )
}

