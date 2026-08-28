pub(crate) mod layout;
mod mouse;
mod scroll;
mod state;
mod update;
mod views;

pub use update::Game;

use macroquad::prelude::*;

use crate::domain::{
    load_bible_from_rst, load_states, load_translations, save_states, Bible, Translations,
    WordStateStore,
};
use crate::ui::layout::{
    ReaderLayout, Theme, BIBLE_PATH, FONT_PATH, READER_FONT_SIZE, READER_LEFT, READER_TOP,
    TRANSLATIONS_PATH,
};
use crate::ui::mouse::{Input, MouseState};
use crate::ui::state::AppState;
use crate::ui::update::update;

#[cfg(test)]
mod mouse_test;
#[cfg(test)]
mod scroll_test;
#[cfg(test)]
mod update_test;
#[cfg(test)]
mod layout_test;

pub async fn run() {
    let font = match load_ttf_font(FONT_PATH).await {
        Ok(font) => font,
        Err(e) => {
            error!("Failed to load font '{}': {}", FONT_PATH, e);
            return;
        }
    };
    set_default_font(font.clone());
    let measure = make_measure(&font);

    let bible = match load_bible_from_rst(BIBLE_PATH).await {
        Ok(bible) => bible,
        Err(e) => {
            error!("Failed to load bible '{}': {}", BIBLE_PATH, e);
            return;
        }
    };

    let translations = match load_translations(TRANSLATIONS_PATH).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to load translations '{}': {}", TRANSLATIONS_PATH, e);
            return;
        }
    };

    let theme = Theme::new();
    let mut game = Game::new(WordStateStore::from_states(load_states()));
    let mut mouse = MouseState::new();

    loop {
        let screen = vec2(screen_width(), screen_height());
        mouse.update(is_mouse_button_down(MouseButton::Left), mouse_wheel());

        let input = Input::from_mouse(&mouse, vec2(mouse_position().0, mouse_position().1));
        let reader_layout = build_reader_layout(&game.state, &bible, screen, &measure);

        update(&mut game, &bible, &input, screen, reader_layout.as_ref());

        if game.word_store.is_dirty() {
            match save_states(&game.word_store.as_map()) {
                Ok(()) => game.word_store.clear_dirty(),
                Err(e) => error!("Failed to save word states: {}", e),
            }
        }

        render(&game, &bible, &font, &translations, &theme, screen, reader_layout.as_ref());

        next_frame().await;
    }
}

/// Build the layout for the current chapter if the app is on the reader screen.
/// The run loop owns the font, so measuring is not available in `update` /
/// headless tests; they pass a fake measurement closure instead.
fn build_reader_layout(
    state: &AppState,
    bible: &Bible,
    screen: Vec2,
    measure: &dyn Fn(&str, f32) -> f32,
) -> Option<ReaderLayout> {
    let (book_id, chapter) = match state {
        AppState::Reader { book_id, chapter } => (book_id.clone(), *chapter),
        _ => return None,
    };
    let tokens = bible.get_chapter_tokens(&book_id, chapter)?;
    let max_width = (screen.x - READER_LEFT * 2.0).max(100.0);
    Some(ReaderLayout::new(
        &tokens,
        READER_FONT_SIZE,
        READER_LEFT,
        READER_TOP,
        max_width,
        measure,
    ))
}

fn make_measure(font: &Font) -> impl Fn(&str, f32) -> f32 + '_ {
    move |text, size| measure_text(text, Some(font), size as u16, 1.0).width
}

fn render(
    game: &Game,
    bible: &Bible,
    font: &Font,
    translations: &Translations,
    theme: &Theme,
    screen: Vec2,
    reader_layout: Option<&ReaderLayout>,
) {
    let font_size = READER_FONT_SIZE;
    let ctx = views::DrawContext { theme, font, screen };
    match &game.state {
        AppState::BookSelection => {
            views::render_book_selection(&ctx, font_size, &bible.get_books_in_order(), &game.book_scroll);
        }
        AppState::ChapterSelection { book_id } => {
            let chapters = bible
                .get_book(book_id)
                .map(|b| b.chapter_numbers())
                .unwrap_or_default();
            views::render_chapter_selection(&ctx, font_size, &chapters, &game.chapter_scroll);
        }
        AppState::Reader { .. } => {
            if let Some(layout) = reader_layout {
                views::render_reader(
                    &ctx,
                    layout,
                    &game.reader_scroll,
                    game.selected_word.as_deref(),
                    &game.word_store,
                    translations,
                );
            }
        }
    }
}