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
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "nav")]
    pub nav: Vec<Nav>,
}

#[derive(Serialize, Deserialize)]
pub struct Nav {
    #[serde(rename = "@type")]
    pub epub_type: String,

    #[serde(rename = "@hidden")]
    pub hidden: Option<String>,

    // Heading varies: h1 in toc, h2 in landmarks, absent in page-list
    #[serde(default)]
    pub h1: Option<String>,
    #[serde(default)]
    pub h2: Option<String>,

    pub ol: Ol,
}

#[derive(Serialize, Deserialize)]
pub struct Ol {
    #[serde(rename = "li", default)]
    pub li: Vec<Li>,
}

#[derive(Serialize, Deserialize)]
pub struct Li {
    pub a: Option<A>,
    // page-list and toc entries are flat <a> links;
    // if you ever have nested <ol> under <li> for sub-headings, add:
    // #[serde(default)]
    // pub ol: Option<Ol>,
}

#[derive(Serialize, Deserialize)]
pub struct A {
    #[serde(rename = "@href")]
    pub href: String,

    #[serde(rename = "@epub:type")]
    pub epub_type: Option<String>, // landmarks' <a> has this, toc/page-list don't

    #[serde(rename = "$text")]
    pub text: Option<String>,
}