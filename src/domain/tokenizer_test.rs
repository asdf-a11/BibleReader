use crate::domain::{TextTokenizer, Token};

#[test]
fn test_tokenize_simple_sentence() {
    let tokens = TextTokenizer::tokenize("В начале");
    assert_eq!(tokens.len(), 2);
    match &tokens[0] {
        Token::Word { original, clean_key } => {
            assert_eq!(original, "В");
            assert_eq!(clean_key, "в");
        }
        _ => panic!("Expected Word token"),
    }
    match &tokens[1] {
        Token::Word { original, clean_key } => {
            assert_eq!(original, "начале");
            assert_eq!(clean_key, "начале");
        }
        _ => panic!("Expected Word token"),
    }
}

#[test]
fn test_tokenize_with_punctuation() {
    let tokens = TextTokenizer::tokenize("В начале,");
    assert_eq!(tokens.len(), 3);
    match &tokens[2] {
        Token::Punctuation(p) => assert_eq!(p, ","),
        _ => panic!("Expected Punctuation token"),
    }
}

#[test]
fn test_tokenize_mixed_content() {
    let tokens = TextTokenizer::tokenize("В начале, сотворил Бог...");
    assert_eq!(tokens.len(), 6);

    assert_eq!(tokens[0].as_word_str(), Some("В"));
    assert_eq!(tokens[3].as_word_str(), Some("сотворил"));
    match &tokens[5] {
        Token::Punctuation(p) => assert_eq!(p, "..."),
        _ => panic!("Expected Punctuation token"),
    }
}

#[test]
fn test_punctuation_does_not_leak_into_clean_key() {
    let tokens = TextTokenizer::tokenize("Земля, была.");
    let word = tokens.iter().find(|t| t.as_word_str() == Some("была")).expect("word");
    match word {
        Token::Word { clean_key, .. } => assert_eq!(clean_key, "была"),
        _ => unreachable!(),
    }
}