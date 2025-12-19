use crate::{book_handler::get_chunk, models::*};
use regex::Regex;


use std::{fs::File, time::Duration};
use serde_json::json;
use hound::{SampleFormat, WavReader,  WavWriter};
use std::io::Cursor;
use std::thread::sleep;




use reqwest::Client;
pub async fn get_audio_chunk(status: &BookStatus) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_txt = tokio::task::spawn_blocking({
        let status = status.clone();
        move || get_chunk(&status)
    }).await??;

    let clean_text = tokio::task::spawn_blocking(move || clean_html(&chunk_txt)).await??;

    text_to_wav_async(&clean_text, "sofia", "http://192.168.88.244:8000/tts").await
}

pub fn clean_html(html:&str)->Result<String, Box<dyn std::error::Error + Send + Sync>>{
    let mut text = html.replace("</div>", ". </div>");

    let re: [(&str, &str); 4] = [
        (r"(?s)<\?xml[^>]*\?>", ""),   // Remove XML header
        (r"(?s)<head.*?>.*?</head>", ""), // Remove <head> content
        (r"</p>|</div>|<br\s*/?>", " "),  // Replace block tags with spaces
        (r"<[^>]+>", ""),                // Strip remaining tags
    ];

    for (pattern, replacement) in &re {
        let regex = Regex::new(pattern)?;
        text = regex.replace_all(&text, *replacement).to_string();
    }
    
    Ok(normalize_whitespace(&text))
}

fn normalize_whitespace(text: &str) -> String {
    let re_spaces = Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(&text.replace('\n', " "), " ").trim().to_string()
}

//ip=tribune_dictio:open port/tts
pub async fn text_to_wav_async(
    text: &str,
    voice: &str,
    ip: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    const MAX_RETRIES: usize = 3;

    for attempt in 1..=MAX_RETRIES {
        let result = client
            .post(ip)
            .json(&json!({
                "text": text,
                "voice": voice,
                "cfg_weight": "0.4",
                "temperature": "0.9"
            }))
            .send()
            .await;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let wav_bytes = response.bytes().await?.to_vec();
                    return Ok(wav_bytes);
                } else if attempt == MAX_RETRIES {
                    return Err(format!(
                        "Request failed after {} attempts, last status: {}",
                        attempt,
                        response.status()
                    ).into());
                }
            }
            Err(err) => {
                if attempt == MAX_RETRIES {
                    return Err(format!(
                        "Request failed after {} attempts: {}",
                        attempt, err
                    ).into());
                }
            }
        }

        // Exponential backoff: 500ms, 1s, 2s
        tokio::time::sleep(Duration::from_millis(500 * (1 << (attempt - 1)))).await;

    }

    unreachable!()
}