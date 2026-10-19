use crate::domain::VerseTranslations;

const JSON: &str = r#"{
    "model": "gemma4:e4b",
    "sentences": [
        {"book_id":"Gen","chapter":1,"verse":3,"index":0,"russian":"И сказал Бог: да будет свет.","english":"And God said: let there be light."},
        {"book_id":"Gen","chapter":1,"verse":3,"index":1,"russian":"И стал свет.","english":"And there was light."},
        {"book_id":"Gen","chapter":1,"verse":4,"index":0,"russian":"Пусто.","english":"  "}
    ],
    "failed": [
        {"book_id":"Gen","chapter":1,"verse":5,"index":0,"russian":"И был вечер.","problem":"empty"}
    ]
}"#;

#[test]
fn test_looks_up_each_sentence_of_a_verse() {
    let translations = VerseTranslations::from_json(JSON).unwrap();
    assert_eq!(translations.len(), 3);
    assert_eq!(
        translations.english("Gen", 1, 3, 0, "И сказал Бог: да будет свет."),
        Some("And God said: let there be light.")
    );
    assert_eq!(
        translations.english("Gen", 1, 3, 1, "И стал свет."),
        Some("And there was light.")
    );
}

#[test]
fn test_missing_failed_or_blank_sentences_have_no_english() {
    let translations = VerseTranslations::from_json(JSON).unwrap();
    assert_eq!(translations.english("Gen", 1, 3, 2, "И стал свет."), None);
    assert_eq!(translations.english("Gen", 1, 5, 0, "И был вечер."), None);
    assert_eq!(translations.english("Gen", 1, 4, 0, "Пусто."), None);
    assert_eq!(translations.english("Exo", 1, 3, 0, "И сказал Бог: да будет свет."), None);
}

#[test]
fn test_a_stale_translation_is_not_shown_under_a_different_sentence() {
    let translations = VerseTranslations::from_json(JSON).unwrap();
    assert_eq!(translations.english("Gen", 1, 3, 1, "И стала тьма."), None);
}

#[test]
fn test_rejects_the_old_file_layout() {
    let old = r#"{"verse_translations":[{"book_id":"Gen","chapter":1,"verse":1,"translation":"x"}]}"#;
    assert!(VerseTranslations::from_json(old).is_err());
}

#[test]
fn test_a_missing_file_means_no_translations() {
    let translations = crate::domain::load_verse_translations("no/such/file.json").unwrap();
    assert!(translations.is_empty());
}
