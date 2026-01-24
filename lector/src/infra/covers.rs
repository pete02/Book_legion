use web_sys::{Blob, Url};
use crate::infra::auth;

#[cfg(not(feature = "mock"))]
pub async fn fetch_cover(url: &str) -> Result<String, String> {

    let resp = auth::get_with_auth(&format!("/api/v1/{}",url )).await?;
    let bytes = resp.binary().await.map_err(|e| e.to_string())?;

    let array = js_sys::Uint8Array::from(bytes.as_slice());
    let blob = Blob::new_with_u8_array_sequence(&array.into())
        .map_err(|_| "failed to create blob")?;

    Ok(Url::create_object_url_with_blob(&blob)
        .map_err(|_| "failed to create object url")?)
}



#[cfg(feature = "mock")]
use dioxus::logger::tracing;

#[cfg(feature = "mock")]
pub async fn fetch_cover(mock_url: &str) -> Result<String, String> {
    let parts: Vec<&str> = mock_url.split('/').collect();

    if !mock_url.contains("/api/v1/books/") ||!mock_url.contains("/cover") {
        tracing::error!("Using wrong url: {}",mock_url)
    }

    Ok(crate::assets::MOCK_COVER.to_string())
}