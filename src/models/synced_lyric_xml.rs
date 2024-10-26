use serde::{Deserialize, Serialize};

use super::apple_music::AppleMusic;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "tt")]
pub struct SyncedLyricXML {
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Body {
    pub div: Vec<Div>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Div {
    pub p: Vec<P>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct P {
    #[serde(rename = "@begin")]
    pub begin: String,
    #[serde(rename = "@end")]
    pub end: String,
    pub span: Vec<Span>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Span {
    #[serde(rename = "@begin")]
    pub begin: Option<String>,
    #[serde(rename = "@end")]
    pub end: Option<String>,
    #[serde(rename = "$text")]
    pub word: Option<String>,
    pub span: Option<Vec<Span>>,
}

impl From<AppleMusic> for SyncedLyricXML {
    fn from(value: AppleMusic) -> Self {
        let data = value.data.first().unwrap();
        println!("data: {:#?}", data);
        let ttml = &data
            .relationships
            .syllable_lyrics
            .data
            .first()
            .unwrap()
            .attributes
            .ttml;
        quick_xml::de::from_str(ttml).unwrap()
    }
}
