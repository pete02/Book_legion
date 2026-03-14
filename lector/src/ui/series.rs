use dioxus::prelude::*;

use crate::{Route, domain::{self, series::delete_series}, styles, ui::components::{Card, TopBar, TopBarEntry}};


#[component]
pub fn Series(series_id:String) -> Element {
    let title=use_signal(||"".to_owned());
    let mut delete_signal=Signal::new(false);
    let nav=use_navigator();
    let entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
    ];

    let books: Signal<Vec<domain::cover::CardData>>=domain::series::use_series(series_id.clone(),title);
    use_effect(move||{
        if books().len() == 0{
            delete_signal.set(true);
        }else{
            delete_signal.set(false);
        }

        if books().len()==1{
            let book=&books()[0];
            nav.push(book.path.clone());
        }
    });

    return rsx! {
        div { style: styles::CONTAINER_STYLE,
            TopBar { entries: entries, show_extra: use_signal(||true), text_extra: Some("Edit".to_string()), on_extra: Callback::new(move |_| {nav.push(Route::SeriesEdit{ series_id: series_id.clone() });}) }
            h1 { style: styles::HEADER_STYLE, "{title}" }

            div { 
                style: styles::GRID_STYLE,
                {
                    books.iter().map(|entry| rsx!( Card{ entry: entry.clone() } ))
                }
            }
        }
    }
}
