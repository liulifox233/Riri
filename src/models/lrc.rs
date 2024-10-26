use super::synced_lyric_xml::SyncedLyricXML;

type Lrc = String;

impl From<SyncedLyricXML> for Lrc {
    fn from(synced_lyric_xml: SyncedLyricXML) -> Self {
        let mut lrc: String = String::new();
        let synced_lyric_array = &synced_lyric_xml.body.div;
        for div in synced_lyric_array {
            for p in &div.p {
                let line_start_time = convert_time_to_lrc_format(&p.begin);
                let mut line: String = format!("[{}] ", line_start_time);
                for span in &p.span {
                    let span = span.clone();
                    match span.span {
                        Some(background) => {
                            for word in background {
                                let word_start_time =
                                    convert_time_to_lrc_format(&word.begin.unwrap());
                                let word: String =
                                    format!("<{}> {} ", word_start_time, word.word.unwrap());
                                line.push_str(&word);
                            }
                        }
                        None => {
                            let word_start_time = convert_time_to_lrc_format(&span.begin.unwrap());
                            let word: String =
                                format!("<{}> {} ", word_start_time, span.word.unwrap());
                            line.push_str(&word);
                        }
                    }
                }
                line.push('\n');
                lrc.push_str(&line);
            }
        }
        lrc
    }
}

fn convert_time_to_lrc_format(time: &str) -> String {
    let mut min: String = String::new();
    let mut sec: String = String::new();
    let mut ms: String = String::new();

    let mut temp: String = String::new();
    for char in time.chars() {
        match char {
            ':' => {
                min = temp.clone();
                temp.clear();
            }
            '.' => {
                sec = temp.clone();
                temp.clear();
            }
            _ => temp.push(char),
        }
    }
    if !temp.is_empty() {
        ms = temp.clone();
    }

    if min.len() < 2 && !min.is_empty() {
        min.insert(0, '0');
    } else if min.is_empty() {
        min = "00".to_string();
    }
    if sec.len() < 2 && !sec.is_empty() {
        sec.insert(0, '0');
    } else if sec.is_empty() {
        sec = "00".to_string();
    }
    if ms.len() > 2 {
        ms.pop();
    }
    format!("{}:{}.{}", min, sec, ms)
}
