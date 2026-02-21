use std::path::Path;

pub mod lib;
mod tests;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    run_converter();
    let _=lib::info_sender::scan_epub_folder(Path::new("./out"), Path::new("./onboard")).await;


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

