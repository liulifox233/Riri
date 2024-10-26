use serde::{Deserialize, Serialize};

use super::apple_music::AppleMusic;

#[derive(Serialize, Deserialize)]
#[serde(rename = "tt")]
pub struct LyricXML {
    pub body: Body,
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub div: Vec<Div>,
}

#[derive(Serialize, Deserialize)]
pub struct Div {
    pub p: Vec<P>,
}

#[derive(Serialize, Deserialize)]
pub struct P {
    #[serde(rename = "@begin")]
    pub begin: String,
    #[serde(rename = "@end")]
    pub end: String,
    #[serde(rename = "$text")]
    pub line: String,
}

impl From<AppleMusic> for LyricXML {
    fn from(value: AppleMusic) -> Self {
        let data = value.data.first().unwrap();
        let ttml = &data
            .relationships
            .lyrics
            .data
            .first()
            .unwrap()
            .attributes
            .ttml;
        quick_xml::de::from_str(ttml).unwrap()
    }
}
