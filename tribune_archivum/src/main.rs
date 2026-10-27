#![warn(clippy::cognitive_complexity)]

mod validator;
//mod generate_broken_zips;

fn main() {
    let chec=validator::verify_ebook("./target/broken_epubs/valid.epub", "OEBPS/content.opf");
    println!("{:?}",chec)
}
