
mod parser;
mod epub_creator;
use epub_creator::*;
use crate::driver::*;

mod driver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configs=load_config("config.json")?;
    let epub=MyEpub::new("Warforged","J. L. Mullins")?;
    scrape_and_build_epub(&configs.royal_road,epub,"https://www.royalroad.com/fiction/47826/millennial-mage-a-slice-of-life-progression-fantasy", "12856")?;
    Ok(())

}
