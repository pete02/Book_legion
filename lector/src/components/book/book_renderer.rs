
use dioxus::{prelude::*};
#[component]
pub fn BookRenderer(visible_chunks:Signal<Vec<String>>) -> Element {
    rsx!(
        div {
            id: "book-renderer",
            style: "
                    position: static;
                    display: block;

                    width: 100%;
                    max-width: 100vw;

                    height: auto;
                    min-height: 0;

                    overflow: visible;

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
                        display: block;
                        width: 100%;
                        height: auto;
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

