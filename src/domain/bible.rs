use std::collections::HashMap;

use crate::domain::models::{Bible, Book, Chapter, RawVerse, Token, Verse};
use crate::domain::TextTokenizer;

impl Bible {
    /// Build the hierarchy from verses in file order.
    ///
    /// The `book_order` vector records the order in which a book is first seen,
    /// so books are kept in the same order as they appear in `rst.json`
    /// (Genesis, Exodus, ...) rather than being re-sorted.
    pub fn from_raw_verses(verses: Vec<RawVerse>) -> Self {
        let mut books: HashMap<String, Book> = HashMap::new();
        let mut book_order: Vec<String> = Vec::new();

        for verse in verses {
            let book_id = verse.book_id.clone();
            if !books.contains_key(&book_id) {
                book_order.push(book_id.clone());
            }

            books
                .entry(book_id)
                .and_modify(|book| append_verse(book, &verse))
                .or_insert_with(|| Book::from_verse(&verse));
        }

        Bible { books, book_order }
    }

    pub fn get_book(&self, book_id: &str) -> Option<&Book> {
        self.books.get(book_id)
    }

    /// Books ordered as they appear in `rst.json` (Genesis first, then Exodus, ...).
    pub fn get_books_in_order(&self) -> Vec<&Book> {
        self.book_order
            .iter()
            .filter_map(|book_id| self.books.get(book_id))
            .collect()
    }

    /// The full, tokenized text of a chapter (all verses joined together).
    pub fn get_chapter_tokens(&self, book_id: &str, chapter: u32) -> Option<Vec<Token>> {
        Some(TextTokenizer::tokenize(
            &self.get_chapter_text(book_id, chapter)?,
        ))
    }

    pub fn get_chapter_text(&self, book_id: &str, chapter: u32) -> Option<String> {
        let book = self.get_book(book_id)?;
        let chapter_data = book.chapters.get(&chapter)?;
        Some(join_verses(chapter_data))
    }
}

impl Book {
    pub fn chapter_numbers(&self) -> Vec<u32> {
        let mut numbers: Vec<u32> = self.chapters.keys().copied().collect();
        numbers.sort_unstable();
        numbers
    }

    fn from_verse(verse: &RawVerse) -> Self {
        let mut chapters = HashMap::new();
        chapters.insert(
            verse.chapter,
            Chapter {
                number: verse.chapter,
                verses: vec![Verse::from_raw(verse)],
            },
        );
        Book {
            id: verse.book_id.clone(),
            name: verse.book_name.clone(),
            chapters,
        }
    }
}

impl Verse {
    fn from_raw(verse: &RawVerse) -> Self {
        Verse {
            number: verse.verse,
            text: verse.text.clone(),
            tokens: Vec::new(),
        }
    }
}

pub(crate) fn join_verses(chapter: &Chapter) -> String {
    chapter
        .verses
        .iter()
        .map(|v| v.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn append_verse(book: &mut Book, verse: &RawVerse) {
    book.chapters
        .entry(verse.chapter)
        .and_modify(|chapter| chapter.verses.push(Verse::from_raw(verse)))
        .or_insert_with(|| Chapter {
            number: verse.chapter,
            verses: vec![Verse::from_raw(verse)],
        });
}
