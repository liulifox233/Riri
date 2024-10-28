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

    pub fn get_lyrics(
        name: &String,
        artist: &String,
        position: f64,
        offset: f64,
        length: i64,
    ) -> (String, f64) {
        let position = position + offset;
        let path = dirs::data_local_dir()
            .unwrap()
            .join("Riri")
            .join("Data")
            .join(format!("{}-{}.xml", name, artist));
        let data = std::fs::read_to_string(path).unwrap();
        let lyrics = quick_xml::de::from_str::<LyricXML>(&data).unwrap();

        let start_time = LyricsFormat::parse_time(&lyrics.body.div[0].p[0].begin);

        let current_line = lyrics.body.div.iter().flat_map(|div| &div.p).find(|line| {
            position < Self::parse_time(&line.end) && position > Self::parse_time(&line.begin)
        });

        let (lyric, duration) = match current_line {
            Some(line) => {
                let lyric = Self::length_cut(&line.line, length);
                let mut duration = Self::parse_time(&line.end) - position;
                if duration > 0.3 {
                    duration = 0.3;
                }
                (lyric, duration)
            }
            None => {
                let mut duration = match lyrics
                    .body
                    .div
                    .iter()
                    .flat_map(|div| &div.p)
                    .find(|line| position < Self::parse_time(&line.begin))
                {
                    Some(line) => Self::parse_time(&line.begin) - position,
                    None => 1.0,
                };
                if duration > 0.3 {
                    duration = 0.3;
                }
                let title = match start_time > position {
                    true => "▶︎ ".to_string() + &Self::length_cut(name, length),
                    false => "   ".to_string(),

                };
                (title, duration)
            }
        };

        (lyric, duration)
    }

    fn parse_time(time_string: &String) -> f64 {
        let time = time_string.split(":").collect::<Vec<&str>>();
        match time.len() {
            1 => match time[0].contains("s") {
                true => time[0].replace("s", "").parse::<f64>().unwrap(),
                false => time[0].parse::<f64>().unwrap(),
            }
            2 => time[0].parse::<f64>().unwrap() * 60.0 + time[1].parse::<f64>().unwrap(),
            3 => {
                time[0].parse::<f64>().unwrap() * 3600.0
                    + time[1].parse::<f64>().unwrap() * 60.0
                    + time[2].parse::<f64>().unwrap()
            }
            _ => 0.0,
        }
    }

    pub fn length_cut(lyric: &String, len: i64) -> String {
        let mut length = 0;
        let mut temp = String::new();
        for c in lyric.chars() {
            if c.is_ascii_alphabetic() || c.is_numeric() || c.is_whitespace() {
                length += 1;
            } else {
                length += 2;
            }
            if length > len {
                temp.push_str("...");
                break;
            }
            temp.push(c);
        }
        temp
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

    #[test]
    fn test_cut() {
        let lyric = "無情な世界を恨んだ目は どうしようもなく愛を欲してた".to_string();
        let lyric = LyricsFormat::length_cut(&lyric, 24);
        assert_eq!(lyric, "無情な世界を恨んだ目は ...");
    }
}
