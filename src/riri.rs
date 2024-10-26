use crate::{
    lyrics::LyricsFormat,
    models::{apple_music::AppleMusic, user_storefront::UserStorefront},
};
use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::path::PathBuf;
use system_status_bar_macos::{Menu, MenuItem, StatusItem};
use tokio::time;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Riri {
    storefront: String,
    user_token: String,
    authorization: Option<String>,
    expire: Option<i64>,
    #[serde(default)]
    offset: f64,
    length: Option<i64>,
}

impl Riri {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let mut riri = serde_yaml::from_str::<Riri>(&std::fs::read_to_string(&path)?)?;

        let now = chrono::Utc::now().timestamp_millis();

        if riri.expire.is_none() || riri.expire.unwrap() < now {
            riri.get_authorization().await?;
            riri.expire = Some(now + 12 * 60 * 60 * 1000);
        }

        if riri.storefront.is_empty() {
            riri.get_user_storefront().await?;
        }

        if riri.length.is_none() {
            riri.length = Some(24);
        }

        let config = std::fs::File::create(path)?;
        serde_yaml::to_writer(config, &riri)?;

        Ok(riri)
    }

    pub async fn run(&self) -> Result<()> {
        let mut not_download_able = Vec::new();
        let mut status_item = StatusItem::new(
            "🎵",
            Menu::new(vec![MenuItem::new(
                "Quit",
                Some(Box::new(|| std::process::exit(0))),
                None,
            )]),
        );
        loop {
            let current_track = apple_music::AppleMusic::get_current_track().ok();
            if current_track.is_none() {
                time::sleep(time::Duration::from_secs(1)).await;
                continue;
            } else {
                let track = current_track.unwrap();
                let name = track.name.clone();
                let artist = track.artist.clone();
                let app_data =
                    apple_music::AppleMusic::get_application_data().map_err(anyhow::Error::msg)?;
                let position = app_data.player_position.unwrap_or(0.0);
                if self.check_lyrics_exist(&name, &artist) {
                    let (lyric, duration) = LyricsFormat::get_lyrics(
                        &name,
                        &artist,
                        position,
                        self.offset,
                        self.length.unwrap(),
                    );
                    status_item.set_title(&lyric);
                    time::sleep(time::Duration::from_secs_f64(duration)).await;
                } else {
                    status_item.set_title(format!(
                        "▶︎ {}",
                        LyricsFormat::length_cut(&name, self.length.unwrap())
                    ));
                    if not_download_able.contains(&format!("{}-{}", name, artist)) {
                        time::sleep(time::Duration::from_secs(1)).await;
                        continue;
                    }
                    println!("Downloading lyrics for {} by {}", name, artist);
                    match self.get_id_by_name_artist(&name, &artist).await {
                        Ok(id) => {
                            println!("Get id success!");
                            match self.download_by_id(&id, &name, &artist).await {
                                Ok(_) => {
                                    println!("Download success!");
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                    not_download_able.push(format!("{}-{}", name, artist));
                                }
                            };
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            not_download_able.push(format!("{}-{}", name, artist));
                        }
                    };
                }
            }
        }
    }

    fn check_lyrics_exist(&self, name: &String, artist: &String) -> bool {
        let path = dirs::data_local_dir()
            .unwrap()
            .join("Riri")
            .join("Data")
            .join(format!("{}-{}.xml", name, artist));
        path.exists()
    }

    pub fn create_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("origin", "https://music.apple.com".parse().unwrap());
        headers.insert(
            "Authorization",
            self.authorization.clone().unwrap().parse().unwrap(),
        );
        headers.insert("Media-User-Token", self.user_token.parse().unwrap());
        headers
    }

    pub async fn get_user_storefront(&mut self) -> Result<()> {
        let headers = self.create_header();
        let client = Client::builder().default_headers(headers).build().unwrap();

        let res = client
            .get("https://api.music.apple.com/v1/me/storefront")
            .send()
            .await?;
        let res_string = res.text().await?;
        let user_storefront: UserStorefront = serde_json::from_str(&res_string)?;
        self.storefront = user_storefront
            .data
            .first()
            .ok_or(anyhow!("Can't found user storefront"))?
            .id
            .clone();
        Ok(())
    }

    pub async fn get_authorization(&mut self) -> Result<()> {
        let res = reqwest::get("https://music.apple.com").await?;
        let res_text = res.text().await?;

        let js_re = Regex::new(r#"(?<=index)(.*?)(?=\.js")"#).unwrap();
        let js_file = js_re.find(&res_text).map(|value| value.unwrap().as_str())?;
        let js_res =
            reqwest::get(format!("https://music.apple.com/assets/index{js_file}.js")).await?;
        let js_res_text = js_res.text().await.unwrap();

        let token_re = Regex::new(r#"(?=eyJh)(.*?)(?=")"#).unwrap();
        let token = token_re
            .find(&js_res_text)
            .map(|value| value.unwrap().as_str())?;

        self.authorization = Some(format!("Bearer {token}"));
        Ok(())
    }

    pub fn create_lyrics_url(&self, song_id: &str) -> String {
        format!("https://amp-api.music.apple.com/v1/catalog/{}/songs/{}?include[songs]=albums,lyrics,syllable-lyrics", self.storefront, song_id)
    }

    pub fn create_search_url(&self, song_name: &str, artist_name: &str) -> String {
        let search_term = format!("{} {}", song_name, artist_name);
        let format = urlencoding::encode(search_term.as_str());
        format!("https://amp-api-edge.music.apple.com/v1/catalog/{}/search?limit=5&platform=web&term={}&with=serverBubbles&types=songs%2Cactivities", self.storefront, format)
    }

    pub async fn download_by_id(
        &self,
        song_id: &String,
        name: &String,
        artist_name: &String,
    ) -> Result<()> {
        let url = self.create_lyrics_url(&song_id);

        let headers = self.create_header();
        let client = Client::builder().default_headers(headers).build()?;

        let res = client.get(url).send().await?;
        let res_string = res.text().await.unwrap();

        let apple_music = serde_json::from_str::<AppleMusic>(&res_string)?;
        if apple_music.data.first().is_none() {
            return Err(anyhow!("No such a song"));
        }
        let data = apple_music.data.first().unwrap();
        let ttml = &data
            .relationships
            .lyrics
            .data
            .first()
            .ok_or(anyhow!("No lyrics found"))?
            .attributes
            .ttml;
        let lyric_xml = quick_xml::de::from_str(ttml)?;
        let lyrics = LyricsFormat::LyricXML(lyric_xml);
        lyrics.save(name, artist_name)?;
        Ok(())
    }

    pub async fn get_id_by_name_artist(
        &self,
        name: &String,
        artist_name: &String,
    ) -> Result<String> {
        let headers = self.create_header();
        let client = Client::builder().default_headers(headers).build().unwrap();
        let url = self.create_search_url(&name, &artist_name);

        let res = client.get(url).send().await?;
        let res_json: serde_json::Value = res.json().await?;
        let id = &res_json["results"]["top"]["data"]
            .as_array()
            .ok_or(anyhow!("Invalid JSON structure"))?
            .iter()
            .find(|data| {
                data["attributes"]["name"] == *name
                    && data["attributes"]["artistName"] == *artist_name
            })
            .ok_or(anyhow!("Song not found"))?["id"];
        Ok(id.as_str().unwrap().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_auth() {
        let mut riri = Riri {
            storefront: String::new(),
            user_token: String::new(),
            authorization: None,
            expire: None,
            offset: 0.0,
            length: None,
        };
        riri.get_authorization().await.unwrap();

        assert!(riri.authorization.is_some())
    }
}
