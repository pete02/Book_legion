use serde_json::Value;
use thiserror::Error;
use reqwest::Client;
use serde_json::json;
use serde::{Deserialize};
use std::path::Path;
use crate::lib::{helpers, nav_model::A};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use tokio::fs;
use futures::stream::{self, StreamExt};


#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct BooksData {
    pub books: Vec<Book>,
}

#[derive(Debug, Deserialize)]
pub struct Book {
    pub title: String,
    pub release_date: Option<String>,
    pub slug: String,
    pub subtitle: Option<String>,
    pub featured_book_series: Option<FeaturedBookSeries>,
    pub contributions: Vec<Contribution>,
}

#[derive(Debug, Deserialize)]
pub struct FeaturedBookSeries {
    pub series: Series,
    pub position: Option<f32>
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Contribution {
    pub author: Author,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("GraphQL returned no results")]
    NoResults,

    #[error("Missing configuration: {0}")]
    MissingConfig(String),
}


#[derive(Debug)]
pub struct BookData{
    pub author: String,
    pub title: String,
    pub series: String,
    pub pos: u32
}
impl BookData {
    pub fn new()->BookData{
        return BookData{
            author: "".to_owned(),
            title: "".to_owned(),
            series: "".to_owned(),
            pos: 0
        }
    }
}

const TITLE_AND_AUTHOR:&str = r#"
    query GetSpecificEdition($title: String!, $author: String!) {
        books(
        where: {
            _and: [
            { title: { _eq: $title } }
            { contributions: { author: { name: { _eq: $author } } } }
            ]
        }
        ) {
        title
        release_date
        slug
        subtitle
        featured_book_series {
            series {
            name
            }
            position
        }
        contributions {
            author {
            name
            }
        }
        }
    }
"#;



pub async fn query_api(
    endpoint: &str,
    bearer_token: &str,
    query: &str,
    variables: Value
) -> Result<String, ApiError> {
    let client = Client::new();
    let body =json!({
            "query": query,
            "variables": variables
        });
    let response = client
        .post(endpoint)
        .bearer_auth(bearer_token)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let text = response.text().await?;
    Ok(text)
}



use std::env;
pub async fn get_series_title(
    title: &str,
    author: &str,
) -> Result<Option<(String, Option<f32>)>, ApiError> {
    let bearer_token = env::var("HARDCOVER_API_TOKEN")
        .map_err(|_| ApiError::MissingConfig("HARDCOVER_API_TOKEN".into()))?;

    let endpoint = env::var("HARDCOVER_API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.hardcover.app/v1/graphql".to_string());

    let variables=json!({
        "author": author,
        "title": title
    });

    match query_api(&endpoint, &bearer_token, TITLE_AND_AUTHOR, variables).await {
        Ok(books_string) => {
            // Take the first matching edition
            let books:GraphQLResponse<BooksData>=serde_json::from_str(&books_string)?;

            let series_tuple = books.data.books
                .into_iter()
                .find_map(|book| {
                    book.featured_book_series
                        .and_then(|fbs| {
                            fbs.series.name.map(|name| (name, fbs.position))
                        })
                });

            Ok(series_tuple)
        }
        Err(ApiError::NoResults) => Err(ApiError::NoResults),
        Err(e) => Err(e),
    }
}


pub fn extract_title_author(
    epub_path: &str,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {

    let mut archive = helpers::get_zip(Path::new(epub_path))?;
    let opf_path = helpers::read_container_opf_path(&mut archive)?;
    let mut opf_file = archive.by_name(&opf_path)?;

    let mut opf_xml = String::new();
    opf_file.read_to_string(&mut opf_xml)?;

    let mut reader = Reader::from_str(&opf_xml);
    

    let mut buf = Vec::new();

    let mut title: Option<String> = None;
    let mut author: Option<String> = None;

    let mut inside_title = false;
    let mut inside_creator = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = e.name();

                // Match local names only (ignore namespace prefix)
                if name.as_ref().ends_with(b"title") && title.is_none() {
                    inside_title = true;
                }

                if name.as_ref().ends_with(b"creator") && author.is_none() {
                    inside_creator = true;
                }
            }

            Event::Text(e) => {
                let text = e.decode()?;

                if inside_title && title.is_none() {
                    title = Some(text.to_string());
                }

                if inside_creator && author.is_none() {
                    author = Some(text.to_string());
                }
            }

            Event::End(ref e) => {
                let name = e.name();

                if name.as_ref().ends_with(b"title") {
                    inside_title = false;
                }

                if name.as_ref().ends_with(b"creator") {
                    inside_creator = false;
                }
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok((title, author))
}


async fn get_book_data(epub_path: &str)->Result<BookData,Box<dyn std::error::Error>>{
    let mut bd=BookData::new();
    let (title_o,author_o)=extract_title_author(epub_path)?;
    if let Some(title)=title_o && let Some(author)=author_o{
        bd.title=title;
        bd.author=author;
        let series_o=get_series_title(&bd.title, &bd.author).await?;
        if let Some((series, Some(pos)))=series_o{
            bd.series=series;
            bd.pos=pos.floor() as u32;
        }
    }else{
        return Err("no title and author".into())
    }
    Ok(bd)
}

/// Scans a folder (and subfolders) for `.epub` files and returns a vector of BookData
pub async fn scan_epub_folder<P: AsRef<Path>>(folder: P) -> Result<Vec<BookData>, Box<dyn std::error::Error>> {
    let mut epubs = Vec::new();

    // Collect all epub paths recursively
    let mut stack = vec![folder.as_ref().to_path_buf()];

    while let Some(path) = stack.pop() {
        let mut dir = match fs::read_dir(&path).await {
            Ok(d) => d,
            Err(_) => continue, // skip unreadable folders
        };

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map(|s| s.eq_ignore_ascii_case("epub")).unwrap_or(false) {
                epubs.push(path);
            }
        }
    }

    // Process all EPUBs concurrently (limit concurrency to avoid rate limits)
    let results = stream::iter(epubs)
        .map(|epub_path| async move {
            match get_book_data(epub_path.to_str().unwrap()).await {
                Ok(data) => Some(data),
                Err(err) => {
                    eprintln!("Failed to read {}: {}", epub_path.display(), err);
                    None
                }
            }
        })
        .buffer_unordered(5) // adjust concurrency: 5 parallel requests
        .filter_map(|x| async { x }) // discard failed EPUBs
        .collect::<Vec<_>>()
        .await;

    Ok(results)
}