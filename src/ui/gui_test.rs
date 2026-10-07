//! End-to-end GUI tests, driven through `egui_kittest`.
//!
//! These run the real [`AppShell`] headlessly, so they cover the bugs the user
//! actually sees: words running off the right edge, taps not behaving like
//! mouse clicks, the missing scroll bar, the last line hiding behind the
//! translation drawer, and drag-to-scroll on a touch screen.

use egui::accesskit::Role;
use egui::{pos2, Pos2, Rect, Vec2};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use crate::app::{App, AppData, AppShell};
use crate::domain::{Bible, MemoryStore, RawVerse, Translations, WordStatus};
use crate::ui::layout::READER_BOTTOM_PADDING;
use crate::ui::reader;
use crate::ui::state::AppState;

/// A long Russian chapter, comfortably taller than the test window so that
/// there is something to scroll.
const CHAPTER: &str = "\
В начале сотворил Бог небо и землю, и земля была безводна и пуста, и тьма была над \
бездною, и Дух Божий носился над водою. И сказал Бог да будет свет, и стал свет. \
И увидел Бог свет, что он хорош, и отделил Бог свет от тьмы. И нарек Бог свет днем, \
а тьму нарек ночью. И был вечер, и был утро, день единственный. И сказал Бог да \
будет твердь посреди воды, и да будет она отделять воду от воды. И сотворил Бог \
твердь, и отделил воду, которая под твердью, от воды, которая над твердью, и так \
сталось. И нарек Бог твердь небом, а землю нарек сушею, а воды собрал в моря, и \
увидел Бог, что это хорошо. И сказал Бог да произведет земля траву, зеленую и \
траву, сеющую семя, и дерево плодовитое, делающее плод по роду его, семя которого \
в нем на земле. И сотворил Бог землю, и произведе ее, и произведе земля траву, \
и дерево плодовитое. И было вечер, и было утро, день второй. И сказал Бог да \
будут светила на тверди небесной, чтобы отделять день от ночи, и чтобы были знамена, \
и времена, и дни, и годы. И сотворил Бог два светила великих, светило большее, \
чтобы управлять днем, и светило меньшее, чтобы управлять ночью, и звезды.";

const TRANSLATIONS: &str = r#"{"translations":[
    {"word":"сотворил","closestWords":["created", "made"]},
    {"word":"землю","closestWords":["earth", "land"]},
    {"word":"свет","closestWords":["light"]},
    {"word":"вода","closestWords":["water"]},
    {"word":"тьма","closestWords":["darkness", "gloom"]},
    {"word":"небо","closestWords":["sky", "heaven"]},
    {"word":"бог","closestWords":["God", "deity"]},
    {"word":"день","closestWords":["day"]},
    {"word":"ночь","closestWords":["night"]}
]}"#;

const WINDOW: Vec2 = Vec2::new(900.0, 600.0);

fn verse(chapter: u32, verse: u32, text: &str) -> RawVerse {
    RawVerse {
        chapter,
        verse,
        text: text.to_string(),
        translation_id: "rst".to_string(),
        book_id: "gen".to_string(),
        book_name: "Бытие".to_string(),
    }
}

fn test_data() -> AppData {
    AppData {
        bible: Bible::from_raw_verses(vec![verse(1, 1, CHAPTER), verse(2, 1, "Второй")]),
        translations: Translations::from_json(TRANSLATIONS).expect("translations parse"),
        load_error: None,
    }
}

type Test = Harness<'static, AppShell>;

/// A harness sitting on the book list.
fn harness() -> Test {
    Harness::builder()
        .with_size(WINDOW)
        // One simulated frame is one real frame. The default (a quarter of a
        // second) is so coarse that a three frame tap is mistaken for a long
        // press, which is exactly the kind of thing these tests must not do.
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(60)
        .build_eframe(|cc| AppShell::with_data(cc, test_data(), Box::new(MemoryStore::new())))
}

/// A harness already showing chapter 1.
fn reader() -> Test {
    let mut harness = harness();
    harness.state_mut().app_mut().open_book("gen");
    harness.state_mut().app_mut().open_chapter(1);
    harness.run();
    harness
}

fn app(harness: &Test) -> &App {
    harness.state().app()
}

fn state(harness: &Test) -> AppState {
    app(harness).state.clone()
}

/// The rect `word` was drawn in, in screen coordinates.
fn word_rect(harness: &Test, word: &str) -> Rect {
    let app = app(harness);
    reader::word_rect(&app.reader_layout, word, app.reader_origin)
        .unwrap_or_else(|| panic!("{word:?} is not in the laid out text"))
}

fn word_pos(harness: &Test, word: &str) -> Pos2 {
    word_rect(harness, word).center()
}

/// Click a word where the user sees it, instead of looking it up by label: a
/// word can appear more than once in a chapter, and only the position is what
/// a user actually aims at.
fn click_word(harness: &mut Test, word: &str) {
    let pos = word_pos(harness, word);
    click_at(harness, pos);
}

// ------------------------------------------------------------------ gestures

/// A plain mouse click at `pos`.
fn click_at(harness: &mut Test, pos: Pos2) {
    harness.hover_at(pos);
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    harness.run();
}

/// A finger going down at `pos`.
///
/// A touch screen reports both the touch and the button that goes with it, so
/// the tests send both; the app sees them as a press that a click also is.
fn touch_start(harness: &mut Test, pos: Pos2) {
    touch(harness, egui::TouchPhase::Start, pos);
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
}

/// A finger moving to `pos` without lifting.
fn touch_move(harness: &mut Test, pos: Pos2) {
    touch(harness, egui::TouchPhase::Move, pos);
}

/// A finger lifting at `pos`.
fn touch_end(harness: &mut Test, pos: Pos2) {
    touch(harness, egui::TouchPhase::End, pos);
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
}

fn touch(harness: &mut Test, phase: egui::TouchPhase, pos: Pos2) {
    harness.event(egui::Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase,
        pos,
        force: Some(1.0),
    });
    harness.event(egui::Event::PointerMoved(pos));
    harness.step();
}

/// A complete tap, as a touch screen reports it.
fn touch_tap(harness: &mut Test, pos: Pos2) {
    touch_start(harness, pos);
    touch_end(harness, pos);
}

/// A mouse drag from `from` to `to` in a few steps.
fn mouse_drag(harness: &mut Test, from: Pos2, to: Pos2) {
    harness.hover_at(from);
    harness.drag_at(from);
    for step in 1..=6 {
        let t = step as f32 / 6.0;
        harness.hover_at(from.lerp(to, t));
        harness.step();
    }
    harness.drop_at(to);
    harness.step();
}

/// Scroll the reader with the mouse wheel.
///
/// The wheel only reaches the reader while the pointer is over it, so the
/// pointer is parked in the middle of the reader first.
fn wheel(harness: &mut Test, delta: f32) {
    harness.hover_at(app(harness).reader_viewport.center());
    harness.step();
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: Vec2::new(0.0, delta),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
}

/// Scroll as far down as the reader goes.
fn scroll_to_end(harness: &mut Test) {
    for _ in 0..40 {
        wheel(harness, -400.0);
    }
    harness.run();
}

// ----------------------------------------------------------------- navigation

#[test]
fn the_app_starts_on_the_book_list() {
    let harness = harness();
    assert_eq!(state(&harness), AppState::BookSelection);
    assert!(harness.query_by_label_contains("Бытие").is_some());
}

#[test]
fn clicking_a_book_then_a_chapter_opens_the_reader() {
    let mut harness = harness();
    harness.get_by_label("Бытие").click();
    harness.run();
    assert_eq!(
        state(&harness),
        AppState::ChapterSelection {
            book_id: "gen".into()
        }
    );

    harness.get_by_label("2").click();
    harness.run();
    assert_eq!(
        state(&harness),
        AppState::Reader {
            book_id: "gen".into(),
            chapter: 2
        }
    );
}

#[test]
fn the_back_button_walks_back_out_of_the_reader() {
    let mut harness = reader();
    harness.get_by_label("Back").click();
    harness.run();
    assert_eq!(
        state(&harness),
        AppState::ChapterSelection {
            book_id: "gen".into()
        }
    );

    harness.get_by_label("Back").click();
    harness.run();
    assert_eq!(state(&harness), AppState::BookSelection);
}

#[test]
fn the_settings_button_opens_the_settings_screen() {
    let mut harness = reader();
    harness.get_by_label("Settings").click();
    harness.run();
    assert!(matches!(state(&harness), AppState::Settings { .. }));
    assert_eq!(state(&harness).title(), "Settings");
}

#[test]
fn the_zoom_buttons_change_the_zoom() {
    let mut harness = reader();
    harness.get_by_label("Settings").click();
    harness.run();

    let before = app(&harness).zoom;
    harness.get_by_label("+").click();
    harness.run();
    assert!(app(&harness).zoom > before, "+ should zoom in");
    // The zoom really reaches the window, not just the app state.
    assert!(
        harness.ctx.zoom_factor() > 1.0,
        "the egui context should be zoomed in, got {}",
        harness.ctx.zoom_factor()
    );

    harness.get_by_label("−").click();
    harness.run();
    assert!((app(&harness).zoom - before).abs() < f32::EPSILON);
    assert!((harness.ctx.zoom_factor() - before).abs() < 0.001);
}

#[test]
fn the_back_button_returns_from_the_settings_screen() {
    let mut harness = reader();
    harness.get_by_label("Settings").click();
    harness.run();
    harness.get_by_label("Back").click();
    harness.run();
    assert_eq!(
        state(&harness),
        AppState::Reader {
            book_id: "gen".into(),
            chapter: 1
        }
    );
}

// -------------------------------------------------- bug: words off the window

#[test]
fn no_word_is_drawn_past_the_right_edge_of_the_window() {
    let harness = reader();
    let app = app(&harness);
    assert!(!app.reader_layout.items.is_empty());

    for item in &app.reader_layout.items {
        let left = app.reader_origin.x + item.rect.left();
        let right = app.reader_origin.x + item.rect.right();
        assert!(
            left >= -0.5 && right <= WINDOW.x + 0.5,
            "{:?} is drawn at x={left}..{right}, outside the {WINDOW:?} window",
            item.text
        );
    }
}

#[test]
fn no_word_sticks_out_of_the_reader_panel() {
    let harness = reader();
    let app = app(&harness);
    let viewport = app.reader_viewport;

    for item in &app.reader_layout.items {
        let highlight = app
            .reader_layout
            .highlight_rect(item)
            .translate(app.reader_origin.to_vec2());
        assert!(
            highlight.right() <= viewport.right() + 0.5,
            "{:?} highlight ends at {} but the panel ends at {}",
            item.text,
            highlight.right(),
            viewport.right()
        );
    }
}

#[test]
fn the_text_is_wrapped_into_several_lines() {
    let harness = reader();
    assert!(app(&harness).reader_layout.line_count > 5);
}

#[test]
fn the_gap_between_words_is_wide_enough_to_see() {
    let harness = reader();
    let app = app(&harness);
    let layout = &app.reader_layout;

    // Two words on the same line with nothing but a comma between them.
    let pair = layout
        .items
        .windows(2)
        .find(|w| w[0].line == w[1].line && w[0].is_word && w[1].is_word)
        .expect("two words should share a line");
    let gap = pair[1].rect.left() - pair[0].rect.right();
    assert!(gap >= 10.0, "the gap between words is only {gap} points");
}

// ---------------------------------------------------- bug: tap vs mouse click

#[test]
fn clicking_a_word_opens_its_translation() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");

    assert_eq!(app(&harness).selected_word.as_deref(), Some("сотворил"));
    assert_eq!(app(&harness).status_of("сотворил"), WordStatus::Seen);
    assert!(
        harness.query_by_label_contains("created").is_some(),
        "the drawer should show the translation"
    );
}

#[test]
fn clicking_the_open_word_closes_the_drawer_again() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");
    assert!(app(&harness).selected_word.is_some());

    click_word(&mut harness, "сотворил");
    assert!(app(&harness).selected_word.is_none());
}

#[test]
fn clicking_another_word_switches_the_drawer() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");
    click_word(&mut harness, "землю");

    assert_eq!(app(&harness).selected_word.as_deref(), Some("землю"));
    assert!(harness.query_by_label_contains("earth").is_some());
}

#[test]
fn the_status_buttons_in_the_drawer_change_the_word_status() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");

    harness.get_by_label("Known").click();
    harness.run();
    assert_eq!(app(&harness).status_of("сотворил"), WordStatus::Known);

    harness.get_by_label("Seen").click();
    harness.run();
    assert_eq!(app(&harness).status_of("сотворил"), WordStatus::Seen);
}

#[test]
fn the_close_button_closes_the_drawer() {
    let mut harness = reader();
    click_word(&mut harness, "землю");
    harness.get_by_label("Close").click();
    harness.run();
    assert!(app(&harness).selected_word.is_none());
}

#[test]
fn a_touch_tap_does_the_same_thing_as_a_mouse_click() {
    let mut harness = reader();
    let pos = word_pos(&harness, "сотворил");

    touch_tap(&mut harness, pos);
    harness.run();

    assert_eq!(
        app(&harness).selected_word.as_deref(),
        Some("сотворил"),
        "a finger tap should open the word just like a click does"
    );
    assert_eq!(app(&harness).status_of("сотворил"), WordStatus::Seen);
}

#[test]
fn a_second_touch_tap_on_the_same_word_closes_the_drawer() {
    let mut harness = reader();
    let pos = word_pos(&harness, "сотворил");

    touch_tap(&mut harness, pos);
    harness.run();
    touch_tap(&mut harness, pos);
    harness.run();

    assert!(app(&harness).selected_word.is_none());
}

#[test]
fn a_slight_finger_jitter_does_not_cancel_a_tap() {
    let mut harness = reader();
    let pos = word_pos(&harness, "сотворил");

    // A real finger wobbles by a pixel or two; that still counts as a tap.
    touch_start(&mut harness, pos);
    touch_move(&mut harness, pos + egui::vec2(1.0, 1.0));
    touch_end(&mut harness, pos + egui::vec2(1.0, 1.0));
    harness.run();

    assert_eq!(
        app(&harness).selected_word.as_deref(),
        Some("сотворил"),
        "a one pixel wobble is still a tap"
    );
    assert_eq!(app(&harness).status_of("сотворил"), WordStatus::Seen);
}

#[test]
fn tapping_a_word_hidden_behind_the_drawer_does_nothing() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");

    let open = app(&harness);
    let drawer = open.reader_drawer.expect("the drawer is open");
    // A word that is currently drawn behind the drawer.
    let (word, spot) = open
        .reader_layout
        .items
        .iter()
        .filter(|item| item.is_clickable() && item.full_text != "сотворил")
        .filter_map(|item| {
            let rect = open
                .reader_layout
                .highlight_rect(item)
                .translate(open.reader_origin.to_vec2());
            let hidden = rect.intersect(drawer);
            (hidden.width() > 4.0 && hidden.height() > 4.0)
                .then(|| (item.full_text.clone(), hidden))
        })
        .next_back()
        .expect("the drawer should be covering at least one word");

    touch_tap(&mut harness, spot.center());
    harness.run();

    assert_eq!(
        app(&harness).selected_word.as_deref(),
        Some("сотворил"),
        "the tap landed on the drawer, not on a word"
    );
    assert_eq!(
        app(&harness).status_of(&word),
        WordStatus::Unseen,
        "a word behind the drawer must not be tappable"
    );
}

// ------------------------------------------------------------- bug: scrolling

#[test]
fn the_reader_has_a_visible_scroll_bar() {
    let harness = reader();
    let app = app(&harness);
    assert!(
        app.reader_content_height > app.reader_viewport.height(),
        "the test chapter should be taller than the window"
    );

    let bar = harness.get_by_role(Role::ScrollBar).rect();
    let viewport = app.reader_viewport;
    assert!(
        bar.min.x >= viewport.max.x - 1.0,
        "the scroll bar belongs to the right of the text: bar {bar:?}, viewport {viewport:?}"
    );
    assert!(
        bar.width() > 4.0 && bar.height() > 20.0,
        "the bar is grabbable"
    );
}

#[test]
fn the_open_drawer_leaves_the_scroll_bar_alone() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");

    let bar = harness.get_by_role(Role::ScrollBar).rect();
    let drawer = app(&harness).reader_drawer.expect("the drawer is open");
    assert!(
        drawer.right() <= bar.left() + 1.0,
        "the drawer must not cover the scroll bar: drawer {drawer:?}, bar {bar:?}"
    );
}

#[test]
fn the_mouse_wheel_scrolls_the_text_down_and_back_up() {
    let mut harness = reader();
    assert_eq!(app(&harness).reader_scroll, 0.0);

    wheel(&mut harness, -300.0);
    harness.run();
    let down = app(&harness).reader_scroll;
    assert!(down > 0.0, "the wheel should scroll down");

    wheel(&mut harness, 300.0);
    harness.run();
    assert!(app(&harness).reader_scroll < down);
}

#[test]
fn dragging_the_scroll_bar_scrolls_the_text() {
    let mut harness = reader();
    let start = harness.get_by_role(Role::ScrollBar).rect().center();

    mouse_drag(&mut harness, start, pos2(start.x, start.y + 150.0));
    harness.run();

    assert!(
        app(&harness).reader_scroll > 0.0,
        "dragging the scroll bar should move the text"
    );
}

#[test]
fn dragging_up_with_a_finger_scrolls_down() {
    let mut harness = reader();
    // Start on a word, like a finger would: the drag must scroll rather than
    // select the word it happened to start on.
    let start = word_pos(&harness, "сотворил");
    let end = pos2(start.x, start.y - 150.0);

    touch_start(&mut harness, start);
    for step in 1..=6 {
        touch_move(&mut harness, start.lerp(end, step as f32 / 6.0));
        harness.run();
    }
    touch_end(&mut harness, end);
    harness.run();

    assert!(
        app(&harness).reader_scroll > 0.0,
        "dragging up should scroll further down"
    );
    assert!(
        app(&harness).selected_word.is_none(),
        "a drag must not pick a word"
    );
}

#[test]
fn dragging_down_with_a_finger_scrolls_back_up() {
    let mut harness = reader();
    wheel(&mut harness, -300.0);
    harness.run();
    let before = app(&harness).reader_scroll;
    assert!(before > 0.0);

    let start = pos2(400.0, 300.0);
    let end = pos2(400.0, 450.0);
    touch_start(&mut harness, start);
    for step in 1..=6 {
        touch_move(&mut harness, start.lerp(end, step as f32 / 6.0));
        harness.run();
    }
    touch_end(&mut harness, end);
    harness.run();

    assert!(app(&harness).reader_scroll < before);
}

#[test]
fn dragging_the_text_with_the_mouse_also_scrolls() {
    let mut harness = reader();
    let start = word_pos(&harness, "сотворил");
    mouse_drag(&mut harness, start, pos2(start.x, start.y - 150.0));
    harness.run();

    assert!(app(&harness).reader_scroll > 0.0);
    assert!(app(&harness).selected_word.is_none());
}

#[test]
fn a_drag_over_a_word_does_not_select_it() {
    let mut harness = reader();
    let start = word_pos(&harness, "сотворил");
    mouse_drag(&mut harness, start, pos2(start.x, start.y - 150.0));
    harness.run();
    assert!(app(&harness).selected_word.is_none());
}

#[test]
fn a_tap_after_a_drag_still_selects_a_word() {
    let mut harness = reader();
    // Scroll around a bit, then tap: the reader must not be left in some
    // half-dragged state.
    mouse_drag(&mut harness, pos2(400.0, 300.0), pos2(400.0, 450.0));
    harness.run();

    let pos = word_pos(&harness, "землю");
    touch_tap(&mut harness, pos);
    harness.run();
    assert_eq!(app(&harness).selected_word.as_deref(), Some("землю"));
}

// ------------------------------------ bug: last line hidden by the drawer

#[test]
fn opening_the_drawer_reserves_room_to_scroll_past_it() {
    let mut harness = reader();
    let without = app(&harness).reader_content_height;

    click_word(&mut harness, "сотворил");

    let with = app(&harness).reader_content_height;
    let drawer = app(&harness).reader_drawer.expect("the drawer is open");
    assert!(drawer.height() > 0.0);
    assert!(
        with - without >= drawer.height(),
        "opening the drawer must reserve at least its own height: {without} -> {with}, drawer {:?}",
        drawer.height()
    );
}

#[test]
fn the_last_line_can_be_scrolled_above_the_drawer() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");
    let drawer = app(&harness).reader_drawer.expect("the drawer is open");

    scroll_to_end(&mut harness);

    let app = app(&harness);
    let last = app
        .reader_layout
        .items
        .last()
        .expect("the chapter has words");
    let last_bottom =
        app.reader_origin.y + app.reader_layout.highlight_rect(last).bottom() - app.reader_scroll;

    assert!(
        last_bottom <= drawer.top() + 0.5,
        "the last line ends at {last_bottom}, still behind the drawer at {:?}",
        drawer.top()
    );
}

#[test]
fn scrolling_stops_at_the_end_of_the_reserved_space() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");
    scroll_to_end(&mut harness);

    let app = app(&harness);
    let max = app.reader_content_height - app.reader_viewport.height();
    assert!(
        (app.reader_scroll - max).abs() < 1.0,
        "scrolled to {} but the maximum is {max}",
        app.reader_scroll
    );
    assert!(
        app.reader_content_height >= app.reader_layout.height() + READER_BOTTOM_PADDING,
        "there is always at least a little room below the text"
    );
}

// ---------------------------------------------------------------- persistence

#[test]
fn a_status_change_is_saved_as_soon_as_it_happens() {
    let mut harness = reader();
    click_word(&mut harness, "сотворил");
    harness.get_by_label("Known").click();
    harness.run();

    let app = app(&harness);
    assert_eq!(app.status_of("сотворил"), WordStatus::Known);
    assert!(
        !app.word_store.is_dirty(),
        "the state should already be written to the store"
    );
    assert!(app.save_error.is_none());
}
