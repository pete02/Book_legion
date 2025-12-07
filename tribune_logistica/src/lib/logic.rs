use axum::{
    Json, body::Body, extract::{Path, Query, State}, http::{HeaderMap, HeaderValue, Response, StatusCode, header}, response::IntoResponse
};

use serde_json::json;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::BookStatus;

use crate::AppState;


use crate::book_handler::*; // your existing functions

// ----------------------
// /init?name=...&type=...
// ----------------------
#[derive(Deserialize)]
pub struct InitQuery {
    name: String,
    #[serde(rename = "type")]
    book_type: String,
}

pub async fn init_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<InitQuery>,
) -> impl IntoResponse {
    match init_book(&params.name, &params.book_type, &state.path()) {
        Ok(status) => Json(serde_json::to_value(status).unwrap()).into_response(),
        Err(err) => Json(err).into_response(),
    }
}




pub async fn book_handler(Json(book): Json<BookStatus>) -> impl IntoResponse {
    let mut headers: HeaderMap = HeaderMap::new();
    match get_chapter(Some(book)) {
        Ok(text) => {
            headers.insert("Content-Type", HeaderValue::from_static("text/html; charset=utf-8"));
            headers.insert("status", HeaderValue::from_static("ok"));
            (headers,text).into_response()
        },
        Err(e) => {
            headers.insert("Content-Type", HeaderValue::from_static("text/html; charset=utf-8"));
            headers.insert("status", HeaderValue::from_static("error"));
            (headers,e).into_response()
        },
    }
}



pub async fn audiomap(Json(book): Json<BookStatus>) -> impl IntoResponse {
    println!("audiomap");
    match get_audiomap(&format!("{}/{}.json",book.name,book.name)){
        Ok(map)=>    Json(json!({"status":"ok","data":map})).into_response(),
        Err(e)=>{
            println!("{}",e);
            Json(json!({"status":"error","data":"error in audiomap"})).into_response()
        }
    }
}


#[derive(Deserialize)]
pub struct AudioQuery {
    chunk: u32,
}

pub async fn audio_handler(
    Query(query): Query<AudioQuery>,
    Json(book): Json<BookStatus>,
) -> impl IntoResponse {
    println!("got audio request");
    match get_audio_chunk(
        Some(&book),
        query.chunk
    ) {
        Ok(chunk) => {
            println!("Sending audio");
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("audio/mpeg"));
            headers.insert("Content-Length", HeaderValue::from_str(&chunk.data.len().to_string()).unwrap());
            headers.insert("Reached-End", HeaderValue::from_str(&chunk.reached_end.to_string()).unwrap());
            
            headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
            (headers, chunk.data).into_response()
        }
        Err(err) => {
            println!("{}",err);
            Json(json!({ "status": "error", "message": err.to_string() })).into_response()
        },
    }
}
pub async fn update_handler(Json(book): Json<BookStatus>) -> impl IntoResponse {
    println!("updated");
    match update_progress(Some(book)) {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(e) => Json(json!({ "status": "error", "message": e })).into_response(),
    }
}


pub async fn manifest_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match get_library_manifest(&state.path()) {
        Ok(data) => Json(serde_json::from_str::<serde_json::Value>(&data).unwrap()).into_response(),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })).into_response(),
    }
}

pub async fn cover_handler(
    Path(book): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let book = format!("./{}/{}/{}.epub",&state.prefix, book,book);

    match extract_files(&book, vec![".jpg", ".jpeg"]) {
        Ok(css)=>{
            let mut values = css.values();
            if values.len() == 1{
                let Some(v)=values.next()else {return const_err_response("could not extract image".to_owned())};
                return ([("Content-Type", "image/jpeg")],v.to_owned()).into_response();
            }else{
                return const_err_response(format!("cover not unabigious: {} pieces",values.len()));
            }
        }
        Err(e)=> const_err_response(format!("could not extract cover: {}", e))
    }
}


pub async fn css_handler( Path(book): Path<String>,
    State(state): State<Arc<AppState>>,
)->impl IntoResponse{
    let book = format!("./{}/{}/{}.epub",&state.prefix, book,book);
    match extract_css(&book){
        Ok(css)=>([(header::CONTENT_TYPE, "text/css; charset=utf8")],css).into_response(),
        Err(e)=> const_err_response(format!("could not extract css: {}", e))
    }
}


fn const_err_response(err:String)->Response<Body>{
    (StatusCode::INTERNAL_SERVER_ERROR,err ).into_response()
}