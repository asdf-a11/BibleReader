use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Translations {
    map: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct TranslationsFile {
    translations: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    word: String,
    #[serde(rename = "closestWords")]
    closest_words: Vec<String>,
}

impl Translations {
    pub fn get(&self, word: &str) -> Option<&Vec<String>> {
        self.map.get(&word.to_lowercase())
    }

    /// Comma-separated translation string for the drawer, or a fallback message.
    pub fn display(&self, word: &str) -> String {
        match self.get(word) {
            Some(words) if !words.is_empty() => words.join(", "),
            _ => "translation not found".to_string(),
        }
    }

    pub fn from_json(content: &str) -> Result<Translations, String> {
        let file: TranslationsFile = serde_json::from_str(content)
            .map_err(|e| format!("Failed to parse translations: {}", e))?;
        let mut map = HashMap::new();
        for entry in file.translations {
            map.insert(entry.word.to_lowercase(), entry.closest_words);
        }
        Ok(Translations { map })
    }
}