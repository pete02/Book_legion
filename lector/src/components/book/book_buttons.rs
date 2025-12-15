use dioxus::prelude::*;

#[component]
pub fn BookButtons(mut move_page:Signal<i32>)->Element{
    rsx!(
       div {
        style: "
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 90vh;
            display: flex;
        ",
        id: "book-buttons",
            // Left half
            button {
                id: "backward",
                style: "
                    flex: 1 1 0;
                    cursor: pointer;
                    background: transparent;
                ",
                onclick: move |_| { move_page.set(-1); }
            },

            // Right half
            button {
                id: "forward",
                style: "
                    flex: 1 1 0;
                    cursor: pointer;
                    background: transparent;
                ",
                onclick: move |_| { move_page.set(1); }
            }
        }
    )
}