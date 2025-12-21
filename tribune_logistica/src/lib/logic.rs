use axum::{
    Json, body::Body, 
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode, header}, 
    response::{IntoResponse}
};

use chrono::Duration;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::{ buffer_handler::{self, FillerCommand}, db_handlers, models::*, password_handler, update_handler};
use crate::AppState;
use crate::db_handlers::*;
use crate::book_handler::*; 



pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(login): Json<LoginRecord>
)-> impl IntoResponse{
    let ppath=&format!("{}/user.json",state.config);
    println!("REQUEST: /endpint login");
    println!("sourcing pasword from {}",ppath);
    let password_data=match get_password_data(&ppath){
        Err(a)=>return (StatusCode::INTERNAL_SERVER_ERROR,a.to_string()).into_response(),
        Ok(d)=>d
    };

    match password_handler::verify_login(&login,password_data.clone()) {
        Err(a)=>return (StatusCode::INTERNAL_SERVER_ERROR,a.to_string()).into_response(),
        Ok(res)=>{
            if res{
                let token=password_handler::generate_jwt(&login.username, &state.secret, Duration::minutes(5));
                match password_handler::generate_and_store_refresh_token(&login.username, password_data){
                    Ok((refresh,saving))=>{
                        let _=save_password_data(&ppath, &saving);
                        println!("REQUEST: user: {} endpoint: /login, Success ", &login.username);
                        return (StatusCode::OK, Json(json!({ "access_token": token, "refresh_token": refresh }))).into_response()
                    },
                    Err(_)=>{
                        println!("REQUEST: user: {} endpoint: /login,Denied ", &login.username);
                        println!("Error in generating refresh token");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
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
    let ppath=&format!("{}/user.json",state.config);
    let password_data=match get_password_data(&ppath){
        Err(a)=>return (StatusCode::INTERNAL_SERVER_ERROR,a.to_string()).into_response(),
        Ok(d)=>d
    };

     match password_handler::check_refesh_token(&refresh_record.username, &refresh_record.refresh_token, Duration::minutes(5), &state.secret,password_data) {
        Err(_)=>return StatusCode::FORBIDDEN.into_response(),
        Ok((access,(refresh,saving)))=>{
            let _=save_password_data(ppath, &saving);
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

    let manifest_path=format!("{}/{}",state.prefix,state.manifest);
    match db_handlers::init_book(&params.name, &params.book_type, &manifest_path,&state.prefix) {
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
    match get_chapter(&book) {
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

    match get_audiomap(&book){
        Ok(map)=>    Json(json!({"status":"ok","data":map})).into_response(),
        Err(e)=>{
            println!("{}",e);
            Json(json!({"status":"error","data":"error in audiomap"})).into_response()
        }
    }
}



pub async fn audio_endpoint(
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
    handle_audio_chunks(&book,query.chunk,state).await.into_response()
}

use tokio::sync::oneshot;

pub async fn handle_audio_chunks(
    status: &BookStatus,
    advance: u32,
    state: Arc<AppState>,
) -> Json<Value> {
    println!("asked: {}, {}", status.chapter, status.chunk);
    let buffer = {
        let mut lock = state.global_buffer.write().await;
        lock.get_or_insert_with(|| Arc::new(RwLock::new(AudioBuffer::new(3, 8))))
            .clone()
    };
    let tx = {
        let mut lock = state.filler_tx.write().await;

        if lock.is_none() {
            let sender = start_filler(buffer.clone()).await;
            *lock = Some(sender);
        }

        lock.as_ref().unwrap().clone()
    };

    // ALWAYS ensure — cheap, idempotent
    let (decision_tx, decision_rx) = oneshot::channel();

    // Send the Ensure command with the channel
    println!("start ensure");
    tx.send(FillerCommand::Ensure {
        book: BookKey {
            name: status.name.clone(),
            path: status.path.clone(),
        },
        start: ChunkCursor {
            chapter: status.chapter,
            chunk: status.chunk,
            chapter_to_chunk: status.chapter_to_chunk.clone(),
            max_chapter: status.max_chapter,
        },
        respond_to: Some(decision_tx),
    }).await.unwrap();

    // Await the decision from the filler
    let decision: buffer_handler::SeekDecision = decision_rx.await.unwrap();
    if decision == buffer_handler::SeekDecision::Reset{
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Wait ONLY if buffer empty
    println!("start read");
    loop {
        let buf = buffer.read().await;
        if !buf.chunks.is_empty() {
            break;
        }
        drop(buf);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    println!("end read");
    // Drain immediately available chunks
    let mut out = Vec::new();
    {
        print!("sending: ");
        let mut buf = buffer.write().await;
        for _ in 0..advance.min(buf.chunks.len() as u32) {
            if let Some(c) = buf.pop() {
                print!(", {}",c.place);
                out.push(c);
            }
        }
        println!("len: {}", out.len());
    }

    Json(json!({
        "status": "Ok",
        "chunks": out
    }))
}


/// Starts the global filler if not already started
pub async fn start_filler(buffer: Arc<RwLock<AudioBuffer>>) -> mpsc::Sender<FillerCommand> {
    let (tx, rx) = mpsc::channel(8);
    tokio::spawn(crate::buffer_handler::run_filler(rx, buffer));
    tx
}

pub async fn update_endpoint(
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
    handle_update(&book).into_response()
}

fn handle_update(status:&BookStatus)->Json<Value>{

    match update_handler::update_progress(status) {
        Ok(_) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "status": "error", "message": e })),
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
    handle_manifest(state).into_response()
}

fn handle_manifest(state:Arc<AppState>)->Json<Value>{
    let manifest_path=format!("{}/{}",state.prefix, state.manifest);
    match get_library_manifest(&manifest_path) {
        Ok(data) => Json(serde_json::from_str::<serde_json::Value>(&data).unwrap()),
        Err(e) => Json(json!({ "status": "error","message": e.to_string(), "path used": manifest_path })),
    }
}


pub async fn cover_handler(
    State(state): State<Arc<AppState>>,
    Path(book): Path<String>,
) -> impl IntoResponse {
    println!(" REQUEST: endoint: /cover book: {}",  book);

    let book = format!("{}/{}/{}.epub",state.prefix,book,book);

    match extract_cover(&book) {
        Ok(cover)=>return ([("Content-Type", "image/jpeg")],cover.to_owned()).into_response(),
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

    let book = format!("{}/{}/{}.epub",state.prefix,book,book);

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
    let username = match password_handler::verify_jwt(token, secret) {
        Ok(u) => u,
        Err(_) => return Err((StatusCode::FORBIDDEN, "Invalid token").into_response()),
    };
    return Ok(username);
}
