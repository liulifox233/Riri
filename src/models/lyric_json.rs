use serde::{Deserialize, Serialize};

use super::synced_lyric_xml::SyncedLyricXML;

#[derive(Serialize, Deserialize)]
pub struct Line {
    begin: String,
    end: String,
    words: Vec<Word>,
    background: Vec<Word>,
}

#[derive(Serialize, Deserialize)]
pub struct Word {
    begin: String,
    end: String,
    text: String,
}

impl LyricsJSON {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }
}

impl Line {
    pub fn new(begin: String, end: String) -> Self {
        Self {
            begin,
            end,
            words: Vec::new(),
            background: Vec::new(),
        }
    }

    pub fn add_words(&mut self, word: Word) {
        self.words.push(word);
    }

    pub fn add_background(&mut self, word: Word) {
        self.background.push(word);
    }
}

impl Word {
    pub fn new(begin: String, end: String, text: String) -> Self {
        Self { begin, end, text }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LyricsJSON {
    lines: Vec<Line>,
}

impl From<SyncedLyricXML> for LyricsJSON {
    fn from(synced_lyric_xml: SyncedLyricXML) -> Self {
        let synced_lyric_array = &synced_lyric_xml.body.div;
        let mut lyrics = LyricsJSON::new();
        for div in synced_lyric_array {
            for p in &div.p {
                let mut line: Line = Line::new(p.begin.clone(), p.end.clone());
                for span in &p.span {
                    let span = span.clone();
                    match span.span {
                        Some(background) => {
                            for word in background {
                                let word: Word = Word::new(
                                    word.begin.unwrap(),
                                    word.end.unwrap(),
                                    word.word.unwrap(),
                                );
                                line.add_background(word);
                            }
                        }
                        None => {
                            let word: Word = Word::new(
                                span.begin.unwrap(),
                                span.end.unwrap(),
                                span.word.unwrap(),
                            );
                            line.add_words(word);
                        }
                    }
                }
                lyrics.add_line(line);
            }
        }
        lyrics
    }
}
