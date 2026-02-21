use log::{debug, error, info};
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


#[derive(Deserialize)]
struct SearchResponse {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    search: Search,
}

#[derive(Deserialize)]
struct Search {
    results: Results,
}

#[derive(Deserialize)]
struct Results {
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct Hit {
    document: Document,
}

#[derive(Deserialize)]
struct Document {
    alternative_titles: Option<Vec<String>>,
    author_names: Option<Vec<String>>,
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

const SEARCH:&str= r#"query LordOfTheRingsBooks($query: String!) {
  search(
      query: $query,
      query_type: "Book",
      per_page: 5,
      page: 1
  ) {
      results
  }
}"#;



pub async fn query_api(
    query: &str,
    variables: Value
) -> Result<String, ApiError> {
    let bearer_token = env::var("HARDCOVER_API_TOKEN")
        .map_err(|_| ApiError::MissingConfig("HARDCOVER_API_TOKEN".into()))?;

    let endpoint = env::var("HARDCOVER_API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.hardcover.app/v1/graphql".to_string());

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

fn extract_from_search_json(
    json: &str,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {

    let parsed: SearchResponse = serde_json::from_str(json)?;

    let first_hit = parsed
        .data
        .search
        .results
        .hits
        .get(0);

    if let Some(hit) = first_hit {
        let alt_title = hit
            .document
            .alternative_titles
            .as_ref()
            .and_then(|v| v.get(0))
            .cloned();

        let author = hit
            .document
            .author_names
            .as_ref()
            .and_then(|v| v.get(0))
            .cloned();

        return Ok((alt_title, author));
    }

    Ok((None, None))
}



use std::env;
pub async fn get_series_title(
    title: &str,
    author: &str,
) -> Result<Option<(String, Option<f32>)>, ApiError> {
    let variables=json!({
        "author": author,
        "title": title
    });

    match query_api( TITLE_AND_AUTHOR, variables).await {
        Ok(books_string) => {
            // Take the first matching edition
            let books:GraphQLResponse<BooksData>=serde_json::from_str(&books_string)?;
            if books.data.books.len() ==0{
                return Err(ApiError::NoResults);
            }

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

fn clean_title(title: &str, author: &Option<String>) -> String {
    if let Some(author_name) = author {
        let trimmed_title = title.trim();
        let trimmed_author = author_name.trim();

        let suffix = format!(" - {}", trimmed_author);

        // Case-insensitive end match
        if trimmed_title
            .to_lowercase()
            .ends_with(&suffix.to_lowercase())
        {
            let new_len = trimmed_title.len() - suffix.len();
            return trimmed_title[..new_len].trim_end().to_string();
        }
    }

    title.trim().to_string()
}

async fn get_book_data(epub_path: &str)->Result<BookData,Box<dyn std::error::Error>>{
    let mut bd=BookData::new();
    let (title_o,author_o)=extract_title_author(epub_path)?;
    if let Some(title)=title_o && let Some(author)=author_o{
        let cleaned=clean_title(&title, &Some(author.clone()));
        bd.title=cleaned;
        bd.author=author;

        match get_series_title(&bd.title, &bd.author).await{
            Ok(series_o)=>{
                if let Some((series, Some(pos)))=series_o{
                    bd.series=series;
                    bd.pos=pos.floor() as u32;
                }
            },
            Err(_)=>{
                let variables=json!({
                    "query": bd.title
                });
                let txt=query_api(SEARCH, variables).await?;
                let res=extract_from_search_json(&txt)?;
                debug!("res: {:?}",res);
                if let (Some(title),Some(author))=res{
                    let series_o=get_series_title(&title, &author).await?;
                        bd.title=title;
                        bd.author=author;
                        if let Some((series, Some(pos)))=series_o{
                            bd.series=series;
                            bd.pos=pos.floor() as u32;
                        }
                }
            }
        }


    }else{
        return Err("no title and author".into())
    }
    Ok(bd)
}

/// Scans a folder (and subfolders) for `.epub` files and returns a vector of BookData
pub async fn scan_epub_folder<P: AsRef<Path>>(
    folder: P,
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut epubs = Vec::new();
    let mut stack = vec![folder.as_ref().to_path_buf()];

    // --- Recursive collection ---
    while let Some(path) = stack.pop() {
        let mut dir = match fs::read_dir(&path).await {
            Ok(d) => d,
            Err(_) => continue,
        };

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .map(|s| s.eq_ignore_ascii_case("epub"))
                .unwrap_or(false)
            {
                epubs.push(path);
            }
        }
    }

    let output_dir = output_dir.as_ref().to_path_buf();

    // --- Concurrent processing ---
    stream::iter(epubs)
        .map(|epub_path| {
            let output_dir = output_dir.clone();
            async move {
                match get_book_data(epub_path.to_str().unwrap()).await {
                    Ok(data) => {
                        if let Err(e) =
                            handle_successful_book(&epub_path, &output_dir, &data).await
                        {
                            error!(
                                "Failed to move {}: {}",
                                epub_path.display(),
                                e
                            );
                        }
                    }
                    Err(err) => {
                        error!(
                            "Failed to read {}: {}",
                            epub_path.display(),
                            err
                        );
                    }
                }
            }
        })
        .buffer_unordered(5)
        .collect::<Vec<_>>() // drive stream
        .await;

    Ok(())
}

async fn handle_successful_book(
    source_path: &Path,
    output_dir: &Path,
    data: &BookData,
) -> Result<(), Box<dyn std::error::Error>> {

    // Sanitize components
    let author = sanitize_component(&data.author);
    let title = sanitize_component(&data.title);
    let series = sanitize_component(&data.series);

    // Build directory path
    let mut target_dir = output_dir.join(&author);

    if !series.is_empty() {
        target_dir = target_dir.join(series);
    }

    
    // Ensure directories exist
    fs::create_dir_all(&target_dir).await?;

    // Construct filename
    let filename = format!("{}.epub", title);
    let destination = target_dir.join(filename);

    // Collision handling (simple incremental suffix)
    let mut final_destination = destination.clone();
    let mut counter = 1;

    while final_destination.exists() {
        let new_name = format!("{} ({}){}.epub",
            title,
            counter,
            ""
        );
        final_destination = target_dir.join(new_name);
        counter += 1;
    }

    // Move file (fallback-safe version)
    match fs::rename(source_path, &final_destination).await {
        Ok(_) => {}
        Err(_) => {
            fs::copy(source_path, &final_destination).await?;
            fs::remove_file(source_path).await?;
        }
    }

    info!(
        "Moved → Title: {}, Author: {}, Series: {}, Pos: {}",
        data.title,
        data.author,
        data.series,
        data.pos
    );

    Ok(())
}

fn sanitize_component(input: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    input
        .chars()
        .filter(|c| !invalid.contains(c))
        .collect::<String>()
        .trim()
        .to_string()
}