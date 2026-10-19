//! Sentence by sentence English translations of the bible, produced by
//! `generate_verse_translations.py`.

use serde::Deserialize;
use std::collections::HashMap;

/// English for every translated sentence, keyed by
/// `(book_id, chapter, verse, sentence index within the verse)`.
#[derive(Debug, Clone, Default)]
pub struct VerseTranslations {
    map: HashMap<(String, u32, u32, u32), Entry>,
}

#[derive(Deserialize)]
struct VerseTranslationsFile {
    sentences: Vec<Entry>,
}

#[derive(Debug, Clone, Deserialize)]
struct Entry {
    book_id: String,
    chapter: u32,
    verse: u32,
    index: u32,
    russian: String,
    english: String,
}

impl VerseTranslations {
    /// The English for one sentence.
    ///
    /// `russian` must be the sentence exactly as the translation file has it:
    /// if the bible text or the sentence splitting ever changes, a stale
    /// translation is dropped rather than shown under the wrong sentence.
    pub fn english(
        &self,
        book_id: &str,
        chapter: u32,
        verse: u32,
        index: u32,
        russian: &str,
    ) -> Option<&str> {
        let entry = self
            .map
            .get(&(book_id.to_string(), chapter, verse, index))?;
        (entry.russian == russian && !entry.english.trim().is_empty())
            .then_some(entry.english.as_str())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn from_json(content: &str) -> Result<VerseTranslations, String> {
        let file: VerseTranslationsFile = serde_json::from_str(content)
            .map_err(|e| format!("Failed to parse verse translations: {}", e))?;
        let map = file
            .sentences
            .into_iter()
            .map(|entry| {
                (
                    (entry.book_id.clone(), entry.chapter, entry.verse, entry.index),
                    entry,
                )
            })
            .collect();
        Ok(VerseTranslations { map })
    }
}
