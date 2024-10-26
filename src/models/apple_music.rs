use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppleMusic {
    pub data: Vec<AppleMusicData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppleMusicData {
    pub relationships: Relationships,
    pub attributes: Attributes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    pub name: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Relationships {
    pub syllable_lyrics: Lyrics,
    pub lyrics: Lyrics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lyrics {
    pub href: String,
    pub data: Vec<LyricsDatum>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricsDatum {
    pub id: String,
    #[serde(rename = "type")]
    pub datum_type: String,
    pub attributes: TentacledAttributes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TentacledAttributes {
    pub ttml: String,
}
