use crate::domain::{TextTokenizer, Token};

#[test]
fn test_tokenize_simple_sentence() {
    let tokens = TextTokenizer::tokenize("В начале");
    assert_eq!(tokens.len(), 2);
    match &tokens[0] {
        Token::Word {
            original,
            clean_key,
        } => {
            assert_eq!(original, "В");
            assert_eq!(clean_key, "в");
        }
        _ => panic!("Expected Word token"),
    }
    match &tokens[1] {
        Token::Word {
            original,
            clean_key,
        } => {
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
    let word = tokens
        .iter()
        .find(|t| t.as_word_str() == Some("была"))
        .expect("word");
    match word {
        Token::Word { clean_key, .. } => assert_eq!(clean_key, "была"),
        _ => unreachable!(),
    }
}

#[test]
fn test_split_sentences_at_sentence_ends() {
    assert_eq!(
        TextTokenizer::split_sentences("И сказал Бог: да будет свет. И стал свет."),
        vec!["И сказал Бог: да будет свет.", "И стал свет."]
    );
    assert_eq!(
        TextTokenizer::split_sentences("зачем ты обманул меня? не за Рахиль ли!  Аминь…  да"),
        vec!["зачем ты обманул меня?", "не за Рахиль ли!", "Аминь…", "да"]
    );
}

#[test]
fn test_split_sentences_keeps_closing_quotes_and_unfinished_verses() {
    assert_eq!(
        TextTokenizer::split_sentences("сказал: «иди сюда.» Он пошел,"),
        vec!["сказал: «иди сюда.»", "Он пошел,"]
    );
    assert_eq!(TextTokenizer::split_sentences("Сыны Рувима,"), vec!["Сыны Рувима,"]);
    // A full stop with no space after it (an abbreviation, say) does not end
    // a sentence.
    assert_eq!(TextTokenizer::split_sentences("в 3.5 локтя"), vec!["в 3.5 локтя"]);
    assert!(TextTokenizer::split_sentences("   ").is_empty());
}
