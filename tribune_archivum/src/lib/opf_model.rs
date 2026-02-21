use serde::{self, Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Package {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub metadata: Metadata,
    pub manifest: Manifest,
    pub spine: Spine,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub item: Vec<Item>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "@media-type")]
    pub media_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct Spine {
    #[serde(rename = "@toc")]
    pub toc: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub itemref: Vec<Itemref>,
}

#[derive(Serialize, Deserialize)]
pub struct Itemref {
    #[serde(rename = "@idref")]
    pub idref: String,
}

