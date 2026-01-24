use dioxus::prelude::*;

use crate::{Route, domain, styles, ui::components::{Card, TopBar, TopBarEntry}};


#[component]
pub fn Series(series_id:String) -> Element {
    let title=use_signal(||"".to_owned());
    let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
    ];

    let books=domain::series::use_series(series_id.clone(),title);
    return rsx! {
        div { style: styles::CONTAINER_STYLE,
            TopBar{ entries: top_entries }
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
