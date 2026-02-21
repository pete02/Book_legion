use serde::{self,Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ncx {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub head: Head,
    #[serde(rename = "docTitle")]
    pub doc_title: DocTitle,
    #[serde(rename = "navMap")]
    pub nav_map: NavMap,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Head {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DocTitle {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavMap {
    #[serde(rename = "navPoint")]
    pub nav_point: Vec<NavPoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavPoint {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@playOrder")]
    pub play_order: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "navLabel")]
    pub nav_label: NavLabel,
    pub content: Content,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NavLabel {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    #[serde(rename = "@src")]
    pub src: String,
}

