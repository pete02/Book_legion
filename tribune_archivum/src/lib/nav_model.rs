use serde::{self, Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Html {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "@xmlns:epub")]
    pub xmlns_epub: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub head: Head,
    pub body: Body,
}

#[derive(Serialize, Deserialize)]
pub struct Head {
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub nav: Vec<Nav>,
}

#[derive(Serialize, Deserialize)]
pub struct Nav {
    #[serde(rename = "@type")]
    pub epub_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub h1: String,
    pub ol: Ol,
}

#[derive(Serialize, Deserialize)]
pub struct Ol {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub li: Vec<Li>,
}

#[derive(Serialize, Deserialize)]
pub struct Li {
    pub a: A,
}

#[derive(Serialize, Deserialize)]
pub struct A {
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

