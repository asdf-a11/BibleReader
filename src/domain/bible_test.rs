use crate::domain::{Bible, RawVerse};

fn raw(chapter: u32, verse: u32, text: &str, book_id: &str, book_name: &str) -> RawVerse {
    RawVerse {
        chapter,
        verse,
        text: text.to_string(),
        translation_id: "RST".to_string(),
        book_id: book_id.to_string(),
        book_name: book_name.to_string(),
    }
}

fn genesis_chapter_1() -> Vec<RawVerse> {
    vec![
        raw(
            1,
            1,
            "В начале сотворил Бог небо и землю.",
            "Gen",
            "Genesis",
        ),
        raw(1, 2, "Земля же была безвидна и пуста.", "Gen", "Genesis"),
        raw(1, 3, "И сказал Бог: да будет свет.", "Gen", "Genesis"),
    ]
}

// Stage 5 bug 1: books must appear in the order they appear in rst.json.
#[test]
fn test_books_preserve_rst_file_order() {
    let mut verses = genesis_chapter_1();
    verses.extend([
        raw(1, 1, "Вот имена сынов Израилевых.", "Exod", "Exodus"),
        raw(1, 2, "И сказал Господь Моисею.", "Exod", "Exodus"),
    ]);

    let bible = Bible::from_raw_verses(verses);
    let order: Vec<&str> = bible
        .get_books_in_order()
        .iter()
        .map(|b| b.id.as_str())
        .collect();
    assert_eq!(order, vec!["Gen", "Exod"], "books must keep rst.json order");

    let names: Vec<&str> = bible
        .get_books_in_order()
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(names, vec!["Genesis", "Exodus"]);
}

#[test]
fn test_book_order_is_not_sorted_alphabetically() {
    // Repeated book ids must not re-append or reorder.
    let verses = vec![
        raw(1, 1, "...", "James", "James"),
        raw(1, 1, "...", "Acts", "Acts"),
        raw(1, 2, "...", "James", "James"),
        raw(1, 1, "...", "Revelation", "Revelation"),
    ];
    let bible = Bible::from_raw_verses(verses);
    let order: Vec<&str> = bible
        .get_books_in_order()
        .iter()
        .map(|b| b.id.as_str())
        .collect();
    assert_eq!(order, vec!["James", "Acts", "Revelation"]);
}

#[test]
fn test_hierarchy_building_groups_chapters_and_verses() {
    let verses = vec![
        raw(1, 1, "Verse 1", "Gen", "Genesis"),
        raw(1, 2, "Verse 2", "Gen", "Genesis"),
        raw(2, 1, "Verse 3", "Gen", "Genesis"),
    ];
    let bible = Bible::from_raw_verses(verses);

    let gen = bible.get_book("Gen").expect("Genesis present");
    assert_eq!(gen.chapters.len(), 2);
    assert_eq!(gen.chapters.get(&1).unwrap().verses.len(), 2);
    assert_eq!(gen.chapters.get(&2).unwrap().verses.len(), 1);
}

#[test]
fn test_chapter_numbers_are_sorted() {
    let verses = vec![
        raw(3, 1, "c3", "Gen", "Genesis"),
        raw(1, 1, "c1", "Gen", "Genesis"),
        raw(2, 1, "c2", "Gen", "Genesis"),
    ];
    let bible = Bible::from_raw_verses(verses);
    let numbers = bible.get_book("Gen").unwrap().chapter_numbers();
    assert_eq!(numbers, vec![1, 2, 3]);
}

// Stage 5/6 bug: the reader must show the ENTIRE chapter, not just the first verse.
#[test]
fn test_get_chapter_tokens_join_all_verses() {
    let verses = genesis_chapter_1();
    let bible = Bible::from_raw_verses(verses);

    let tokens = bible.get_chapter_tokens("Gen", 1).expect("chapter tokens");

    // Every word from all three verses must be present.
    for expected in [
        "В",
        "начале",
        "сотворил",
        "Бог",
        "Земля",
        "сказал",
        "будет",
        "свет",
    ] {
        assert!(
            tokens.iter().any(|t| t.as_word_str() == Some(expected)),
            "expected word '{expected}' in chapter text"
        );
    }

    // The chapter is more than just the first verse.
    let words: Vec<&str> = tokens.iter().filter_map(|t| t.as_word_str()).collect();
    let first_verse_words = ["В", "начале", "сотворил", "Бог", "небо", "и", "землю"];
    assert!(
        words.len() > first_verse_words.len(),
        "chapter must contain more than the first verse"
    );
}

#[test]
fn test_get_chapter_text_joins_verses_with_space() {
    let verses = genesis_chapter_1();
    let bible = Bible::from_raw_verses(verses);
    let text = bible.get_chapter_text("Gen", 1).unwrap();
    assert!(text.starts_with("В начале сотворил Бог небо и землю."));
    assert!(text.ends_with("И сказал Бог: да будет свет."));
    assert!(text.contains("безвидна и пуста"));
}

#[test]
fn test_chapter_sentences_are_numbered_within_their_verse() {
    let bible = Bible::from_raw_verses(vec![
        raw(1, 1, "В начале сотворил Бог небо и землю.", "Gen", "Genesis"),
        raw(1, 2, "И сказал Бог: да будет свет. И стал свет.", "Gen", "Genesis"),
    ]);
    let sentences = bible.get_chapter_sentences("Gen", 1).unwrap();
    let summary: Vec<_> = sentences
        .iter()
        .map(|s| (s.verse, s.index, s.text.as_str()))
        .collect();
    assert_eq!(
        summary,
        vec![
            (1, 0, "В начале сотворил Бог небо и землю."),
            (2, 0, "И сказал Бог: да будет свет."),
            (2, 1, "И стал свет."),
        ]
    );
    assert!(bible.get_chapter_sentences("Gen", 2).is_none());
}
