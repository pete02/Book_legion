use std::path::Path;

pub mod lib;
mod tests;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    let input=Path::new("./data2");
    let output=Path::new("./out");
    let onboarded=Path::new("./onboard");
    let errs=Path::new("./errs");


    let res=lib::orchestrator::process_library(input, output, errs, true);

    if res.is_err(){
        println!("error happened: {:?}",res);
    }
    let _=lib::info_sender::scan_epub_folder(output, onboarded).await;


}
