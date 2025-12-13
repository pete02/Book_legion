
use dioxus::{prelude::*};
#[component]
pub fn BookRenderer(visible_chunks:Signal<Vec<String>>) -> Element {
    rsx!(
        div {
            id: "book-renderer",
            style: "
                position: relative;
                width: 100vw;
                box-sizing: border-box;
                overflow-wrap: break-word;
                word-wrap: break-word;
                overflow-x: hidden;
                padding: 1rem;
            ",
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
        }
    )
}

