use tribune_logistica::{audio_handler::get_audio_chunk, db_handlers::load_books, models::BookStatus};


mod test;
//se tribune_logistica::server;
/*
#[tokio::main]
async fn main(){
    server().await
}

     */

fn main(){
    let b=load_books("./data/books.json").unwrap();
    let book=b.get("mageling").unwrap();
    let status=BookStatus::new("mageling", "./data", book.clone(), "books.json");

    println!("{:?}",get_audio_chunk(&status, status.initial_chapter as usize, 1, "./test.mp3", true, "./data"));

}