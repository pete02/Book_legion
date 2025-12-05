use dioxus::prelude::*;

use crate::components::{book::BookRenderer, use_load_book};

#[component]
pub fn BookView()->Element{
    let time=use_signal(||0.0);
    let idle=use_signal(||false);

    use_load_book("mageling".to_string(), time, idle);
    rsx!{
        h1 {" reader view"}
        BookRenderer { idle }
    }
}