use crate::domain::models::Token;

pub struct TextTokenizer;

/// Characters that end a sentence.
const SENTENCE_END: [char; 4] = ['.', '!', '?', '…'];
/// Closing quotes and brackets that stay with the sentence they close.
const SENTENCE_CLOSERS: [char; 6] = ['»', '”', '"', '’', ')', ']'];

impl TextTokenizer {
    /// Splits a verse into sentences: a sentence ends at `.`, `!`, `?` or `…`
    /// (plus any closing quotes) followed by a space. Runs of whitespace are
    /// collapsed to a single space.
    ///
    /// This must match `split_sentences` in `generate_verse_translations.py`:
    /// the translations are looked up by sentence index and checked against
    /// the sentence text.
    pub fn split_sentences(text: &str) -> Vec<String> {
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let chars: Vec<char> = text.chars().collect();
        let mut sentences = Vec::new();
        let mut current = String::new();
        let mut i = 0;

        while i < chars.len() {
            if !SENTENCE_END.contains(&chars[i]) {
                current.push(chars[i]);
                i += 1;
                continue;
            }
            while i < chars.len() && SENTENCE_END.contains(&chars[i]) {
                current.push(chars[i]);
                i += 1;
            }
            while i < chars.len() && SENTENCE_CLOSERS.contains(&chars[i]) {
                current.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == ' ' {
                sentences.push(std::mem::take(&mut current));
                i += 1;
            }
        }

        if !current.is_empty() {
            sentences.push(current);
        }
        sentences
    }

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
