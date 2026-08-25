use crate::domain::models::Token;

pub struct TextTokenizer;

impl TextTokenizer {
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
                    tokens.push(Token::Word { original: current_word.clone(), clean_key });
                    current_word.clear();
                }
                if !c.is_whitespace() {
                    current_punctuation.push(c);
                }
            }
        }

        if !current_word.is_empty() {
            let clean_key = current_word.to_lowercase();
            tokens.push(Token::Word { original: current_word, clean_key });
        }

        if !current_punctuation.is_empty() {
            tokens.push(Token::Punctuation(current_punctuation));
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Token::Punctuation(p) => {
                assert_eq!(p, ",");
            }
            _ => panic!("Expected Punctuation token"),
        }
    }

    #[test]
    fn test_tokenize_mixed_content() {
        let tokens = TextTokenizer::tokenize("В начале, сотворил Бог...");
        assert_eq!(tokens.len(), 6);
        
        match &tokens[0] {
            Token::Word { original, .. } => assert_eq!(original, "В"),
            _ => panic!("Expected Word token"),
        }
        
        match &tokens[3] {
            Token::Word { original, .. } => assert_eq!(original, "сотворил"),
            _ => panic!("Expected Word token"),
        }
        
        match &tokens[5] {
            Token::Punctuation(p) => assert_eq!(p, "..."),
            _ => panic!("Expected Punctuation token"),
        }
    }
}
