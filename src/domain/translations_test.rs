use crate::domain::Translations;

#[test]
fn test_parse_and_case_insensitive_lookup() {
    let json = r#"{
        "translations": [
            {"word": "начале", "closestWords": ["beginning", "start"]},
            {"word": "бог", "closestWords": ["god"]}
        ]
    }"#;
    let translations = Translations::from_json(json).unwrap();

    assert_eq!(translations.get("начале").unwrap()[0], "beginning");
    assert_eq!(translations.get("НАЧАЛЕ").unwrap()[1], "start");
    assert_eq!(translations.get("Бог").unwrap()[0], "god");
}

#[test]
fn test_display_joins_words_or_falls_back() {
    let json = r#"{"translations":[{"word":"начале","closestWords":["beginning","start"]}]}"#;
    let translations = Translations::from_json(json).unwrap();

    assert_eq!(translations.display("начале"), "beginning, start");
    assert_eq!(translations.display("несуществующее"), "translation not found");
}

#[test]
fn test_empty_input_is_valid() {
    let translations = Translations::from_json(r#"{"translations":[]}"#).unwrap();
    assert!(translations.get("бог").is_none());
}

#[test]
fn test_invalid_json_is_an_error() {
    assert!(Translations::from_json("not json").is_err());
}