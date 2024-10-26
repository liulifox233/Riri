use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserStorefront {
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub href: String,
    pub attributes: Attributes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    #[serde(rename = "supportedLanguageTags")]
    supported_language_tags: Vec<String>,
    #[serde(rename = "defaultLanguageTag")]
    default_language_tag: String,
    name: String,
    #[serde(rename = "explicitContentPolicy")]
    explicit_content_policy: String,
}
