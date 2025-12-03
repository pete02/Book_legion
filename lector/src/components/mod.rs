
mod hero;
pub use hero::Hero;
mod load_book;
pub use load_book::use_load_book;
mod book_cover;
pub use book_cover::BookCover;

pub mod audio;

mod parse_book;
pub use parse_book::use_book_parsing;