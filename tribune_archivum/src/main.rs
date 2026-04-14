use std::{error::Error, fs::{self, File}, io::Seek, path::Path};
use log::{ error, info};
use zip::ZipArchive;

use crate::lib::verifiers;

pub mod lib;
mod tests;


#[tokio::main]
async fn main() {
   println!("{:?}",test());
}


async fn run_main(){
    dotenvy::dotenv().ok();
    env_logger::init();

    let input = Path::new("/onboard");
    let output = Path::new("/out");
    let onboarded = Path::new("/data");
    let errs = Path::new("/errs");
    let copy = false;

    let shutdown = tokio::signal::ctrl_c();
    let mut shutdown = std::pin::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("shutdown signal received");
                break;
            }

            _ = async {
                if let Err(e) = lib::orchestrator::process_library(input, output, errs, copy) {
                    error!("error happened: {:?}", e);
                }

                if let Err(e) = lib::info_sender::scan_epub_folder(output, onboarded, errs).await {
                    error!("error in scanning: {}", e);
                }

                if !copy {
                    if let Err(e) = remove_empty_dirs(input) {
                        error!("failed to clean empty dirs: {:?}", e);
                    }
                }

                println!("begin waiting");
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            } => {}
        }
    }

    info!("exiting main loop");
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


fn test()->Result<(), Box<dyn Error>>{
    let path="./err.epub";
    println!("{:?}",verifiers::repair_epub_if_needed(path));


    println!("is ok");
    Ok(())
}