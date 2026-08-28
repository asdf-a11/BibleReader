use crate::domain::models::Token;

pub struct TextTokenizer;

impl TextTokenizer {
    /// Splits text into word and punctuation tokens. Whitespace is skipped.
    /// e.g. `"В начале,"` -> `[Word("В"), Word("начале"), Punctuation(",")]`
    pub fn tokenize(text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current_word = String::new();
        let mut current_punctuation = String::new();

        for c in text.chars() {
            if c.is_alphabetic() || c.is_numeric() {
                if !current_punctuation.is_empty() {
                    tokens.push(Token::Punctuation(current_punctuation.clone()));
                    current_punctuation.clear();
                }
                current_word.push(c);
            } else {
                if !current_word.is_empty() {
                    let clean_key = current_word.to_lowercase();
                    tokens.push(Token::Word {
                        original: current_word.clone(),
                        clean_key,
                    });
                    current_word.clear();
                }
                if !c.is_whitespace() {
                    current_punctuation.push(c);
                }
            }
        }

        if !current_word.is_empty() {
            let clean_key = current_word.to_lowercase();
            tokens.push(Token::Word {
                original: current_word,
                clean_key,
            });
        }

        if !current_punctuation.is_empty() {
            tokens.push(Token::Punctuation(current_punctuation));
        }

        tokens
    }
}