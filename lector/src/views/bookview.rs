use dioxus::prelude::*;

use crate::components::{book::{BookRenderer, use_css_injector}, global_updater, use_load_book};

#[component]
pub fn BookView()->Element{
    let time=use_signal(||0.0);
    let idle=use_signal(||false);
    let css_idle=use_signal(||true);
    let book=use_signal(||"fusing".to_owned());

    use_load_book(book(), time, idle);
    use_css_injector(idle, css_idle);
    global_updater();
    rsx! {
        h1 {
            "{book}",
        }
        BookRenderer { idle, css_idle }
    }
 }