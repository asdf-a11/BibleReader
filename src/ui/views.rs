use macroquad::math::Vec2;
use macroquad::prelude::*;

use crate::domain::{Book, Translations, WordStateStore, WordStatus};
use crate::ui::layout::{
    back_button_rect, chapter_buttons, grid_content_height, list_content_height,
    scrollbar_viewport, settings_button_rect, DrawerLayout, ReaderLayout, SettingsLayout, Theme,
    BOOKS_LEFT, BOOKS_TOP, BOOK_HEIGHT, GRID_TOP, READER_FONT_SIZE, READER_TOP,
};
use crate::ui::scroll::{ScrollState, ScrollbarGeometry};

const TITLE_Y: f32 = 100.0;
const TITLE_PADDING_X: f32 = 20.0;

/// The shared drawing inputs for every screen.
pub(crate) struct DrawContext<'a> {
    pub theme: &'a Theme,
    pub font: &'a Font,
    pub screen: Vec2,
    /// Effective UI scale (window fit * user zoom); all geometry/fonts are
    /// multiplied by this.
    pub scale: f32,
}

pub(crate) fn render_book_selection(ctx: &DrawContext, books: &[&Book], scroll: &ScrollState) {
    clear_background(ctx.theme.background_color);
    let font_size = READER_FONT_SIZE * ctx.scale;
    draw_title("Books", ctx, font_size);

    for (index, book) in books.iter().enumerate() {
        let y = BOOKS_TOP * ctx.scale + index as f32 * BOOK_HEIGHT * ctx.scale - scroll.offset;
        draw_text(
            &book.name,
            BOOKS_LEFT * ctx.scale,
            y + font_size - 4.0,
            font_size - 4.0,
            ctx.theme.text_color,
        );
    }

    let geometry = scroll.geometry(
        list_content_height(books.len(), ctx.scale),
        scrollbar_viewport(ctx.screen, BOOKS_TOP, ctx.scale),
    );
    draw_scrollbar(ctx.theme, &geometry);
    draw_settings_button(ctx);
}

pub(crate) fn render_chapter_selection(
    ctx: &DrawContext,
    chapters: &[u32],
    scroll: &ScrollState,
) {
    clear_background(ctx.theme.background_color);
    let font_size = READER_FONT_SIZE * ctx.scale;
    draw_title("Chapters", ctx, font_size);
    draw_back_button(ctx);
    draw_settings_button(ctx);

    let buttons = chapter_buttons(chapters, ctx.screen.x, GRID_TOP, ctx.scale);
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
        grid_content_height(chapters.len(), ctx.scale),
        scrollbar_viewport(ctx.screen, GRID_TOP, ctx.scale),
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
    draw_settings_button(ctx);

    let font_size = READER_FONT_SIZE * ctx.scale;

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

    let geometry = scroll.geometry(
        layout.content_height,
        scrollbar_viewport(ctx.screen, READER_TOP, ctx.scale),
    );
    draw_scrollbar(ctx.theme, &geometry);
}

pub(crate) fn render_settings(ctx: &DrawContext, zoom: f32) {
    clear_background(ctx.theme.background_color);
    let font_size = READER_FONT_SIZE * ctx.scale;
    draw_title("Settings", ctx, font_size);
    draw_back_button(ctx);

    let layout = SettingsLayout::new(ctx.screen, ctx.scale);

    let percent = (zoom * 100.0).round() as i32;
    let label = format!("Zoom: {}%", percent);
    let metrics = measure_text(&label, Some(ctx.font), font_size as u16, 1.0);
    draw_text(
        &label,
        layout.zoom_label.center().x - metrics.width / 2.0,
        layout.zoom_label.center().y + font_size / 2.0,
        font_size,
        ctx.theme.text_color,
    );

    draw_zoom_button(ctx, layout.zoom_out, "-");
    draw_zoom_button(ctx, layout.zoom_in, "+");
}

fn draw_zoom_button(ctx: &DrawContext, rect: macroquad::math::Rect, symbol: &str) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, ctx.theme.panel_color);
    let font_size = READER_FONT_SIZE * ctx.scale;
    let metrics = measure_text(symbol, Some(ctx.font), font_size as u16, 1.0);
    draw_text(
        symbol,
        rect.center().x - metrics.width / 2.0,
        rect.center().y + font_size / 2.0 - 4.0,
        font_size,
        ctx.theme.text_color,
    );
}

fn render_drawer(ctx: &DrawContext, word: &str, translations: &Translations) {
    let drawer = DrawerLayout::new(ctx.screen.x, ctx.screen.y, ctx.scale);

    draw_rectangle(
        drawer.rect.x,
        drawer.rect.y,
        drawer.rect.w,
        drawer.rect.h,
        ctx.theme.panel_color,
    );

    let title_font = READER_FONT_SIZE * ctx.scale;
    draw_text(
        word,
        drawer.title_rect.x,
        drawer.title_rect.y + title_font,
        title_font,
        ctx.theme.text_color,
    );

    let translation = translations.display(word);
    let trans_font = (READER_FONT_SIZE - 4.0) * ctx.scale;
    let metrics = measure_text(&translation, Some(ctx.font), trans_font as u16, 1.0);
    draw_text(
        &translation,
        drawer.translation_rect.x,
        drawer.translation_rect.y + metrics.height,
        trans_font,
        ctx.theme.text_color,
    );

    draw_close_button(ctx, drawer.close_button);

    for button in &drawer.buttons {
        draw_rectangle(
            button.rect.x,
            button.rect.y,
            button.rect.w,
            button.rect.h,
            ctx.theme.unseen_color,
        );
        let label_font = (READER_FONT_SIZE - 8.0) * ctx.scale;
        let metrics = measure_text(&button.label, Some(ctx.font), label_font as u16, 1.0);
        draw_text(
            &button.label,
            button.rect.center().x - metrics.width / 2.0,
            button.rect.center().y - metrics.height / 2.0,
            label_font,
            ctx.theme.text_color,
        );
    }
}

/// An "X" glyph drawn with two strokes (font independent).
fn draw_close_button(ctx: &DrawContext, rect: macroquad::math::Rect) {
    let thickness = (2.0 * ctx.scale).max(1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, ctx.theme.panel_color);
    let (x0, y0) = (rect.center().x, rect.center().y);
    let inset = rect.h * 0.25;
    draw_line(
        x0 - inset,
        y0 - inset,
        x0 + inset,
        y0 + inset,
        thickness,
        ctx.theme.text_color,
    );
    draw_line(
        x0 + inset,
        y0 - inset,
        x0 - inset,
        y0 + inset,
        thickness,
        ctx.theme.text_color,
    );
}

fn draw_title(title: &str, ctx: &DrawContext, font_size: f32) {
    let metrics = measure_text(title, Some(ctx.font), font_size as u16, 1.0);
    let x = (ctx.screen.x - metrics.width) / 2.0;
    let y = TITLE_Y * ctx.scale;
    draw_text(title, x, y, font_size, ctx.theme.text_color);
    draw_line(
        x - TITLE_PADDING_X * ctx.scale,
        y + 10.0 * ctx.scale,
        x + metrics.width + TITLE_PADDING_X * ctx.scale,
        y + 10.0 * ctx.scale,
        (2.0 * ctx.scale).max(1.0),
        ctx.theme.seen_color,
    );
}

fn draw_back_button(ctx: &DrawContext) {
    let rect = back_button_rect(ctx.scale);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, ctx.theme.seen_color);
    let font_size = 20.0 * ctx.scale;
    let metrics = measure_text("Back", Some(ctx.font), font_size as u16, 1.0);
    draw_text(
        "Back",
        rect.center().x - metrics.width / 2.0,
        rect.y + rect.h / 2.0 + font_size / 2.0 - 4.0,
        font_size,
        ctx.theme.text_color,
    );
}

fn draw_settings_button(ctx: &DrawContext) {
    let rect = settings_button_rect(ctx.screen.x, ctx.scale);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, ctx.theme.panel_color);

    // Simple gear: a ring of ticks around a hollow circle. Drawn with shapes so
    // it never depends on a font having a "gear" glyph.
    let cx = rect.center().x;
    let cy = rect.center().y;
    let r = rect.h * 0.26;
    let teeth = 6;
    for i in 0..teeth {
        let angle = i as f32 / teeth as f32 * std::f32::consts::TAU;
        let tx = cx + (r * 1.8) * angle.cos();
        let ty = cy + (r * 1.8) * angle.sin();
        draw_rectangle(
            tx - r * 0.45,
            ty - r * 0.45,
            r * 0.9,
            r * 0.9,
            ctx.theme.text_color,
        );
    }
    draw_circle(cx, cy, r * 1.4, ctx.theme.text_color);
    draw_circle(cx, cy, r * 0.6, ctx.theme.panel_color);
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