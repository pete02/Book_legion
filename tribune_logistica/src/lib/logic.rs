use axum::{
    Json, body::Body, 
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode, header}, 
    response::{IntoResponse}
};

use chrono::Duration;
use serde_json::json;
use serde::Deserialize;
use std::sync::Arc;

use crate::{models::{BookStatus, InitQuery, LoginRecord, RefreshRecord}, password_handler::{check_refesh_token, generate_and_store_refresh_token, generate_jwt, verify_jwt, verify_login}};

use crate::AppState;


use crate::book_handler::*; // your existing functions

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(login): Json<LoginRecord>
)-> impl IntoResponse{
    match verify_login(&login) {
        Err(_)=>return StatusCode::FORBIDDEN.into_response(),
        Ok(res)=>{
            if res{
                println!("REQUEST: user: {} endpoint: /login, Success ", &login.username);
                let token=generate_jwt(&login.username, &state.secret, Duration::minutes(5));
                match generate_and_store_refresh_token(&login.username){
                    Ok(refresh)=>return (StatusCode::OK, Json(json!({ "access_token": token, "refresh_token": refresh }))).into_response(),
                    Err(_)=>return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }else{
                println!("REQUEST: user: {} endpoint: /login, Denied", &login.username);
                return StatusCode::FORBIDDEN.into_response()
            }
        }
        
    }
}


pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(refresh_record): Json<RefreshRecord>
)-> impl IntoResponse{
     match check_refesh_token(&refresh_record.username, &refresh_record.refresh_token, Duration::minutes(5), &state.secret) {
        Err(_)=>return StatusCode::FORBIDDEN.into_response(),
        Ok((access,refresh))=>{
            println!("TOKEN REFRESHED: user: {}",refresh_record.username);
            return (StatusCode::OK, Json(json!({ "access_token": access, "refresh_token": refresh }))).into_response()
        },
    }
}

pub async fn init_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<InitQuery>,
    headers: axum::http::HeaderMap
) -> impl IntoResponse {
    let user = match check_token(state.secret.as_ref(), &headers) {
            Ok(u) => u,
            Err(resp) => {
                println!("REQUEST DENIED: endpoint /init");
                return resp
            },
        };
    println!(" REQUEST: user: {} , endoint: /init", user);


    match init_book(&params.name, &params.book_type, &state.path()) {
        Ok(status) => Json(serde_json::to_value(status).unwrap()).into_response(),
        Err(err) => Json(err).into_response(),
    }
}


pub async fn book_handler(State(state): State<Arc<AppState>>,h:HeaderMap, Json(book): Json<BookStatus>) -> impl IntoResponse {
    let user = match check_token(state.secret.as_ref(), &h) {
            Ok(u) => u,
            Err(resp) => {
                println!("REQUEST DENIED: endpoint /book");
                return resp
            },
        };
    println!(" REQUEST: user: {} , endoint: /book book:{}, chapter: {}", user, book.name, book.chapter);

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



pub async fn audiomap(State(state): State<Arc<AppState>>, h: HeaderMap, Json(book): Json<BookStatus>) -> impl IntoResponse {
    let user = match check_token(state.secret.as_ref(), &h) {
        Ok(u) => u,
        Err(resp) => {
            println!("REQUEST DENIED: endpoint /audiomap");
            return resp
        },
    };
    println!(" REQUEST: user: {} , endoint: /audiomap, book: {}", user, book.name);

    let path=format!("{}/{}.json",book.name,book.name);
    match get_audiomap(&path){
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
    State(state): State<Arc<AppState>>,
    h: HeaderMap,
    Query(query): Query<AudioQuery>,
    Json(book): Json<BookStatus>,
) -> impl IntoResponse {
    
    let user = match check_token(state.secret.as_ref(), &h) {
        Ok(u) => u,
        Err(resp) =>{
            println!("REQUEST DENIED: endpoint: /audio");
            return resp
        },
    };
    println!(" REQUEST: user: {} , endoint: /audio={}, book: {}", user, query.chunk, book.name);

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
pub async fn update_handler(
    State(state): State<Arc<AppState>>,
    h: HeaderMap,
    Json(book): Json<BookStatus>
) -> impl IntoResponse {
    let user = match check_token(state.secret.as_ref(), &h) {
        Ok(u) => u,
        Err(resp) => {
            println!("REQUEST DENIED: endpoint / update");
            return resp
        },
    };
    println!(" REQUEST: user: {} , endoint: /update book: {}", user,  book.name);

    match update_progress(Some(book)) {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(e) => Json(json!({ "status": "error", "message": e })).into_response(),
    }
}


pub async fn manifest_handler(
State(state): State<Arc<AppState>>,
h: HeaderMap
) -> impl IntoResponse {
    let user = match check_token(state.secret.as_ref(), &h) {
        Ok(u) => u,
        Err(resp) => {
            println!("REQUEST DENIED: endpoint /manifest");
            return resp
        },
    };
    println!(" REQUEST: user: {} , endoint: /manifest", user);

    match get_library_manifest(&state.path()) {
        Ok(data) => Json(serde_json::from_str::<serde_json::Value>(&data).unwrap()).into_response(),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })).into_response(),
    }
}

pub async fn cover_handler(
    Path(book): Path<String>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    println!(" REQUEST: endoint: /cover book: {}",  book);

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
    h: HeaderMap
)->impl IntoResponse{
    let user = match check_token(state.secret.as_ref(), &h) {
        Ok(u) => u,
        Err(resp) => {
            println!("REQUEST DENIED: endpoint /css");
            return resp
        },
    };
    println!(" REQUEST: user: {} , endoint: /css book: {}", user,  book);

    let book = format!("./{}/{}/{}.epub",&state.prefix, book,book);
    match extract_css(&book){
        Ok(css)=>([(header::CONTENT_TYPE, "text/css; charset=utf8")],css).into_response(),
        Err(e)=> const_err_response(format!("could not extract css: {}", e))
    }
}


fn const_err_response(err:String)->Response<Body>{
    (StatusCode::INTERNAL_SERVER_ERROR,err ).into_response()
}


fn check_token(secret: &[u8], headers: &axum::http::HeaderMap)->Result<String,Response<Body> >{
    let token = match headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => return Err((StatusCode::FORBIDDEN, "Missing token").into_response()),
    };
    let username = match verify_jwt(token, secret) {
        Ok(u) => u,
        Err(_) => return Err((StatusCode::FORBIDDEN, "Invalid token").into_response()),
    };
    return Ok(username);
}
