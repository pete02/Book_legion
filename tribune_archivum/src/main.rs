#![warn(clippy::cognitive_complexity)]

mod validator;
//mod generate_broken_zips;

fn main() {
    let chec=validator::verify_ebook("./data/A Practical Guide to Evil II (Verburg, David) (z-library.sk, 1lib.sk, z-lib.sk).epub", "content.opf");
    println!("{:?}",chec)
}
