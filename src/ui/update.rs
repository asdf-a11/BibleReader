use macroquad::math::{vec2, Vec2};

use crate::domain::{Bible, WordStateStore};
use crate::ui::layout::{
    back_button_rect, chapter_buttons, grid_content_height, list_content_height,
    scrollbar_viewport, DrawerLayout, ReaderLayout, BOOK_HEIGHT, BOOKS_TOP, GRID_TOP, READER_TOP,
};
use crate::ui::mouse::Input;
use crate::ui::scroll::{update_scroll, ScrollState};
use crate::ui::state::AppState;

/// All mutable application state, kept separate from rendering so it can be
/// unit-tested by feeding `update` synthetic `Input` values.
pub struct Game {
    pub state: AppState,
    pub book_scroll: ScrollState,
    pub chapter_scroll: ScrollState,
    pub reader_scroll: ScrollState,
    pub selected_word: Option<String>,
    pub word_store: WordStateStore,
}

impl Game {
    pub fn new(word_store: WordStateStore) -> Self {
        Game {
            state: AppState::BookSelection,
            book_scroll: ScrollState::new(),
            chapter_scroll: ScrollState::new(),
            reader_scroll: ScrollState::new(),
            selected_word: None,
            word_store,
        }
    }
}

/// Main state-machine dispatch. `reader_layout` is the pre-computed layout of
/// the currently displayed chapter (built by the run loop, which owns the font).
pub fn update(
    game: &mut Game,
    bible: &Bible,
    input: &Input,
    screen: Vec2,
    reader_layout: Option<&ReaderLayout>,
) -> bool {
    let app_state = game.state.clone();
    match app_state {
        AppState::BookSelection => update_book_selection(game, bible, input, screen),
        AppState::ChapterSelection { book_id } => {
            update_chapter_selection(game, bible, &book_id, input, screen);
        }
        AppState::Reader { book_id, .. } => {
            update_reader(game, &book_id, input, screen, reader_layout);
        }
    }
    // Returns whether the word store changed this frame (used by the loop
    // to decide whether to persist to disk/storage).
    game.word_store.is_dirty()
}

fn update_book_selection(game: &mut Game, bible: &Bible, input: &Input, screen: Vec2) {
    let books = bible.get_books_in_order();

    let scrollbar = scrollbar_viewport(screen, BOOKS_TOP);
    let consumed = update_scroll(
        &mut game.book_scroll,
        list_content_height(books.len()),
        scrollbar,
        input,
    );
    if consumed || !input.left_pressed {
        return;
    }

    // Hit-test the book list in content coordinates (scroll offset applied).
    let content_y = input.mouse_pos.y + game.book_scroll.offset;
    if content_y < BOOKS_TOP {
        return;
    }
    let index = ((content_y - BOOKS_TOP) / BOOK_HEIGHT) as usize;
    if let Some(book) = books.get(index) {
        game.state = AppState::ChapterSelection {
            book_id: book.id.clone(),
        };
        game.chapter_scroll.reset();
        game.selected_word = None;
    }
}

fn update_chapter_selection(
    game: &mut Game,
    bible: &Bible,
    book_id: &str,
    input: &Input,
    screen: Vec2,
) {
    if input.left_pressed && back_button_rect().contains(input.mouse_pos) {
        game.state = AppState::BookSelection;
        game.selected_word = None;
        return;
    }

    let Some(book) = bible.get_book(book_id) else {
        return;
    };
    let chapters = book.chapter_numbers();

    let scrollbar = scrollbar_viewport(screen, GRID_TOP);
    let consumed = update_scroll(
        &mut game.chapter_scroll,
        grid_content_height(chapters.len()),
        scrollbar,
        input,
    );
    if consumed || !input.left_pressed {
        return;
    }

    let content_pos = vec2(
        input.mouse_pos.x,
        input.mouse_pos.y + game.chapter_scroll.offset,
    );
    for (chapter, rect) in chapter_buttons(&chapters, screen.x, GRID_TOP) {
        if rect.contains(content_pos) {
            game.state = AppState::Reader {
                book_id: book_id.to_string(),
                chapter,
            };
            game.reader_scroll.reset();
            game.selected_word = None;
            return;
        }
    }
}

fn update_reader(
    game: &mut Game,
    book_id: &str,
    input: &Input,
    screen: Vec2,
    reader_layout: Option<&ReaderLayout>,
) {
    if input.left_pressed && back_button_rect().contains(input.mouse_pos) {
        game.state = AppState::ChapterSelection {
            book_id: book_id.to_string(),
        };
        game.selected_word = None;
        return;
    }

    let Some(layout) = reader_layout else {
        return;
    };

    let scrollbar = scrollbar_viewport(screen, READER_TOP);
    let consumed = update_scroll(
        &mut game.reader_scroll,
        layout.content_height,
        scrollbar,
        input,
    );
    if consumed {
        return;
    }

    // Drawer status buttons take priority over word-selection clicks.
    if let Some(word) = game.selected_word.clone() {
        let drawer = DrawerLayout::new(screen.x, screen.y);
        if input.left_pressed {
            if let Some(button) = drawer.is_point_in_button(input.mouse_pos.x, input.mouse_pos.y)
            {
                game.word_store.set_status(&word, button.status);
                return;
            }
        }
    }

    if !input.left_pressed {
        return;
    }

    // Hit-test words in content coordinates (scroll offset applied).
    let content_pos = vec2(
        input.mouse_pos.x,
        input.mouse_pos.y + game.reader_scroll.offset,
    );
    if let Some(word_layout) = layout.get_word_at_position(content_pos.x, content_pos.y) {
        let word = word_layout.token.to_string();
        if game.selected_word.as_deref() == Some(word.as_str()) {
            // Clicking the same word again hides its translation window.
            game.selected_word = None;
        } else {
            // Clicking a word shows its translation window and marks it seen.
            game.word_store.access_word(&word);
            game.selected_word = Some(word);
        }
    }
}