//! Bible Reader: read a chapter, tap a word, see its translation.
//!
//! The window is an `eframe` app; everything interesting lives in
//! [`app`], [`ui`] and [`domain`].

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// The test suite links these same modules, so helpers that only the tests use
// would otherwise be reported as dead code by a plain `cargo build`.
#![cfg_attr(not(test), allow(dead_code))]

mod app;
mod domain;
mod ui;

#[cfg(test)]
mod app_test;

use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bible Reader")
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([480.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Bible Reader",
        options,
        Box::new(|cc| Ok(Box::new(app::AppShell::new(cc)))),
    )
}
