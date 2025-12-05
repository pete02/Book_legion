use dioxus::prelude::*;

use crate::components::{book::BookRenderer, use_load_book};

#[component]
pub fn BookView()->Element{
    let time=use_signal(||0.0);
    let idle=use_signal(||false);
    let book=use_signal(||"mageling".to_owned());

    use_load_book(book(), time, idle);
    rsx!{
        div {
            style: "height: calc(100vh - 100px); display: flex; flex-direction: column;",
            h1 {"{book}"}
            BookRenderer { idle }
          }
        
    }
}