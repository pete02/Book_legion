use dioxus::prelude::*;

use crate::models::GlobalState;

pub fn use_book_parsing(book:Signal<String>){

    use_effect({
        let mut book=book.clone();
        let global=use_context::<Signal<GlobalState>>();
        move || {
            match global().book {
                None=>{},
                Some(b)=>{book.set(b.name);}
            }
        }
    });
}