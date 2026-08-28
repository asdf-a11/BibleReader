use crate::domain::{RawVerse, Token, WordStatus};

#[test]
fn test_raw_verse_deserializes_from_rst_line() {
    let line = r#"{"chapter":1,"verse":1,"text":"В начале сотворил Бог","translation_id":"RST","book_id":"Gen","book_name":"Genesis"}"#;
    let verse: RawVerse = serde_json::from_str(line).unwrap();
    assert_eq!(verse.chapter, 1);
    assert_eq!(verse.verse, 1);
    assert_eq!(verse.text, "В начале сотворил Бог");
    assert_eq!(verse.translation_id, "RST");
    assert_eq!(verse.book_id, "Gen");
    assert_eq!(verse.book_name, "Genesis");
}

#[test]
fn test_token_word_display_and_is_word() {
    let word = Token::Word {
        original: "Бог".to_string(),
        clean_key: "бог".to_string(),
    };
    assert!(word.is_word());
    assert_eq!(word.to_string(), "Бог");
    assert_eq!(word.as_word_str(), Some("Бог"));

    let punct = Token::Punctuation(",".to_string());
    assert!(!punct.is_word());
    assert_eq!(punct.to_string(), ",");
    assert_eq!(punct.as_word_str(), None);
}

#[test]
fn test_word_status_string_roundtrip() {
    assert_eq!(WordStatus::from_str("unseen"), WordStatus::Unseen);
    assert_eq!(WordStatus::from_str("seen"), WordStatus::Seen);
    assert_eq!(WordStatus::from_str("known"), WordStatus::Known);
    assert_eq!(WordStatus::from_str("bogus"), WordStatus::Unseen);

    assert_eq!(WordStatus::Unseen.to_str(), "unseen");
    assert_eq!(WordStatus::Seen.to_str(), "seen");
    assert_eq!(WordStatus::Known.to_str(), "known");
}