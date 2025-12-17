
use tribune_vox::{make_audiobook, AudiobookOptions};
use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about = "\
Convert EPUB to audiobook
The book must have the following structure:
. 
└─ NAME 
   └─ NAME.epub
")]

struct Args {
    /// Book name
    #[arg(short, long)]
    name: String,

    /// Author name
    #[arg(short, long)]
    author: String,

    /// TTS server IP or URL
    #[arg(short, long, default_value = "0.0.0.0")]
    ip: String,


    /// Overwrite existing files
    #[arg(long, default_value_t = false)]
    overwrite: bool,

    /// Enable debug logging and extracts only the first chapter
    #[arg(long, default_value_t = false)]
    debug: bool,

    // give the starting chapter of the book
    #[arg(long, default_value_t = 0)]
    start: usize


}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let debug=args.debug.clone();
    let start=std::time::Instant::now();
    audiobook(args)?;
    let end=std::time::Instant::now();
    if debug{
        println!("time took: {:?}", end-start);
    }

    Ok(())
}


fn audiobook(args:Args)->Result<(),Box<dyn std::error::Error>>{
    let http=format!("http://{}:8000/tts",args.ip);
    make_audiobook(&AudiobookOptions { 
        name:args.name,
        author: args.author,
        ip: http,
        overwrite: args.overwrite,
        debug: args.debug,
        initial:args.start })?;
    Ok(())
}

