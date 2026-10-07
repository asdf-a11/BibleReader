//! Tests for the state machine in [`App`].
//!
//! These cover the behaviour the GUI depends on but cannot easily show: screen
//! transitions, word status bookkeeping and persistence.

use std::collections::HashMap;

use crate::app::{App, AppData};
use crate::domain::{Bible, FileStore, MemoryStore, RawVerse, Translations, WordStatus};
use crate::ui::layout::{MAX_ZOOM, MIN_ZOOM};
use crate::ui::state::AppState;

/// A store in the temp directory, unique per test name and empty to start.
fn temp_store(name: &str) -> FileStore {
    let mut path = std::env::temp_dir();
    path.push(name);
    let _ = std::fs::remove_file(&path);
    FileStore::new(path.to_str().expect("temp path is valid utf-8"))
}

const VERSES: [&str; 4] = [
    r#"{"chapter":1,"verse":1,"text":"В начале","translation_id":"rst","book_id":"gen","book_name":"Бытие"}"#,
    r#"{"chapter":1,"verse":2,"text":"сотворил Бог","translation_id":"rst","book_id":"gen","book_name":"Бытие"}"#,
    r#"{"chapter":2,"verse":1,"text":"И сказал Бог","translation_id":"rst","book_id":"gen","book_name":"Бытие"}"#,
    r#"{"chapter":2,"verse":2,"text":"небо и землю","translation_id":"rst","book_id":"gen","book_name":"Бытие"}"#,
];

const TRANSLATIONS: &str = r#"{"translations":[
    {"word":"бог","closestWords":["God", "deity"]},
    {"word":"небо","closestWords":["sky", "heaven"]}
]}"#;

fn verse(json: &str) -> RawVerse {
    serde_json::from_str(json).expect("test verse should parse")
}

fn test_data() -> AppData {
    let bible = Bible::from_raw_verses(VERSES.iter().map(|json| verse(json)).collect());
    let translations = Translations::from_json(TRANSLATIONS).expect("translations should parse");
    AppData {
        bible,
        translations,
        load_error: None,
    }
}

fn test_app() -> App {
    App::with_store(test_data(), Box::new(MemoryStore::new()))
}

fn app_with_progress(states: HashMap<String, WordStatus>) -> App {
    App::with_store(test_data(), Box::new(MemoryStore::with_states(states)))
}

#[test]
fn starts_on_the_book_list() {
    let app = test_app();
    assert_eq!(app.state, AppState::BookSelection);
    assert_eq!(app.state.title(), "Books");
    assert!(app.selected_word.is_none());
}

#[test]
fn books_keep_their_order_from_the_bible_file() {
    let app = test_app();
    let names: Vec<&str> = app
        .data
        .bible
        .get_books_in_order()
        .iter()
        .map(|book| book.name.as_str())
        .collect();
    assert_eq!(names, vec!["Бытие"]);
}

#[test]
fn opening_a_book_then_a_chapter_reaches_the_reader() {
    let mut app = test_app();
    app.open_book("gen");
    assert_eq!(
        app.state,
        AppState::ChapterSelection {
            book_id: "gen".into()
        }
    );
    app.open_chapter(2);
    assert_eq!(
        app.state,
        AppState::Reader {
            book_id: "gen".into(),
            chapter: 2
        }
    );
    assert_eq!(app.state.title(), "Chapter 2");
}

#[test]
fn opening_a_chapter_outside_the_chapter_list_is_ignored() {
    let mut app = test_app();
    app.open_chapter(1);
    assert_eq!(app.state, AppState::BookSelection);
}

#[test]
fn back_walks_reader_chapter_book() {
    let mut app = test_app();
    app.open_book("gen");
    app.open_chapter(1);
    app.go_back();
    assert_eq!(
        app.state,
        AppState::ChapterSelection {
            book_id: "gen".into()
        }
    );
    app.go_back();
    assert_eq!(app.state, AppState::BookSelection);
}

#[test]
fn leaving_the_reader_closes_the_drawer() {
    let mut app = test_app();
    app.open_book("gen");
    app.open_chapter(1);
    app.toggle_word("бог");
    assert_eq!(app.selected_word.as_deref(), Some("бог"));

    app.go_back();
    assert!(app.selected_word.is_none());
}

#[test]
fn settings_returns_to_the_screen_it_was_opened_from() {
    let mut app = test_app();
    app.open_book("gen");
    app.open_chapter(1);
    app.open_settings();
    assert!(matches!(app.state, AppState::Settings { .. }));
    assert_eq!(app.state.title(), "Settings");

    app.go_back();
    assert_eq!(
        app.state,
        AppState::Reader {
            book_id: "gen".into(),
            chapter: 1
        }
    );
}

#[test]
fn opening_settings_twice_does_not_lose_the_return_screen() {
    let mut app = test_app();
    app.open_book("gen");
    app.open_settings();
    app.open_settings();
    app.go_back();
    assert_eq!(
        app.state,
        AppState::ChapterSelection {
            book_id: "gen".into()
        }
    );
}

#[test]
fn tapping_a_word_opens_the_drawer_and_marks_it_seen() {
    let mut app = test_app();
    assert_eq!(app.status_of("бог"), WordStatus::Unseen);

    app.toggle_word("бог");
    assert_eq!(app.selected_word.as_deref(), Some("бог"));
    assert_eq!(app.status_of("бог"), WordStatus::Seen);
}

#[test]
fn tapping_the_open_word_closes_the_drawer_and_keeps_the_status() {
    let mut app = test_app();
    app.toggle_word("бог");
    app.toggle_word("бог");
    assert!(app.selected_word.is_none());
    assert_eq!(app.status_of("бог"), WordStatus::Seen);
}

#[test]
fn tapping_another_word_switches_the_drawer() {
    let mut app = test_app();
    app.toggle_word("бог");
    app.toggle_word("небо");
    assert_eq!(app.selected_word.as_deref(), Some("небо"));
    assert_eq!(app.status_of("небо"), WordStatus::Seen);
}

#[test]
fn closing_the_drawer_keeps_the_status() {
    let mut app = test_app();
    app.toggle_word("бог");
    app.close_drawer();
    assert!(app.selected_word.is_none());
    assert_eq!(app.status_of("бог"), WordStatus::Seen);
}

#[test]
fn the_status_buttons_overwrite_the_automatic_seen_state() {
    let mut app = test_app();
    app.toggle_word("бог");
    app.set_selected_status(WordStatus::Known);
    assert_eq!(app.status_of("бог"), WordStatus::Known);

    // Re-opening a known word must not knock it back down to "seen".
    app.toggle_word("бог");
    app.set_selected_status(WordStatus::Known);
    assert_eq!(app.status_of("бог"), WordStatus::Known);
}

#[test]
fn setting_a_status_without_an_open_drawer_does_nothing() {
    let mut app = test_app();
    app.set_selected_status(WordStatus::Known);
    assert_eq!(app.status_of("бог"), WordStatus::Unseen);
}

#[test]
fn status_lookup_is_case_insensitive() {
    let mut app = test_app();
    app.toggle_word("Бог");
    assert_eq!(app.status_of("бог"), WordStatus::Seen);
    assert_eq!(app.status_of("БОГ"), WordStatus::Seen);
}

#[test]
fn saved_progress_is_restored_on_startup() {
    let mut states = HashMap::new();
    states.insert("бог".to_string(), WordStatus::Known);
    let app = app_with_progress(states);
    assert_eq!(app.status_of("бог"), WordStatus::Known);
    assert!(!app.word_store.is_dirty(), "loading must not queue a write");
}

#[test]
fn progress_is_written_out_once_it_changes() {
    let store = temp_store("bible_reader_persist_states.json");
    let path = store.path().to_string();

    let mut app = App::with_store(test_data(), Box::new(store));
    app.toggle_word("бог");
    assert!(app.save_error.is_none());
    app.persist();

    // The dirty flag is cleared, so a second frame does not rewrite the file.
    assert!(!app.word_store.is_dirty());

    // A fresh app over the same file sees the saved progress.
    let reloaded = App::with_store(test_data(), Box::new(FileStore::new(&path)));
    assert_eq!(reloaded.status_of("бог"), WordStatus::Seen);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_broken_state_file_does_not_stop_the_app_from_starting() {
    let store = temp_store("bible_reader_broken_states.json");
    let path = store.path().to_string();
    std::fs::write(&path, "this is not json").unwrap();

    let app = App::with_store(test_data(), Box::new(store));
    assert_eq!(app.status_of("бог"), WordStatus::Unseen);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn zoom_stops_at_the_limits() {
    let mut app = test_app();
    for _ in 0..100 {
        app.zoom_in();
    }
    assert_eq!(app.zoom, MAX_ZOOM);

    for _ in 0..100 {
        app.zoom_out();
    }
    assert_eq!(app.zoom, MIN_ZOOM);
}

#[test]
fn a_missing_bible_file_is_reported_instead_of_panicking() {
    let mut app = App::with_store(AppData::load(), Box::new(MemoryStore::new()));
    // The real data files are read from the working directory; when they are
    // absent the app must still come up and say so.
    if app.data.load_error.is_some() {
        assert!(app.data.bible.book_order.is_empty());
    }
    app.open_book("gen");
    app.open_chapter(1);
}
