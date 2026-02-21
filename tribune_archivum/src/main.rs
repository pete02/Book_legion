use std::path::Path;

pub mod lib;
mod tests;
use lib::info_sender::get_series_title;
use quick_xml::se;


struct BookQuery<'a> {
    title: &'a str,
    author: &'a str,
    expected_series: Option<&'a str>, // Add expected series
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    run_converter();
    get_names().await;



}

async fn get_names(){
    let books=lib::info_sender::scan_epub_folder(Path::new("./out")).await.unwrap();
    println!("leN: {}", books.len());
    for data in books{
        println!("Title: {}, Author: {}, Series: {}, Pos: {}", data.title, data.author, data.series, data.pos)
    }
}

fn run_converter(){
    let input=Path::new("./data");
    let output=Path::new("./out");
    let errs=Path::new("./errs");


    let res=lib::orchestrator::process_library(input, output, errs);

    if res.is_err(){
        println!("error happened: {:?}",res);
    }
}

