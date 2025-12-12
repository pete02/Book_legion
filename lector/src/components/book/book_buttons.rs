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
                height: 100%;
                display: flex;
            ",

            // Left button
            div {
                style: "
                    width: 50%;
                    height: 100%;
                    cursor: pointer;
                    background: transparent;
                ",
                onclick: move |_| {move_page.set(-1);}
            }

            // Right button
            div {
                style: "
                    width: 50%;
                    height: 100%;
                    cursor: pointer;
                    background: transparent;
                ",
                onclick: move |_| {move_page.set(1);}
            }
            }
    )
}