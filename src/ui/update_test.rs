use macroquad::math::{vec2, Vec2};

use crate::domain::{Bible, WordStateStore, WordStatus};
use crate::ui::layout::{
    settings_button_rect, ui_scale, DrawerLayout, ReaderLayout, SettingsLayout, BOOKS_TOP,
    MAX_ZOOM, MIN_ZOOM, READER_FONT_SIZE, READER_LEFT, READER_TOP, ZOOM_STEP,
};
use crate::ui::mouse::Input;
use crate::ui::state::AppState;
use crate::ui::update::{Game, update};

const SCREEN: Vec2 = vec2(1024.0, 768.0);

fn fake_measure(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.6
}

fn raw(chapter: u32, verse: u32, text: &str, book_id: &str, book_name: &str) -> crate::domain::RawVerse {
    crate::domain::RawVerse {
        chapter,
        verse,
        text: text.to_string(),
        translation_id: "RST".to_string(),
        book_id: book_id.to_string(),
        book_name: book_name.to_string(),
    }
}

fn test_bible() -> Bible {
    let verses = vec![
        raw(1, 1, "В начале сотворил Бог небо и землю.", "Gen", "Genesis"),
        raw(1, 2, "И сказал Бог: да будет свет.", "Gen", "Genesis"),
        raw(
            1,
            3,
            "И увидел Бог свет, что он хорош, и отделил Бог свет от тьмы, и назвал Бог свет днём, а тьму ночью, и был вечер, и было утро, день один.",
            "Gen",
            "Genesis",
        ),
        raw(2, 1, "Так совершены небо и земля и все воинство их.", "Gen", "Genesis"),
        raw(1, 1, "Вот имена сынов Израилевых.", "Exod", "Exodus"),
    ];
    Bible::from_raw_verses(verses)
}

/// A Bible with many books so the book list overflows the screen (scrollable).
fn many_books_bible() -> Bible {
    let mut verses = vec![
        raw(1, 1, "В начале сотворил Бог небо и землю.", "Gen", "Genesis"),
        raw(1, 1, "Вот имена сынов Израилевых.", "Exod", "Exodus"),
    ];
    for i in 0..60 {
        let id = format!("Book{}", i);
        let name = format!("Book {}", i);
        verses.push(raw(1, 1, "Стих один.", &id, &name));
    }
    Bible::from_raw_verses(verses)
}

fn game() -> Game {
    Game::new(WordStateStore::new())
}

fn chapter_layout(bible: &Bible, book_id: &str, chapter: u32) -> ReaderLayout {
    let tokens = bible.get_chapter_tokens(book_id, chapter).unwrap();
    ReaderLayout::new(
        &tokens,
        READER_FONT_SIZE,
        READER_LEFT,
        READER_TOP,
        SCREEN.x - 100.0,
        &fake_measure,
    )
}

fn idle() -> Input {
    Input::new(vec2(0.0, 0.0), false, false, false, 0.0)
}

fn press(pos: Vec2) -> Input {
    Input::new(pos, true, true, false, 0.0)
}

fn hold(pos: Vec2) -> Input {
    Input::new(pos, false, true, false, 0.0)
}

fn release(pos: Vec2) -> Input {
    Input::new(pos, false, false, true, 0.0)
}

fn wheel(wheel_y: f32) -> Input {
    Input::new(vec2(0.0, 0.0), false, false, false, wheel_y)
}

// Clicking a book takes you to its chapter menu.
#[test]
fn test_clicking_book_opens_chapter_selection() {
    let bible = test_bible();
    let mut game = game();
    let input = press(vec2(100.0, BOOKS_TOP + 10.0)); // default scroll = 0 => first book (Genesis)

    update(&mut game, &bible, &input, SCREEN, None);

    assert_eq!(
        game.state,
        AppState::ChapterSelection {
            book_id: "Gen".to_string()
        }
    );
}

// Stage 5: a click must register once; holding the button must not re-trigger
// navigation or spam "clicked" events.
#[test]
fn test_holding_click_does_not_re_navigate() {
    let bible = test_bible();
    let mut game = game();
    update(&mut game, &bible, &press(vec2(100.0, 160.0)), SCREEN, None);
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });

    // Keep holding for many frames; nothing more should happen.
    for _ in 0..5 {
        update(&mut game, &bible, &hold(vec2(100.0, 160.0)), SCREEN, None);
        assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });
    }

    update(&mut game, &bible, &release(vec2(100.0, 160.0)), SCREEN, None);
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });
}

#[test]
fn test_clicking_second_book_selects_it_by_scroll_offset() {
    let bible = many_books_bible();
    let mut game = game();

    // Scroll down by one book row, then click the second book.
    update(&mut game, &bible, &wheel(-1.0), SCREEN, None);
    assert!(game.book_scroll.offset > 0.0);

    // Click where "Exodus" now sits (row 1 content position).
    let content_y = BOOKS_TOP + 30.0 * 1.0 + 10.0;
    let click_y = content_y - game.book_scroll.offset;
    update(&mut game, &bible, &press(vec2(100.0, click_y)), SCREEN, None);
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Exod".to_string() });
}

// Stage 5 bug 3: dragging the scrollbar must not click the book underneath.
#[test]
fn test_dragging_scrollbar_does_not_click_a_book() {
    let bible = many_books_bible();
    let mut game = game();

    // Thumb starts at the top of the track (offset == 0).
    let track_x = SCREEN.x - 12.0 - 8.0;
    update(&mut game, &bible, &press(vec2(track_x + 1.0, BOOKS_TOP + 1.0)), SCREEN, None);
    assert!(game.book_scroll.dragging);

    // Move left/off the slider and down while still holding — this must scroll,
    // never select a book.
    for y in [200.0f32, 260.0, 330.0, 400.0] {
        update(&mut game, &bible, &hold(vec2(400.0, y)), SCREEN, None);
        assert_eq!(game.state, AppState::BookSelection, "drag must not navigate to a book");
        assert!(game.book_scroll.dragging);
    }
    assert!(game.book_scroll.offset > 0.0);

    update(&mut game, &bible, &release(vec2(400.0, 400.0)), SCREEN, None);
    assert!(!game.book_scroll.dragging);
    assert_eq!(game.state, AppState::BookSelection);
}

#[test]
fn test_back_button_returns_to_book_selection() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::ChapterSelection { book_id: "Gen".to_string() };

    update(&mut game, &bible, &press(vec2(30.0, 30.0)), SCREEN, None);
    assert_eq!(game.state, AppState::BookSelection);
}

// Back button must take priority over content clicks.
#[test]
fn test_back_button_priority_over_chapter_click() {
    let bible = test_bible();
    let mut game = game();
    // Note: back button and chapter 1 button rects don't overlap; just verify
    // a click on the back button never navigates into the reader.
    game.state = AppState::ChapterSelection { book_id: "Gen".to_string() };
    update(&mut game, &bible, &press(vec2(30.0, 30.0)), SCREEN, None);
    assert_eq!(game.state, AppState::BookSelection);
}

#[test]
fn test_clicking_chapter_button_opens_reader() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::ChapterSelection { book_id: "Gen".to_string() };

    // Chapter 2 center (columns at x=282, 362 with step 80 starting at 150).
    update(&mut game, &bible, &press(vec2(392.0, 180.0)), SCREEN, None);

    assert_eq!(game.state, AppState::Reader { book_id: "Gen".to_string(), chapter: 2 });
}

#[test]
fn test_reader_back_button_returns_to_chapter_menu() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    update(&mut game, &bible, &press(vec2(30.0, 30.0)), SCREEN, Some(&layout));
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });
}

// Reader clicks a word -> translation window shows and STAYS (not 1 frame).
#[test]
fn test_clicking_word_selects_it_and_persists() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("В"));

    // Next frames (mouse idle) must keep the window open.
    update(&mut game, &bible, &idle(), SCREEN, Some(&layout));
    update(&mut game, &bible, &idle(), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("В"));
}

// Clicking the same word twice toggles the translation window closed.
#[test]
fn test_clicking_same_word_twice_hides_window() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("В"));

    // Release then press again on the same word -> hides.
    update(&mut game, &bible, &release(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word, None);
}

// Selecting a different word switches the window to that word.
#[test]
fn test_clicking_different_word_switches_translation() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("В"));

    // "начале" is the second word (rect x starts ~72).
    update(&mut game, &bible, &press(vec2(80.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("начале"));
}

// Stage 2 rule: an Unseen word becomes Seen the first time it is opened.
#[test]
fn test_opening_unseen_word_marks_it_seen() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    assert_eq!(game.word_store.get_status("начале"), WordStatus::Unseen);
    update(&mut game, &bible, &press(vec2(80.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.word_store.get_status("начале"), WordStatus::Seen);
}

// Drawer status buttons change the word's saved state.
#[test]
fn test_drawer_status_button_updates_word_state() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    // Open the drawer on "В".
    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));

    // "Known" button center on a 1024x768 screen.
    update(&mut game, &bible, &press(vec2(622.0, 718.0)), SCREEN, Some(&layout));
    assert_eq!(game.word_store.get_status("В"), WordStatus::Known);
    // The window stays on the same word.
    assert_eq!(game.selected_word.as_deref(), Some("В"));
}

// Reader scrolling via the mouse wheel.
#[test]
fn test_reader_wheel_scrolls_content() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    // Use a very narrow measure so the chapter wraps into far more lines than
    // fit on screen, guaranteeing there is something to scroll.
    let tokens = bible.get_chapter_tokens("Gen", 1).unwrap();
    let layout = ReaderLayout::new(
        &tokens,
        READER_FONT_SIZE,
        READER_LEFT,
        READER_TOP,
        60.0,
        &fake_measure,
    );

    assert_eq!(game.reader_scroll.offset, 0.0);
    update(&mut game, &bible, &wheel(-1.0), SCREEN, Some(&layout));
    assert!(game.reader_scroll.offset > 0.0, "reader must scroll with the wheel");
}

// Reader back button takes priority over word selection.
#[test]
fn test_reader_back_button_not_overridden_by_word_click() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Exod".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Exod", 1);

    // The back button and the first word are in different places, but the
    // update must process the back button regardless of the word store state.
    update(&mut game, &bible, &press(vec2(30.0, 30.0)), SCREEN, Some(&layout));
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Exod".to_string() });
    assert_eq!(game.selected_word, None);
}

// Word store changes are reported so the loop can persist them.
#[test]
fn test_update_reports_dirty_when_word_state_changes() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    let dirty = update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert!(dirty, "opening a word must mark the store dirty for saving");
}

// Close button in the translation window hides it without touching word state.
#[test]
fn test_close_button_hides_translation_window() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::Reader { book_id: "Gen".to_string(), chapter: 1 };
    let layout = chapter_layout(&bible, "Gen", 1);

    // Open the drawer on "В".
    update(&mut game, &bible, &press(vec2(55.0, 105.0)), SCREEN, Some(&layout));
    assert_eq!(game.selected_word.as_deref(), Some("В"));

    let drawer = DrawerLayout::new(SCREEN.x, SCREEN.y, 1.0);
    update(&mut game, &bible, &press(drawer.close_button.center()), SCREEN, Some(&layout));

    assert_eq!(game.selected_word, None, "close button must hide the translation window");
    assert_eq!(game.word_store.get_status("В"), WordStatus::Seen, "word stays Seen, not lost");
    assert_eq!(game.state, AppState::Reader { book_id: "Gen".to_string(), chapter: 1 });
}

// The settings button on any normal screen opens the settings menu.
#[test]
fn test_settings_button_opens_settings_menu() {
    let bible = test_bible();
    let mut game = game();

    let button = settings_button_rect(SCREEN.x, 1.0);
    update(&mut game, &bible, &press(button.center()), SCREEN, None);

    match &game.state {
        AppState::Settings { return_to } => {
            assert_eq!(**return_to, AppState::BookSelection, "must remember the previous screen");
        }
        other => panic!("expected Settings, got {:?}", other),
    }
}

// Settings works from the reader too and returns to it afterwards.
#[test]
fn test_settings_back_button_returns_to_previous_screen() {
    let bible = test_bible();
    let mut game = game();
    game.state = AppState::ChapterSelection { book_id: "Gen".to_string() };

    let button = settings_button_rect(SCREEN.x, 1.0);
    update(&mut game, &bible, &press(button.center()), SCREEN, None);
    assert!(matches!(game.state, AppState::Settings { .. }));

    update(&mut game, &bible, &press(vec2(30.0, 30.0)), SCREEN, None);
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });
}

// Zoom in/out buttons change the zoom, clamped to the allowed range.
#[test]
fn test_zoom_in_out_buttons_change_zoom_within_bounds() {
    let bible = test_bible();
    let mut game = game();

    let button = settings_button_rect(SCREEN.x, 1.0);
    update(&mut game, &bible, &press(button.center()), SCREEN, None);
    assert_eq!(game.zoom, 1.0);

    for expected in [1.25f32, 1.5, 1.75, 2.0] {
        let layout = SettingsLayout::new(SCREEN, ui_scale(SCREEN, game.zoom));
        update(&mut game, &bible, &press(layout.zoom_in.center()), SCREEN, None);
        assert_eq!(game.zoom, expected, "zoom in must step by {} and hit {}", ZOOM_STEP, expected);
        assert!(matches!(game.state, AppState::Settings { .. }), "still in settings");
    }

    // Clamped at the maximum.
    let layout = SettingsLayout::new(SCREEN, ui_scale(SCREEN, game.zoom));
    update(&mut game, &bible, &press(layout.zoom_in.center()), SCREEN, None);
    assert_eq!(game.zoom, MAX_ZOOM);

    // Zoom back down, clamped at the minimum.
    for _ in 0..100 {
        let layout = SettingsLayout::new(SCREEN, ui_scale(SCREEN, game.zoom));
        update(&mut game, &bible, &press(layout.zoom_out.center()), SCREEN, None);
    }
    assert_eq!(game.zoom, MIN_ZOOM);
}

// Every screen handler keeps working when zoomed: geometry is hit-tested in
// scaled coordinates.
#[test]
fn test_zoom_scales_hit_test_coordinates() {
    let bible = test_bible();
    let mut game2 = game();
    game2.zoom = 2.0;
    // At zoom 2 the y that selects a book at zoom 1 is above the list.
    update(&mut game2, &bible, &press(vec2(100.0, BOOKS_TOP + 10.0)), SCREEN, None);
    assert_eq!(game2.state, AppState::BookSelection, "click above the list start must no-op");

    let mut game = game();
    game.zoom = 2.0;
    // First book row starts at BOOKS_TOP * 2 = 300, rows are 60px tall.
    update(
        &mut game,
        &bible,
        &press(vec2(100.0, BOOKS_TOP * 2.0 + 10.0)),
        SCREEN,
        None,
    );
    assert_eq!(game.state, AppState::ChapterSelection { book_id: "Gen".to_string() });
}