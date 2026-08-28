use macroquad::math::Vec2;
use macroquad::prelude::*;

use crate::domain::{Book, Translations, WordStateStore, WordStatus};
use crate::ui::layout::{
    back_button_rect, chapter_buttons, grid_content_height, list_content_height,
    scrollbar_viewport, DrawerLayout, ReaderLayout, Theme, BOOKS_LEFT, BOOKS_TOP, BOOK_HEIGHT,
    GRID_TOP, READER_FONT_SIZE, READER_TOP,
};
use crate::ui::scroll::{ScrollState, ScrollbarGeometry};

const TITLE_Y: f32 = 100.0;
const TITLE_PADDING_X: f32 = 20.0;

/// The shared drawing inputs for every screen.
pub(crate) struct DrawContext<'a> {
    pub theme: &'a Theme,
    pub font: &'a Font,
    pub screen: Vec2,
}

pub(crate) fn render_book_selection(
    ctx: &DrawContext,
    font_size: f32,
    books: &[&Book],
    scroll: &ScrollState,
) {
    clear_background(ctx.theme.background_color);
    draw_title("Books", ctx, font_size);

    for (index, book) in books.iter().enumerate() {
        let y = BOOKS_TOP + index as f32 * BOOK_HEIGHT - scroll.offset;
        draw_text(
            &book.name,
            BOOKS_LEFT,
            y + font_size - 4.0,
            font_size - 4.0,
            ctx.theme.text_color,
        );
    }

    let geometry = scroll.geometry(
        list_content_height(books.len()),
        scrollbar_viewport(ctx.screen, BOOKS_TOP),
    );
    draw_scrollbar(ctx.theme, &geometry);
}

pub(crate) fn render_chapter_selection(
    ctx: &DrawContext,
    font_size: f32,
    chapters: &[u32],
    scroll: &ScrollState,
) {
    clear_background(ctx.theme.background_color);
    draw_title("Chapters", ctx, font_size);
    draw_back_button(ctx);

    let buttons = chapter_buttons(chapters, ctx.screen.x, GRID_TOP);
    for (chapter, rect) in &buttons {
        let y = rect.y - scroll.offset;
        draw_rectangle(rect.x, y, rect.w, rect.h, ctx.theme.panel_color);

        let label = chapter.to_string();
        let metrics = measure_text(&label, Some(ctx.font), (font_size - 4.0) as u16, 1.0);
        draw_text(
            &label,
            rect.center().x - metrics.width / 2.0,
            y + rect.h / 2.0 + (font_size - 4.0) / 2.0 - 4.0,
            font_size - 4.0,
            ctx.theme.text_color,
        );
    }

    let geometry = scroll.geometry(
        grid_content_height(chapters.len()),
        scrollbar_viewport(ctx.screen, GRID_TOP),
    );
    draw_scrollbar(ctx.theme, &geometry);
}

pub(crate) fn render_reader(
    ctx: &DrawContext,
    layout: &ReaderLayout,
    scroll: &ScrollState,
    selected_word: Option<&str>,
    word_store: &WordStateStore,
    translations: &Translations,
) {
    clear_background(ctx.theme.background_color);
    draw_back_button(ctx);

    let font_size = READER_FONT_SIZE;

    // State highlight boxes behind each word.
    for word_layout in &layout.word_layouts {
        let status = word_store.get_status(&word_layout.token.to_string());
        if status == WordStatus::Known {
            continue; // Known words are not highlighted.
        }
        let color = match status {
            WordStatus::Unseen => ctx.theme.unseen_color,
            WordStatus::Seen => ctx.theme.seen_color,
            WordStatus::Known => ctx.theme.background_color,
        };
        let y = word_layout.highlight_rect.y - scroll.offset;
        draw_rectangle(
            word_layout.highlight_rect.x,
            y,
            word_layout.highlight_rect.w,
            word_layout.highlight_rect.h,
            color,
        );
    }

    // Word text on top of the highlights.
    for word_layout in &layout.word_layouts {
        let y = word_layout.rect.y - scroll.offset;
        let text = word_layout.token.to_string();
        draw_text(&text, word_layout.rect.x, y + font_size, font_size, ctx.theme.text_color);
    }

    if let Some(word) = selected_word {
        if let Some(word_layout) = layout
            .word_layouts
            .iter()
            .find(|layout| layout.token.to_string() == word)
        {
            let y = word_layout.highlight_rect.y - scroll.offset;
            draw_rectangle(
                word_layout.highlight_rect.x,
                y,
                word_layout.highlight_rect.w,
                word_layout.highlight_rect.h,
                ctx.theme.seen_color,
            );
        }
        render_drawer(ctx, word, translations);
    }

    let geometry = scroll.geometry(layout.content_height, scrollbar_viewport(ctx.screen, READER_TOP));
    draw_scrollbar(ctx.theme, &geometry);
}

fn render_drawer(ctx: &DrawContext, word: &str, translations: &Translations) {
    let drawer = DrawerLayout::new(ctx.screen.x, ctx.screen.y);

    draw_rectangle(
        drawer.rect.x,
        drawer.rect.y,
        drawer.rect.w,
        drawer.rect.h,
        ctx.theme.panel_color,
    );

    draw_text(
        word,
        drawer.title_rect.x,
        drawer.title_rect.y + READER_FONT_SIZE,
        READER_FONT_SIZE,
        ctx.theme.text_color,
    );

    let translation = translations.display(word);
    let metrics = measure_text(&translation, Some(ctx.font), (READER_FONT_SIZE - 4.0) as u16, 1.0);
    draw_text(
        &translation,
        drawer.translation_rect.x,
        drawer.translation_rect.y + metrics.height,
        READER_FONT_SIZE - 4.0,
        ctx.theme.text_color,
    );

    for button in &drawer.buttons {
        draw_rectangle(
            button.rect.x,
            button.rect.y,
            button.rect.w,
            button.rect.h,
            ctx.theme.unseen_color,
        );
        let metrics = measure_text(&button.label, Some(ctx.font), (READER_FONT_SIZE - 8.0) as u16, 1.0);
        draw_text(
            &button.label,
            button.rect.center().x - metrics.width / 2.0,
            button.rect.center().y - metrics.height / 2.0,
            READER_FONT_SIZE - 8.0,
            ctx.theme.text_color,
        );
    }
}

fn draw_title(title: &str, ctx: &DrawContext, font_size: f32) {
    let metrics = measure_text(title, Some(ctx.font), font_size as u16, 1.0);
    let x = (ctx.screen.x - metrics.width) / 2.0;
    draw_text(title, x, TITLE_Y, font_size, ctx.theme.text_color);
    draw_line(
        x - TITLE_PADDING_X,
        TITLE_Y + 10.0,
        x + metrics.width + TITLE_PADDING_X,
        TITLE_Y + 10.0,
        2.0,
        ctx.theme.seen_color,
    );
}

fn draw_back_button(ctx: &DrawContext) {
    let rect = back_button_rect();
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, ctx.theme.seen_color);
    let metrics = measure_text("Back", Some(ctx.font), 20u16, 1.0);
    draw_text(
        "Back",
        rect.center().x - metrics.width / 2.0,
        rect.center().y - metrics.height / 2.0,
        20.0,
        ctx.theme.text_color,
    );
}

fn draw_scrollbar(theme: &Theme, geometry: &ScrollbarGeometry) {
    // Only draw a scrollbar when there is something to scroll.
    if geometry.max_scroll <= 0.0 {
        return;
    }
    draw_rectangle(
        geometry.track.x,
        geometry.track.y,
        geometry.track.w,
        geometry.track.h,
        theme.panel_color,
    );
    draw_rectangle(
        geometry.thumb.x,
        geometry.thumb.y,
        geometry.thumb.w,
        geometry.thumb.h,
        theme.seen_color,
    );
}