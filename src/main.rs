use macroquad::prelude::*;

mod domain;
mod ui;

#[cfg(test)]
mod tests {
    use crate::domain::{RawVerse, Bible, TextTokenizer, WordStateStore, load_states, StateFile};
    use std::collections::HashMap;

    #[test]
    fn test_parse_raw_json_lines() {
        let json_line = r#"{"chapter":1,"verse":1,"text":"В начале","translation_id":"RST","book_id":"Gen","book_name":"Genesis"}"#;
        let verse: RawVerse = serde_json::from_str(json_line).unwrap();
        assert_eq!(verse.chapter, 1);
        assert_eq!(verse.verse, 1);
        assert_eq!(verse.text, "В начале");
        assert_eq!(verse.book_id, "Gen");
    }

    #[test]
    fn test_bible_hierarchy_building() {
        let raw_verses = vec![
            RawVerse {
                chapter: 1,
                verse: 1,
                text: "В начале".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Gen".to_string(),
                book_name: "Genesis".to_string(),
            },
            RawVerse {
                chapter: 1,
                verse: 2,
                text: "И сказал Бог".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Gen".to_string(),
                book_name: "Genesis".to_string(),
            },
            RawVerse {
                chapter: 2,
                verse: 1,
                text: "Так совершены".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Gen".to_string(),
                book_name: "Genesis".to_string(),
            },
        ];

        let bible = Bible::from_raw_verses(raw_verses);
        
        assert!(bible.books.contains_key("Gen"));
        let gen = bible.books.get("Gen").unwrap();
        assert_eq!(gen.chapters.len(), 2);
        assert!(gen.chapters.contains_key(&1));
        assert!(gen.chapters.contains_key(&2));
        
        let chapter_1 = gen.chapters.get(&1).unwrap();
        assert_eq!(chapter_1.verses.len(), 2);
    }

    #[test]
    fn test_tokenizer_word_punctuation_split() {
        let text = "В начале, сотворил Бог";
        let tokens = TextTokenizer::tokenize(text);
        
        assert_eq!(tokens.len(), 5);
        
        match &tokens[0] {
            crate::domain::Token::Word { original, clean_key } => {
                assert_eq!(original, "В");
                assert_eq!(clean_key, "в");
            }
            _ => panic!("Expected Word token"),
        }
        
        match &tokens[2] {
            crate::domain::Token::Punctuation(p) => {
                assert_eq!(p, ",");
            }
            _ => panic!("Expected Punctuation token"),
        }
        
        match &tokens[3] {
            crate::domain::Token::Word { original, .. } => {
                assert_eq!(original, "сотворил");
            }
            _ => panic!("Expected Word token"),
        }
    }

    #[test]
    fn test_word_status_transition_rules() {
        let mut store = WordStateStore::new();
        
        assert_eq!(store.get_status("привет"), crate::domain::WordStatus::Unseen);
        
        store.access_word("привет");
        assert_eq!(store.get_status("привет"), crate::domain::WordStatus::Seen);
        
        store.set_status("привет", crate::domain::WordStatus::Known);
        assert_eq!(store.get_status("привет"), crate::domain::WordStatus::Known);
        
        store.set_status("привет", crate::domain::WordStatus::Unseen);
        assert_eq!(store.get_status("привет"), crate::domain::WordStatus::Unseen);
    }

    #[test]
    fn test_storage_serialization_roundtrip() {
        let mut store = WordStateStore::new();
        store.set_status("привет", crate::domain::WordStatus::Seen);
        store.set_status("начале", crate::domain::WordStatus::Known);
        
        let states: HashMap<String, crate::domain::WordStatus> = store.get_all_states()
            .into_iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        let file = StateFile {
            states: states.iter()
                .map(|(word, status)| (word.clone(), status.to_str().to_string()))
                .collect(),
        };
        
        let json = serde_json::to_string(&file).unwrap();
        let parsed: StateFile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.states.len(), 2);
        assert!(parsed.states.iter().any(|(w, s)| w == "привет" && s == "seen"));
        assert!(parsed.states.iter().any(|(w, s)| w == "начале" && s == "known"));
    }

    #[test]
    fn test_missing_file_graceful_fallback() {
        let _ = std::fs::remove_file("word_states.json");
        
        let states = load_states();
        assert!(states.is_ok());
        assert!(states.unwrap().is_empty());
    }
}

fn app_conf() -> Conf {
    Conf {
        window_title: "Bible Reader".to_string(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(app_conf)]
async fn main() {
    ui::run_app().await;
}
