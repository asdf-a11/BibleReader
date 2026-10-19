mod bible;
mod models;
mod stats;
mod storage;
mod tokenizer;
mod translations;
mod verse_translations;
mod word_state;

pub use models::{Bible, RawVerse, Token, WordStatus};
pub use stats::{days_from_date, today, CountHistory, StatsHistory};
pub use storage::{default_store, StateStore};
// Only the tests reach for the other two stores directly; keeping them out of
// the non-test build keeps `cargo build` free of unused import warnings.
#[cfg(test)]
pub use storage::{FileStore, MemoryStore};
pub use tokenizer::TextTokenizer;
pub use translations::Translations;
pub use verse_translations::VerseTranslations;
pub use word_state::WordStateStore;

/// Read the newline-delimited JSON bible export into memory.
pub fn load_bible_from_rst(path: &str) -> Result<Bible, String> {
    use std::io::BufRead;

    let file =
        std::fs::File::open(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    let mut verses = Vec::new();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.map_err(|e| format!("Failed to read '{}': {}", path, e))?;
        if line.trim().is_empty() {
            continue;
        }
        let verse: RawVerse = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        verses.push(verse);
    }

    Ok(Bible::from_raw_verses(verses))
}

pub fn load_translations(path: &str) -> Result<Translations, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    Translations::from_json(&content)
}

/// Read the sentence translations. They are optional: until
/// `generate_verse_translations.py` has been run there is no file, and the
/// reader simply shows the Russian on its own.
pub fn load_verse_translations(path: &str) -> Result<VerseTranslations, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(VerseTranslations::default())
        }
        Err(e) => return Err(format!("Failed to read '{}': {}", path, e)),
    };
    VerseTranslations::from_json(&content)
}

#[cfg(test)]
mod bible_test;
#[cfg(test)]
mod models_test;
#[cfg(test)]
mod stats_test;
#[cfg(test)]
mod storage_test;
#[cfg(test)]
mod tokenizer_test;
#[cfg(test)]
mod translations_test;
#[cfg(test)]
mod verse_translations_test;
#[cfg(test)]
mod word_state_test;
