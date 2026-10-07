//! The book list, the chapter grid and the settings screen.

use egui::containers::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::{vec2, Align, Rect, ScrollArea, Ui};

use crate::app::App;
use crate::ui::layout::{
    self, CHAPTER_BUTTON_SIZE, CHAPTER_BUTTON_SPACING, MAX_ZOOM, MIN_ZOOM, ZOOM_STEP,
};

/// Book list: one full-width button per book.
pub fn show_books(ui: &mut Ui, app: &mut App) {
    // The ids are collected first and applied after the loop, because the loop
    // borrows `app` immutably while the buttons are alive.
    let books: Vec<(String, String)> = app
        .data
        .bible
        .get_books_in_order()
        .iter()
        .map(|book| (book.id.clone(), book.name.clone()))
        .collect();

    let mut chosen: Option<String> = None;
    ScrollArea::vertical()
        .id_salt("books")
        .auto_shrink([false, false])
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .scroll_source(ScrollSource::ALL)
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.vertical(|ui| {
                for (id, name) in &books {
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [width - 8.0, layout::BOOK_ROW_HEIGHT],
                                egui::Button::new(name),
                            )
                            .clicked()
                        {
                            chosen = Some(id.clone());
                        }
                    });
                }
            });
        });

    if let Some(id) = chosen {
        app.open_book(&id);
    }
}

/// Chapter grid: square buttons, as many per row as fit.
pub fn show_chapters(ui: &mut Ui, app: &mut App, book_id: &str) {
    let Some(book) = app.data.bible.get_book(book_id) else {
        ui.centered_and_justified(|ui| {
            ui.label("This book is missing from the bible file.");
        });
        return;
    };
    let chapters = book.chapter_numbers();

    let rects: Vec<(u32, Rect)> = {
        let width = ui.available_width();
        layout::chapter_grid(
            &chapters,
            width,
            0.0,
            CHAPTER_BUTTON_SIZE,
            CHAPTER_BUTTON_SPACING,
        )
    };
    let rows = rects
        .last()
        .map(|(_, rect)| (rect.top() - rect.min.y) / (CHAPTER_BUTTON_SIZE + CHAPTER_BUTTON_SPACING))
        .map_or(0.0, |r| r.floor())
        + 1.0;
    let height = layout::grid_height(rows as usize, CHAPTER_BUTTON_SIZE, CHAPTER_BUTTON_SPACING);

    let mut chosen: Option<u32> = None;
    ui.vertical(|ui| {
        let (area, _response) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), egui::Sense::hover());
        for (chapter, rect) in &rects {
            // Centre the grid in the panel and place the buttons by hand, so
            // they can never grow past the right edge.
            let rect = Rect::from_min_size(
                egui::pos2(area.left() + rect.min.x, area.top() + rect.min.y),
                rect.size(),
            );
            if ui
                .put(rect, egui::Button::new(chapter.to_string()))
                .clicked()
            {
                chosen = Some(*chapter);
            }
        }
    });

    if let Some(chapter) = chosen {
        app.open_chapter(chapter);
    }
}

/// Settings: currently just the zoom, applied to every screen.
pub fn show_settings(ui: &mut Ui, app: &mut App) {
    ui.add_space(8.0);
    ui.heading("Settings");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Text size");
        if ui.button("−").clicked() {
            app.zoom_out();
        }
        ui.label(format!("{:.0}%", app.zoom * 100.0));
        if ui.button("+").clicked() {
            app.zoom_in();
        }
    });

    ui.add_space(8.0);
    ui.label(format!(
        "Zoom can be set between {:.0}% and {:.0}% in steps of {:.0}%.",
        MIN_ZOOM * 100.0,
        MAX_ZOOM * 100.0,
        ZOOM_STEP * 100.0
    ));
    ui.add_space(8.0);
    ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
        ui.label(
            "Click a word in the text to see its translation and to mark it as seen or known.",
        );
    });
}
