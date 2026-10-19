//! The egui front end.
//!
//! Every screen is drawn from the pure layout code in [`layout`], which is
//! what keeps the "nothing may stick out of the window" guarantee testable.

pub mod fonts;
pub mod layout;
pub mod lists;
pub mod reader;
pub mod state;
pub mod stats;
pub mod theme;

use egui::{CentralPanel, Panel, RichText, Ui};

use crate::app::App;
use crate::ui::state::AppState;
use crate::ui::theme::Theme;

#[cfg(test)]
mod gui_test;
#[cfg(test)]
mod layout_test;

/// Draw one frame of the whole application.
pub fn show(ui: &mut Ui, app: &mut App) {
    // The colours and the scroll bar style are installed once, when the app is
    // created (see [`crate::ui::theme::install`]), so that even the first frame
    // is drawn the way we want it. Only the zoom has to be pushed every frame,
    // because the user can change it.
    ui.ctx().set_zoom_factor(app.zoom);

    if let Some(error) = app.data.load_error.clone() {
        show_error(ui, &error);
        return;
    }

    show_toolbar(ui, app);

    // The state is cloned so the screens can borrow `app` mutably.
    let current = app.state.clone();
    CentralPanel::default().show(ui, |ui| match current {
        AppState::BookSelection => lists::show_books(ui, app),
        AppState::ChapterSelection { ref book_id } => lists::show_chapters(ui, app, book_id),
        AppState::Reader {
            ref book_id,
            chapter,
        } => reader::show(ui, app, book_id, chapter),
        AppState::Settings { .. } => lists::show_settings(ui, app),
        AppState::Stats { .. } => stats::show(ui, app),
    });
}

/// Toolbar with the back button, the current screen's title and the settings
/// and stats buttons. It is shown on every screen, so it lives outside the central panel.
fn show_toolbar(ui: &mut Ui, app: &mut App) {
    Panel::top("toolbar").show(ui, |ui| {
        ui.horizontal(|ui| {
            let back = ui
                .add_sized([88.0, 30.0], egui::Button::new("Back"))
                .on_hover_text("Back to the previous screen");
            if back.clicked() {
                app.go_back();
            }

            ui.label(RichText::new(app.state.title()).strong());

            if let Some(error) = &app.save_error {
                ui.label(RichText::new(error).color(Theme::new().error));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let settings = ui
                    .add_sized([96.0, 30.0], egui::Button::new("Settings"))
                    .on_hover_text("Settings");
                if settings.clicked() {
                    app.open_settings();
                }
                // Right to left: this lands just left of Settings.
                let stats = ui
                    .add_sized([96.0, 30.0], egui::Button::new("Stats"))
                    .on_hover_text("Known and seen words over time");
                if stats.clicked() {
                    app.open_stats();
                }
            });
        });
    });
}

fn show_error(ui: &mut Ui, error: &str) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(40.0);
        ui.heading("Could not load the bible");
        ui.label(error);
        ui.label(format!(
            "Expected '{}' in the working directory.",
            layout::BIBLE_PATH
        ));
    });
}
