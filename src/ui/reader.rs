//! The reading screen: the chapter text plus the translation drawer.
//!
//! The text is laid out by [`TextLayout`] against the width the panel actually
//! has, so words can never stick out of the window. It is painted inside a
//! `ScrollArea`, which gives us a visible scroll bar, mouse-wheel scrolling and
//! touch drag-to-scroll for free, and the drawer is painted on top of it with
//! matching blank space reserved below the last line so the text can always be
//! scrolled out from behind it.

use egui::containers::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::{
    pos2, vec2, Color32, CornerRadius, FontId, Margin, Pos2, Rect, RichText, ScrollArea, Sense,
    Stroke, Ui, UiBuilder, Vec2,
};

use crate::app::App;
use crate::domain::WordStatus;
use crate::ui::layout::{
    bottom_band, drawer_height, reader_bottom_space, wrapped_line_count, TextLayout,
    DRAWER_PADDING, DRAWER_TEXT_SIZE, DRAWER_TITLE_SIZE, LINE_HEIGHT_FACTOR, READER_FONT_SIZE,
};
use crate::ui::theme::{lighten, Theme};

/// Words that are `Known` are not highlighted, and the currently open word gets
/// a brighter colour.
fn highlight_color(theme: &Theme, status: WordStatus, selected: bool) -> Option<Color32> {
    let base = match status {
        WordStatus::Unseen => theme.unseen,
        WordStatus::Seen => theme.seen,
        WordStatus::Known => return None,
    };
    Some(if selected { lighten(base, 0.25) } else { base })
}

pub fn show(ui: &mut Ui, app: &mut App, book_id: &str, chapter: u32) {
    let Some(tokens) = app.data.bible.get_chapter_tokens(book_id, chapter) else {
        ui.centered_and_justified(|ui| {
            ui.label("This chapter is missing from the bible file.");
        });
        return;
    };

    let theme = Theme::new();
    let panel = ui.max_rect();
    let font_size = READER_FONT_SIZE;
    let line_height = font_size * LINE_HEIGHT_FACTOR;
    let font_id = FontId::proportional(font_size);

    // How much room the drawer needs, so the text can be scrolled clear of it.
    // The drawer covers the text but not the scroll bar: the bar has to stay
    // visible and grabbable while a word is open.
    let drawer = app.selected_word.as_deref().map(|word| {
        drawer_rect(
            drawer_area(panel, ui.spacing().scroll.allocated_width()),
            app,
            ui.ctx(),
            word,
        )
    });
    let drawer_h = drawer.map_or(0.0, |rect| rect.height());

    let mut selected: Option<String> = None;
    let mut painted: Option<(TextLayout, Pos2)> = None;
    // The drawer floats on top of the text, so words behind it must not be
    // tappable: they would look hidden but still react.
    let drawer_top = drawer.map_or(f32::INFINITY, |rect| rect.top());

    let scroll = ScrollArea::vertical()
        .id_salt("reader")
        // Keep the scroll bar visible and reserve its width, so the width used
        // for the text below is the width the text may actually occupy.
        .auto_shrink(false)
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        // Scroll bar, mouse wheel and drag-to-scroll (the one a touch screen
        // user expects) all at once.
        .scroll_source(ScrollSource::ALL)
        .show(ui, |ui| {
            let content_width = ui.available_width();
            let ctx = ui.ctx().clone();
            let measure = crate::ui::fonts::measurer(&ctx);
            let built = TextLayout::build(&tokens, font_size, line_height, content_width, &measure);
            let height = built.height();

            // Reserve the whole content area, then keep the offset so the items
            // can be placed in screen coordinates.
            let (rect, _response) =
                ui.allocate_exact_size(vec2(content_width, height), Sense::hover());
            let delta: Vec2 = rect.min - pos2(0.0, 0.0);
            let painter = ui.painter().clone();

            let clip = ui.clip_rect();
            let visible = visible_range(&built, clip, delta, line_height);
            let items = &built.items[visible.clone()];

            // Highlights first, so the text draws on top of them.
            for item in items.iter().filter(|item| item.is_word) {
                let is_selected = app.selected_word.as_deref() == Some(item.full_text.as_str());
                let Some(color) =
                    highlight_color(&theme, app.status_of(&item.full_text), is_selected)
                else {
                    continue;
                };
                let rect = built.highlight_rect(item).translate(delta);
                painter.rect_filled(rect.intersect(clip), CornerRadius::same(3), color);
            }

            for item in items {
                let galley = ui.ctx().fonts_mut(|fonts| {
                    fonts.layout_no_wrap(item.text.clone(), font_id.clone(), Color32::PLACEHOLDER)
                });
                let row_height = galley.size().y;
                let text_rect = item.rect.translate(delta);
                // Centre the text inside its line box.
                let top = text_rect.top() + (text_rect.height() - row_height) * 0.5;
                painter.galley(pos2(text_rect.left(), top), galley, theme.text);
            }

            // Tap/click targets. Only the visible part of a word is tappable, so
            // words hidden behind the toolbar or the drawer cannot be hit.
            for item in items.iter().filter(|item| item.is_clickable()) {
                let visible =
                    Rect::from_min_max(clip.min, pos2(clip.max.x, drawer_top.min(clip.max.y)));
                let rect = built
                    .highlight_rect(item)
                    .translate(delta)
                    .intersect(visible);
                if rect.width() < 1.0 || rect.height() < 1.0 {
                    continue;
                }
                let response = ui.interact(
                    rect,
                    ui.id().with(("word", item.token_index)),
                    Sense::click(),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        item.full_text.clone(),
                    )
                });
                if response.clicked() {
                    selected = Some(item.full_text.clone());
                }
            }

            // Blank space below the last line so it can be scrolled above the
            // translation drawer instead of hiding behind it.
            ui.add_space(reader_bottom_space(drawer_h));
            painted = Some((built, rect.min));
        });

    if let Some((built, origin)) = painted {
        // Republish the geometry so tests can aim taps at real words.
        app.reader_layout = built;
        app.reader_origin = origin;
        app.reader_scroll = scroll.state.offset.y;
        app.reader_content_height = scroll.content_size.y;
        app.reader_viewport = scroll.inner_rect;
    }
    app.reader_drawer = drawer;
    if let Some(word) = selected {
        app.toggle_word(&word);
    }

    if let (Some(rect), Some(word)) = (drawer, app.selected_word.clone()) {
        show_drawer(ui, app, rect, &word);
    }
}

/// Index range of the items that can be seen, so a long chapter only paints
/// what is on screen.
fn visible_range(
    layout: &TextLayout,
    clip: Rect,
    delta: Vec2,
    line_height: f32,
) -> std::ops::Range<usize> {
    if line_height <= 0.0 || layout.line_count == 0 {
        return 0..0;
    }
    let first = ((clip.min.y - delta.y) / line_height).floor();
    let last = ((clip.max.y - delta.y) / line_height).ceil();
    if !first.is_finite() || !last.is_finite() {
        return 0..0;
    }
    layout.item_range_for_lines(
        (first.max(0.0) as usize).min(layout.line_count - 1),
        (last.max(0.0) as usize).min(layout.line_count - 1),
    )
}

/// The part of the reader the drawer may cover: everything but the scroll bar.
fn drawer_area(panel: Rect, scroll_bar_width: f32) -> Rect {
    let right = (panel.right() - scroll_bar_width).max(panel.left());
    Rect::from_min_max(panel.min, pos2(right, panel.bottom()))
}

/// Where the translation drawer goes: a band across the bottom of the reader.
fn drawer_rect(panel: Rect, app: &App, ctx: &egui::Context, word: &str) -> Rect {
    let translation = app.data.translations.display(word);
    let text_width = (panel.width() - DRAWER_PADDING * 4.0).max(40.0);
    let measure = crate::ui::fonts::measurer(ctx);
    let lines = wrapped_line_count(&translation, DRAWER_TEXT_SIZE, text_width, &measure);
    let height = drawer_height(lines, READER_FONT_SIZE, panel.height());
    bottom_band(panel, height)
}

/// The translation drawer: the word, its closest words and the status buttons.
fn show_drawer(ui: &mut Ui, app: &mut App, rect: Rect, word: &str) {
    let theme = Theme::new();
    // `new_child` rather than `scope_builder`: the drawer is an overlay, so it
    // must not push the reader's layout around.
    let mut drawer_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let ui = &mut drawer_ui;
    // The drawer is a band pinned to the bottom; never let its content spill
    // over the text above it.
    ui.set_clip_rect(rect);

    egui::Frame::new()
        .fill(theme.panel)
        .stroke(Stroke::new(1.0, theme.dim_text))
        .inner_margin(Margin::symmetric(
            DRAWER_PADDING as i8,
            (DRAWER_PADDING * 0.75) as i8,
        ))
        .show(ui, |ui| {
            ui.set_width((rect.width() - DRAWER_PADDING * 2.0).max(1.0));

            let mut close = false;
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(word)
                            .size(DRAWER_TITLE_SIZE)
                            .strong()
                            .color(theme.text),
                    )
                    .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui.small_button("Close").clicked();
                });
            });
            if close {
                app.close_drawer();
            }

            let translation = app.data.translations.display(word);
            let color = match app.data.translations.get(word) {
                Some(words) if !words.is_empty() => theme.text,
                _ => theme.dim_text,
            };
            ui.add(
                egui::Label::new(
                    RichText::new(translation)
                        .size(DRAWER_TEXT_SIZE)
                        .color(color),
                )
                .wrap(),
            );

            ui.add_space(4.0);
            let mut current = app.status_of(word);
            let mut chosen = None;
            ui.horizontal(|ui| {
                for status in [WordStatus::Unseen, WordStatus::Seen, WordStatus::Known] {
                    if ui
                        .selectable_value(&mut current, status, status.label())
                        .changed()
                    {
                        chosen = Some(status);
                    }
                }
            });
            if let Some(status) = chosen {
                app.set_selected_status(status);
            }
        });
}

/// Screen-space rect of `word` in `layout` as drawn from `origin`.
///
/// The reader itself does not need this; the GUI tests use it to aim taps at a
/// word the way a user would.
#[cfg(test)]
pub fn word_rect(layout: &TextLayout, word: &str, origin: egui::Pos2) -> Option<Rect> {
    let item = layout.item_for_word(word)?;
    Some(layout.highlight_rect(item).translate(origin.to_vec2()))
}
