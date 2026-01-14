use tribune_archivum::check_epub;

fn main()-> Result<(),Box<dyn std::error::Error>> {
    let a= ["homebound"];
    for book in a{
        let path=format!("./data/{book}/{book}.epub");
        match check_epub(&path, book,book) {
            Ok(_)=>println!("{book} Toc ok"),
            Err(e)=> println!("Err in {book}: {e}")
        }
    }

    Ok(())
}
