use crate::{
    lyrics::LyricsFormat,
    models::{apple_music::AppleMusic, user_storefront::UserStorefront},
    player::Player,
    utils::check_lyrics_exist,
    MediaEvent,
};
use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::{path::PathBuf, time::SystemTime};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::sleep,
};
use tracing::info;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Riri {
    storefront: Option<String>,
    user_token: String,
    authorization: Option<String>,
    expire: Option<i64>,
    #[serde(default)]
    offset: f64,
    length: Option<i64>,
    #[serde(default, skip_serializing)]
    player: Player,
}

impl Riri {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let mut riri = serde_yaml::from_str::<Riri>(&std::fs::read_to_string(&path)?)?;
        riri.player = Player::default();
        let now = chrono::Utc::now().timestamp_millis();

        if riri.expire.is_none() || riri.expire.unwrap() < now {
            riri.get_authorization().await?;
            riri.expire = Some(now + 12 * 60 * 60 * 1000);
        }

        if riri.storefront.is_none() {
            riri.get_user_storefront().await?;
        }

        if riri.length.is_none() {
            riri.length = Some(24);
        }

        let config = std::fs::File::create(path)?;
        serde_yaml::to_writer(config, &riri)?;

        Ok(riri)
    }

    pub async fn run(
        &mut self,
        lyrics_tx: Sender<String>,
        mut media_event_rx: Receiver<MediaEvent>,
    ) -> Result<()> {
        let mut not_download_able = Vec::new();
        let mut downloaded = Vec::new();
        let path = dirs::data_local_dir().unwrap().join("Riri").join("Data");
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            if file_name.ends_with(".xml") {
                let name_artist = file_name.trim_end_matches(".xml");
                downloaded.push(name_artist.to_string());
            }
        }

        loop {
            sleep(std::time::Duration::from_millis(100)).await;
            if let Ok(event) = media_event_rx.try_recv() {
                match event {
                    MediaEvent::Paused => {
                        self.player.playing = false;
                    }
                    MediaEvent::Playing(play_info) => {
                        self.player.play_info = Some(play_info);
                        self.player.playing = true;
                    }
                }
            }

            if let Some(play_info) = &self.player.play_info {
                if !self.player.playing {
                    continue;
                }
                if play_info.id.is_none() {
                    continue;
                }

                if downloaded.contains(&format!("{}-{}", play_info.name, play_info.artist))
                    || check_lyrics_exist(&play_info.name, &play_info.artist)
                {
                    let elapsed_duration =
                        std::time::Duration::from_secs_f64(play_info.elapsed_time);
                    let position = SystemTime::now()
                        .duration_since(play_info.timestamp.checked_sub(elapsed_duration).unwrap())
                        .unwrap()
                        .as_secs_f64();
                    let (lyric, _) = LyricsFormat::get_lyrics(
                        &play_info.name,
                        &play_info.artist,
                        position,
                        self.offset,
                        self.length.unwrap(),
                    );
                    lyrics_tx.send(lyric).await?;
                } else {
                    if not_download_able
                        .contains(&format!("{}-{}", play_info.name, play_info.artist))
                    {
                        continue;
                    }

                    info!(
                        "Downloading lyrics for {} by {}",
                        play_info.name, play_info.artist
                    );
                    if let Some(id) = play_info.id {
                        match self
                            .download_by_id(id, &play_info.name, &play_info.artist)
                            .await
                        {
                            Ok(_) => {
                                info!("Download success!");
                                downloaded.push(format!("{}-{}", play_info.name, play_info.artist));
                            }
                            Err(e) => {
                                info!("Download error: {:?}", e);
                                not_download_able
                                    .push(format!("{}-{}", play_info.name, play_info.artist));
                            }
                        };
                    } else {
                        not_download_able.push(format!("{}-{}", play_info.name, play_info.artist));
                    }
                }
            } else {
                lyrics_tx.send("🎵".to_string()).await?;
            }
        }
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
        self.storefront = Some(
            user_storefront
                .data
                .first()
                .ok_or(anyhow!("Can't found user storefront"))?
                .id
                .clone(),
        );
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

    pub fn create_lyrics_url(&self, song_id: i64) -> String {
        format!("https://amp-api.music.apple.com/v1/catalog/{}/songs/{}?include[songs]=albums,lyrics,syllable-lyrics", self.storefront.clone().unwrap(), song_id)
    }

    pub async fn download_by_id(&self, song_id: i64, name: &str, artist_name: &str) -> Result<()> {
        let url = self.create_lyrics_url(song_id);

        let headers = self.create_header();

        let client = Client::builder().default_headers(headers).build()?;

        let res = client.get(url).send().await?;

        let res_string = res.text().await.unwrap();

        let apple_music = serde_json::from_str::<AppleMusic>(&res_string)?;

        if apple_music.data.is_empty() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_auth() {
        let mut riri = Riri {
            storefront: None,
            user_token: String::new(),
            authorization: None,
            expire: None,
            offset: 0.0,
            length: None,
            player: Player::default(),
        };
        riri.get_authorization().await.unwrap();

        assert!(riri.authorization.is_some());
    }
}
