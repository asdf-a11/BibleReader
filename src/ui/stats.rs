//! The stats screen: how many words are known and seen, and a chart of each
//! total over time.

use egui::containers::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Ui,
};

use crate::app::App;
use crate::domain::{days_from_date, CountHistory};
use crate::ui::theme::Theme;

pub const CHART_HEIGHT: f32 = 220.0;
/// Room on the left for the count labels and below for the dates.
const AXIS_LEFT: f32 = 48.0;
const AXIS_BOTTOM: f32 = 24.0;
const AXIS_FONT_SIZE: f32 = 13.0;
const GRID_LINES: usize = 4;

pub fn show(ui: &mut Ui, app: &mut App) {
    let theme = Theme::new();
    ScrollArea::vertical()
        .id_salt("stats")
        .auto_shrink([false, false])
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .scroll_source(ScrollSource::ALL)
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                total(ui, "Known words", app.known_count());
                ui.add_space(32.0);
                total(ui, "Seen words", app.seen_count());
            });

            ui.add_space(16.0);
            ui.heading("Known words over time");
            line_chart(ui, &app.history.known, theme.known);

            ui.add_space(16.0);
            ui.heading("Seen words over time");
            line_chart(ui, &app.history.seen, theme.seen);
            ui.add_space(8.0);
        });
}

fn total(ui: &mut Ui, label: &str, count: usize) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).color(Theme::new().dim_text));
        ui.label(RichText::new(count.to_string()).size(32.0).strong());
    });
}

/// The points of a history as (days since 1970, count), skipping malformed
/// dates.
pub fn chart_points(history: &CountHistory) -> Vec<(i64, usize, &str)> {
    history
        .entries
        .iter()
        .filter_map(|e| Some((days_from_date(&e.date)?, e.count, e.date.as_str())))
        .collect()
}

/// A line chart of one history, with a tooltip on hover.
fn line_chart(ui: &mut Ui, history: &CountHistory, color: Color32) {
    let theme = Theme::new();
    let points = chart_points(history);
    let width = ui.available_width().max(AXIS_LEFT + 40.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, CHART_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);

    if points.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No data yet",
            FontId::proportional(AXIS_FONT_SIZE),
            theme.dim_text,
        );
        return;
    }

    let plot = Rect::from_min_max(
        pos2(rect.left() + AXIS_LEFT, rect.top() + 8.0),
        pos2(rect.right() - 12.0, rect.bottom() - AXIS_BOTTOM),
    );
    let first_day = points[0].0;
    let last_day = points[points.len() - 1].0;
    let max_count = points.iter().map(|p| p.1).max().unwrap_or(0);
    let y_max = nice_max(max_count);

    let to_screen = |day: i64, count: usize| -> Pos2 {
        let x = if last_day > first_day {
            (day - first_day) as f32 / (last_day - first_day) as f32
        } else {
            0.5
        };
        let y = count as f32 / y_max as f32;
        pos2(
            plot.left() + x * plot.width(),
            plot.bottom() - y * plot.height(),
        )
    };

    // Recessive grid and count labels.
    let grid = Stroke::new(1.0, theme.panel);
    let font = FontId::proportional(AXIS_FONT_SIZE);
    for i in 0..=GRID_LINES {
        let value = y_max * i / GRID_LINES;
        let y = plot.bottom() - plot.height() * i as f32 / GRID_LINES as f32;
        painter.hline(plot.x_range(), y, grid);
        painter.text(
            pos2(plot.left() - 8.0, y),
            Align2::RIGHT_CENTER,
            value.to_string(),
            font.clone(),
            theme.dim_text,
        );
    }

    // First and last date under the axis.
    let date_y = plot.bottom() + 4.0;
    painter.text(
        pos2(plot.left(), date_y),
        Align2::LEFT_TOP,
        points[0].2,
        font.clone(),
        theme.dim_text,
    );
    if points.len() > 1 {
        painter.text(
            pos2(plot.right(), date_y),
            Align2::RIGHT_TOP,
            points[points.len() - 1].2,
            font.clone(),
            theme.dim_text,
        );
    }

    let screen: Vec<Pos2> = points.iter().map(|p| to_screen(p.0, p.1)).collect();
    if screen.len() > 1 {
        painter.add(egui::Shape::line(screen.clone(), Stroke::new(2.0, color)));
    }
    for p in &screen {
        painter.circle_filled(*p, 4.0, color);
    }

    // Hover: crosshair on the nearest point plus its date and count.
    if let Some(pointer) = response.hover_pos() {
        let nearest = screen
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1.x - pointer.x)
                    .abs()
                    .total_cmp(&(b.1.x - pointer.x).abs())
            })
            .map(|(i, _)| i);
        if let Some(i) = nearest {
            let p = screen[i];
            painter.vline(p.x, plot.y_range(), Stroke::new(1.0, theme.dim_text));
            painter.circle(p, 6.0, color, Stroke::new(2.0, theme.background));
            response.on_hover_ui_at_pointer(|ui| {
                ui.label(RichText::new(points[i].2).color(theme.dim_text));
                ui.label(RichText::new(format!("{} words", points[i].1)).strong());
            });
        }
    }
}

/// The smallest "round" number ≥ `max` that splits evenly into the grid lines,
/// so the axis labels are whole numbers.
pub fn nice_max(max: usize) -> usize {
    let max = max.max(1);
    let step = max.div_ceil(GRID_LINES);
    let magnitude = 10usize.pow((step as f64).log10().floor() as u32);
    let nice_step = [1, 2, 5, 10]
        .iter()
        .map(|m| m * magnitude)
        .find(|&s| s >= step)
        .unwrap_or(10 * magnitude);
    nice_step * GRID_LINES
}
