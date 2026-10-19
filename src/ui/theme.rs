//! Colours for the reader.

use egui::{Color32, Visuals};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Words never looked at.
    pub unseen: Color32,
    /// Words looked at at least once.
    pub seen: Color32,
    /// Known words; only used by the stats chart, known words in the text are
    /// drawn in the plain text colour.
    pub known: Color32,
    pub background: Color32,
    /// Drawers, the toolbar and the translation window.
    pub panel: Color32,
    pub text: Color32,
    /// A word with no translation.
    pub dim_text: Color32,
    /// The English shown under the Russian: a little darker than the text so
    /// the Russian stays in front.
    pub gloss: Color32,
    /// Errors, e.g. a state file that could not be written.
    pub error: Color32,
}

/// Blend `color` towards white by `amount` (0 = unchanged, 1 = white).
pub fn lighten(color: Color32, amount: f32) -> Color32 {
    color.lerp_to_gamma(Color32::WHITE, amount.clamp(0.0, 1.0))
}

/// Blend the theme into egui's dark visuals.
pub fn install(ctx: &egui::Context) {
    let theme = Theme::new();
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        theme.apply(&mut style.visuals);
        // A solid scroll bar that always takes room out of the panel: the text
        // is then never laid out underneath it, and the reader always shows
        // that there is more to read.
        style.spacing.scroll = egui::style::ScrollStyle::solid();
    });
}

impl Theme {
    pub const fn new() -> Self {
        Theme {
            unseen: Color32::from_rgb(0, 172, 200),
            seen: Color32::from_rgb(226, 176, 66),
            known: Color32::from_rgb(110, 200, 120),
            background: Color32::from_rgb(16, 20, 24),
            panel: Color32::from_rgb(28, 34, 41),
            text: Color32::from_rgb(245, 247, 250),
            dim_text: Color32::from_rgb(150, 158, 168),
            gloss: Color32::from_rgb(182, 190, 200),
            error: Color32::from_rgb(255, 120, 120),
        }
    }

    /// Blend the theme into egui's dark visuals.
    pub fn apply(&self, visuals: &mut Visuals) {
        visuals.override_text_color = Some(self.text);
        visuals.panel_fill = self.background;
        visuals.window_fill = self.panel;
        visuals.extreme_bg_color = self.panel;
        visuals.faint_bg_color = self.background;
        visuals.hyperlink_color = self.seen;
        visuals.selection.bg_fill = self.seen.gamma_multiply(0.5);
        visuals.selection.stroke = egui::Stroke::new(1.0, self.seen);
        visuals.window_stroke = egui::Stroke::new(1.0, self.panel);
        visuals.widgets.inactive.bg_fill = self.panel;
        visuals.widgets.inactive.weak_bg_fill = self.panel;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.panel);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text);
        visuals.widgets.hovered.bg_fill = lighten(self.panel, 0.15);
        visuals.widgets.hovered.weak_bg_fill = lighten(self.panel, 0.15);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.dim_text);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, self.text);
        visuals.widgets.active.bg_fill = lighten(self.panel, 0.15);
        visuals.widgets.active.weak_bg_fill = lighten(self.panel, 0.15);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.text);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, self.text);
        visuals.widgets.open.bg_fill = self.panel;
        visuals.error_fg_color = self.error;
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::new()
    }
}
