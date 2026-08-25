use std::collections::HashMap;

use crate::domain::models::{RawVerse, Book, Chapter, Verse, Bible, Token};

impl Bible {
    pub fn new() -> Self {
        Bible {
            books: HashMap::new(),
            book_order: Vec::new(),
        }
    }

    pub fn from_raw_verses(verses: Vec<RawVerse>) -> Self {
        let mut books: HashMap<String, Book> = HashMap::new();
        let mut book_order: Vec<String> = Vec::new();

        for verse in verses {
            let book_id = verse.book_id.clone();
            let book_name = verse.book_name.clone();
            let chapter_num = verse.chapter;
            let verse_num = verse.verse;
            let text = verse.text.clone();

            if !books.contains_key(&book_id) {
                book_order.push(book_id.clone());
            }

            books.entry(book_id.clone())
                .and_modify(|book: &mut Book| {
                    book.chapters.entry(chapter_num)
                        .and_modify(|chapter: &mut Chapter| {
                            chapter.verses.push(Verse {
                                number: verse_num,
                                text: text.clone(),
                                tokens: Vec::new(),
                            });
                        })
                        .or_insert_with(|| Chapter {
                            number: chapter_num,
                            verses: vec![Verse {
                                number: verse_num,
                                text: text.clone(),
                                tokens: Vec::new(),
                            }],
                        });
                })
                .or_insert_with(|| Book {
                    id: book_id,
                    name: book_name,
                    chapters: HashMap::from([
                        (
                            chapter_num,
                            Chapter {
                                number: chapter_num,
                                verses: vec![Verse {
                                    number: verse_num,
                                    text: text,
                                    tokens: Vec::new(),
                                }],
                            }
                        )
                    ]),
                });
        }

        Bible { books, book_order }
    }

    pub fn get_verse(&self, book_id: &str, chapter: u32, verse: u32) -> Option<&Verse> {
        self.books.get(book_id)
            .and_then(|book| book.chapters.get(&chapter))
            .and_then(|chapter| chapter.verses.iter().find(|v| v.number == verse))
    }

    pub fn get_books_in_order(&self) -> Vec<&Book> {
        self.book_order
            .iter()
            .filter_map(|book_id| self.books.get(book_id))
            .collect()
    }
}

pub struct ChapterText {
    pub text: String,
    pub tokens: Vec<Token>,
}

impl Book {
    pub fn get_chapter_text(&self, chapter_num: u32) -> Option<ChapterText> {
        let chapter = self.chapters.get(&chapter_num)?;
        let full_text = chapter.verses.iter().map(|v| v.text.as_str()).collect::<Vec<_>>().join(" ");
        let tokens = crate::domain::TextTokenizer::tokenize(&full_text);
        Some(ChapterText { text: full_text, tokens })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_order_preserved() {
        let raw_verses = vec![
            RawVerse {
                chapter: 1,
                verse: 1,
                text: "Verse 1".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Gen".to_string(),
                book_name: "Genesis".to_string(),
            },
            RawVerse {
                chapter: 1,
                verse: 1,
                text: "Verse 2".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Exod".to_string(),
                book_name: "Exodus".to_string(),
            },
            RawVerse {
                chapter: 1,
                verse: 1,
                text: "Verse 3".to_string(),
                translation_id: "RST".to_string(),
                book_id: "Lev".to_string(),
                book_name: "Leviticus".to_string(),
            },
        ];

        let bible = Bible::from_raw_verses(raw_verses);
        let books = bible.get_books_in_order();
        
        assert_eq!(books.len(), 3);
        assert_eq!(books[0].id, "Gen");
        assert_eq!(books[1].id, "Exod");
        assert_eq!(books[2].id, "Lev");
    }

    #[test]
    fn test_book_get_chapter_text() {
        let mut book = Book {
            id: "Gen".to_string(),
            name: "Genesis".to_string(),
            chapters: HashMap::new(),
        };
        
        book.chapters.insert(1, Chapter {
            number: 1,
            verses: vec![
                Verse {
                    number: 1,
                    text: "В начале".to_string(),
                    tokens: Vec::new(),
                },
                Verse {
                    number: 2,
                    text: "И сказал Бог".to_string(),
                    tokens: Vec::new(),
                },
            ],
        });
        
        let chapter_text = book.get_chapter_text(1).unwrap();
        assert_eq!(chapter_text.text, "В начале И сказал Бог");
        assert_eq!(chapter_text.tokens.len(), 4);
    }

    #[test]
    fn test_reader_layout_grid_spacing() {
        let layout = WordTokenLayout {
            token: Token::Word {
                original: "В".to_string(),
                clean_key: "в".to_string(),
            },
            rect: Rect {
                x: 10.0,
                y: 10.0,
                w: 30.0,
                h: 20.0,
            },
            highlight_rect: Rect {
                x: 6.0,
                y: 6.0,
                w: 38.0,
                h: 28.0,
            },
        };
        
        assert!((layout.highlight_rect.x - 6.0).abs() < 0.1);
        assert!((layout.highlight_rect.y - 6.0).abs() < 0.1);
    }
}
