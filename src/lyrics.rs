use crate::models::{
    lyric_json::LyricsJSON, lyric_xml::LyricXML, synced_lyric_xml::SyncedLyricXML,
};

pub enum LyricsFormat {
    LyricsJSON(LyricsJSON),
    LyricXML(LyricXML),
    SyncedLyricXML(SyncedLyricXML),
}

use anyhow::Result;

impl LyricsFormat {
    pub fn save(&self, name: &String, artist: &String) -> Result<()> {
        let data_path = dirs::data_local_dir().unwrap().join("Riri").join("Data");
        if !data_path.exists() {
            let _ = std::fs::create_dir(data_path.clone());
        }
        match &self {
            &LyricsFormat::LyricsJSON(lyrics) => {
                let data = serde_json::to_string_pretty(&lyrics)?;
                std::fs::write(data_path.join(format!("{}-{}.json", name, artist)), data)?;
            }
            &LyricsFormat::LyricXML(lyrics) => {
                let data = quick_xml::se::to_string(&lyrics)?;
                std::fs::write(data_path.join(format!("{}-{}.xml", name, artist)), data)?;
            }
            &LyricsFormat::SyncedLyricXML(lyrics) => {
                let data = quick_xml::se::to_string(&lyrics)?;
                std::fs::write(data_path.join(format!("{}-{}.xml", name, artist)), data)?;
            }
        }
        Ok(())
    }

    pub fn get_lyrics(name: &String, artist: &String, position: f64, offset: f64) -> (String, f64) {
        let position = position + offset;
        let path = dirs::data_local_dir()
            .unwrap()
            .join("Riri")
            .join("Data")
            .join(format!("{}-{}.xml", name, artist));
        let data = std::fs::read_to_string(path).unwrap();
        let lyrics = quick_xml::de::from_str::<LyricXML>(&data).unwrap();
        let current_line = lyrics.body.div.iter().flat_map(|div| &div.p).find(|line| {
            position < Self::parse_time(&line.end)
                && position > Self::parse_time(&line.begin)
        });

        let (lyric, duration) = match current_line {
            Some(line) => {
                let lyric = match line.line.chars().count() {
                    0..15 => line.line.clone(),
                    _ => format!("{}...", line.line.chars().take(12).collect::<String>()),
                };
                let duration = Self::parse_time(&line.end) - position;
                (lyric, duration)
            }
            None => {
                let duration = match lyrics
                    .body
                    .div
                    .iter()
                    .flat_map(|div| &div.p)
                    .find(|line| position < Self::parse_time(&line.begin))
                {
                    Some(line) => Self::parse_time(&line.begin) - position,
                    None => 1.0,
                };
                (" ".to_string(), duration)
            }
        };

        (lyric, duration)
    }

    fn parse_time(time_string: &String) -> f64 {
        let time = time_string.split(":").collect::<Vec<&str>>();
        match time.len() {
            1 => time[0].parse::<f64>().unwrap(),
            2 => time[0].parse::<f64>().unwrap() * 60.0 + time[1].parse::<f64>().unwrap(),
            3 => {
                time[0].parse::<f64>().unwrap() * 3600.0
                    + time[1].parse::<f64>().unwrap() * 60.0
                    + time[2].parse::<f64>().unwrap()
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        let time = "1:30".to_string();
        let result = LyricsFormat::parse_time(&time);
        assert_eq!(result, 90.0);
    }
}
