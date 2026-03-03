use std::{fs, path::Path};
use log::{debug, error, info};

pub mod lib;
mod tests;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    let input = Path::new("/data");
    let output = Path::new("/out");
    let onboarded = Path::new("/onboard");
    let errs = Path::new("/errs");
    let copy=false;
    loop {
        let res = lib::orchestrator::process_library(input, output, errs, copy);

        if let Err(e) = res {
            error!("error happened: {:?}", e);
        }

        let _ = lib::info_sender::scan_epub_folder(output, onboarded, errs).await;

        if !copy{
            if let Err(e) = remove_empty_dirs(input) {
                error!("failed to clean empty dirs: {:?}", e);
            }
        }
        println!("beguin waiting");
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }

}

pub fn remove_empty_dirs(root: &Path) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            remove_empty_dirs(&path)?;

            if fs::read_dir(&path)?.next().is_none() {
                fs::remove_dir(&path)?;
            }
        }
    }

    Ok(())
}