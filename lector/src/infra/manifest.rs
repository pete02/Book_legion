
use serde::{Serialize, Deserialize};

#[cfg(feature = "mock")]
use serde_json::json;

#[cfg(feature = "mock")]
use dioxus::{logger::tracing, prelude::trace};


#[derive(Deserialize)]
pub struct ManifestResponse {
    pub series: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ManifestEntry {
    pub series_id: String,
    pub series_name: String,
    pub first_book_id: String,
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_manifest() -> Result<Vec<ManifestEntry>, Box<dyn std::error::Error>> {
    use crate::infra::auth;

    let resp = auth::get_with_auth("/api/v1/manifest").await?;

    if !resp.ok() {
        return Err(format!("manifest request failed: {}", resp.status()).into());
    }

    let manifest = resp.json::<ManifestResponse>().await?;
    Ok(manifest.series)
}

#[cfg(feature = "mock")]
pub async fn fetch_manifest() -> Result<Vec<ManifestEntry>, Box<dyn std::error::Error>> {
    tracing::debug!("mock fetch");
    let manifest_json = json!({
        "series": [
            {
                "series_id": "s1",
                "series_name": "Series one",
                "first_book_id": "b1"
            },
            {
                "series_id": "s2",
                "series_name": "Series two",
                "first_book_id": "b2"
            },
            {
                "series_id": "s3",
                "series_name": "Series three",
                "first_book_id": "b5"
            }
        ]
    });

    let manifest: ManifestResponse= serde_json::from_value(manifest_json.clone())
        .map_err(|e| e.to_string())?;

    Ok(manifest.series)
}