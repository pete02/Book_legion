use dioxus::prelude::*;

use crate::models::GlobalState;

pub fn load_name(mut book:Signal<String>){
    let global = use_context::<Signal<GlobalState>>();
    use_effect(move || {
        let Some(name)=global().name else {return;};
        if book().len() > 0 {return;};
        book.set(name);
    });

}