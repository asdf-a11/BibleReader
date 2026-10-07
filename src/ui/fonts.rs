//! Font registration.
//!
//! egui's built in fonts have no Cyrillic coverage, and this reader is mostly
//! Russian text, so the Noto Sans shipped with the project is registered as the
//! first font of every family. It is embedded in the binary so the app (and
//! the tests) never depend on the working directory.

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId};

/// Name the project font is registered under.
pub const SANS_FONT: &str = "bible_reader_sans";

const SANS_TTF: &[u8] = include_bytes!("../../NotoSans-VariableFont_wdth,wght.ttf");

/// Install the project font as the preferred font of every family.
pub fn install(ctx: &Context) {
    let mut definitions = FontDefinitions::default();
    definitions
        .font_data
        .insert(SANS_FONT.to_owned(), FontData::from_static(SANS_TTF).into());
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        definitions
            .families
            .entry(family)
            .or_default()
            .insert(0, SANS_FONT.to_owned());
    }
    ctx.set_fonts(definitions);
}

/// Measure the advance width of `text` without wrapping.
pub fn measure(ctx: &Context, text: &str, size: f32) -> f32 {
    let font_id = FontId::proportional(size);
    ctx.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id, Color32::PLACEHOLDER)
            .size()
            .x
    })
}

/// A `measure` closure for the pure layout code in [`crate::ui::layout`].
pub fn measurer(ctx: &Context) -> impl Fn(&str, f32) -> f32 + '_ {
    let ctx = ctx.clone();
    move |text: &str, size: f32| measure(&ctx, text, size)
}
