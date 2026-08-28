mod bible;
mod models;
mod storage;
mod tokenizer;
mod translations;
mod word_state;

pub use models::{Bible, Book, RawVerse, Token, WordStatus};
pub use storage::{load_states, save_states};
pub use tokenizer::TextTokenizer;
pub use translations::Translations;
pub use word_state::WordStateStore;

pub async fn load_bible_from_rst(path: &str) -> Result<Bible, String> {
    use macroquad::prelude::*;
    use std::io::BufRead;

    let content = load_file(path)
        .await
        .map_err(|e| e.to_string())?;

    let mut verses = Vec::new();
    let reader = std::io::Cursor::new(content);
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let verse: RawVerse = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        verses.push(verse);
    }

    Ok(Bible::from_raw_verses(verses))
}

pub async fn load_translations(path: &str) -> Result<Translations, String> {
    use macroquad::prelude::*;

    let content = load_file(path)
        .await
        .map_err(|e| e.to_string())?;
    let content = String::from_utf8(content).map_err(|e| e.to_string())?;
    Translations::from_json(&content)
}

#[cfg(test)]
mod models_test;
#[cfg(test)]
mod bible_test;
#[cfg(test)]
mod tokenizer_test;
#[cfg(test)]
mod word_state_test;
#[cfg(test)]
mod storage_test;
#[cfg(test)]
mod translations_test;