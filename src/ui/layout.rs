use macroquad::math::{Rect, Vec2, vec2};
use macroquad::prelude::Color;

use crate::domain::{Token, WordStatus};
use crate::ui::scroll::{SCROLLBAR_MARGIN, SCROLLBAR_WIDTH};

pub const SCREEN_BOTTOM_MARGIN: f32 = 20.0;

pub const BOOKS_TOP: f32 = 150.0;
pub const BOOKS_LEFT: f32 = 50.0;
pub const BOOK_HEIGHT: f32 = 30.0;

pub const GRID_TOP: f32 = 150.0;
pub const BUTTON_SIZE: f32 = 60.0;
pub const GRID_SPACING: f32 = 20.0;
pub const GRID_COLS: u32 = 6;

pub const READER_TOP: f32 = 100.0;
pub const READER_LEFT: f32 = 50.0;
pub const READER_FONT_SIZE: f32 = 24.0;
pub const WORD_GAP: f32 = 8.0;
pub const WORD_PADDING: f32 = 2.0;

pub const FONT_PATH: &str = "NotoSans-VariableFont_wdth,wght.ttf";
pub const BIBLE_PATH: &str = "rst.json";
pub const TRANSLATIONS_PATH: &str = "translations.json";

/// The region holding the vertical scrollbar for a screen.
pub fn scrollbar_viewport(screen: Vec2, top: f32) -> Rect {
    Rect::new(
        screen.x - SCROLLBAR_WIDTH - SCROLLBAR_MARGIN,
        top,
        SCROLLBAR_WIDTH,
        (screen.y - top - SCREEN_BOTTOM_MARGIN).max(0.0),
    )
}

pub fn back_button_rect() -> Rect {
    Rect::new(16.0, 16.0, 80.0, 36.0)
}

pub fn list_content_height(book_count: usize) -> f32 {
    book_count as f32 * BOOK_HEIGHT
}

/// Rows are laid out left-to-right, top-to-bottom in a fixed-width grid.
pub fn chapter_buttons(chapters: &[u32], screen_width: f32, top: f32) -> Vec<(u32, Rect)> {
    let row_width = GRID_COLS as f32 * (BUTTON_SIZE + GRID_SPACING) - GRID_SPACING;
    let start_x = (screen_width - row_width).max(0.0) / 2.0;

    chapters
        .iter()
        .enumerate()
        .map(|(index, &chapter)| {
            let row = index as u32 / GRID_COLS;
            let col = index as u32 % GRID_COLS;
            let x = start_x + col as f32 * (BUTTON_SIZE + GRID_SPACING);
            let y = top + row as f32 * (BUTTON_SIZE + GRID_SPACING);
            (chapter, Rect::new(x, y, BUTTON_SIZE, BUTTON_SIZE))
        })
        .collect()
}

pub fn grid_content_height(button_count: usize) -> f32 {
    let rows = if button_count == 0 {
        1
    } else {
        button_count.div_ceil(GRID_COLS as usize)
    };
    rows as f32 * (BUTTON_SIZE + GRID_SPACING)
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub unseen_color: Color,
    pub seen_color: Color,
    pub background_color: Color,
    pub panel_color: Color,
    pub text_color: Color,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            unseen_color: Color::from_rgba(0, 229, 255, 128),
            seen_color: Color::from_rgba(255, 213, 79, 128),
            background_color: Color::from_rgba(16, 20, 24, 255),
            panel_color: Color::from_rgba(24, 30, 36, 255),
            text_color: Color::from_rgba(255, 255, 255, 255),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute_token_width(
        token: &Token,
        measure: &dyn Fn(&str, f32) -> f32,
        font_size: f32,
    ) -> f32 {
        match token {
            Token::Word { original, .. } => measure(original, font_size),
            Token::Punctuation(p) => measure(p, font_size),
        }
    }

    /// Wrap tokens into lines that each fit within `max_w`.
    pub fn compute_line_wrapping(
        tokens: &[Token],
        max_w: f32,
        measure: &dyn Fn(&str, f32) -> f32,
        font_size: f32,
    ) -> Vec<Vec<Token>> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_width = 0.0;

        for token in tokens {
            let token_width = Self::compute_token_width(token, measure, font_size);
            if current_width + token_width > max_w && !current_line.is_empty() {
                lines.push(std::mem::take(&mut current_line));
                current_width = 0.0;
            }
            current_width += token_width;
            current_line.push(token.clone());
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
        lines
    }
}

#[derive(Debug, Clone)]
pub struct WordTokenLayout {
    pub token: Token,
    pub rect: Rect,
    pub highlight_rect: Rect,
}

pub struct ReaderLayout {
    pub word_layouts: Vec<WordTokenLayout>,
    pub content_height: f32,
}

impl ReaderLayout {
    pub fn new(
        tokens: &[Token],
        font_size: f32,
        start_x: f32,
        start_y: f32,
        max_w: f32,
        measure: &dyn Fn(&str, f32) -> f32,
    ) -> Self {
        let lines = LayoutEngine::compute_line_wrapping(tokens, max_w, measure, font_size);
        let line_h = font_size * 1.5;
        let line_height = font_size * 1.2;

        let mut word_layouts = Vec::new();
        for (line_index, line) in lines.iter().enumerate() {
            let mut current_x = start_x;
            let y = start_y + line_index as f32 * line_h;

            for token in line {
                let width = LayoutEngine::compute_token_width(token, measure, font_size);
                let rect = Rect::new(current_x, y, width, line_height);
                let highlight_rect = Rect::new(
                    rect.x - WORD_PADDING,
                    rect.y - WORD_PADDING,
                    rect.w + WORD_PADDING * 2.0,
                    rect.h + WORD_PADDING * 2.0,
                );
                word_layouts.push(WordTokenLayout {
                    token: token.clone(),
                    rect,
                    highlight_rect,
                });
                current_x += width + WORD_GAP;
            }
        }

        let line_count = lines.len();
        ReaderLayout {
            word_layouts,
            content_height: start_y + line_count as f32 * line_h,
        }
    }

    /// Hit-test in content coordinates (i.e. adjusted for the scroll offset)
    /// against clickable word boxes. Punctuation is not clickable.
    pub fn get_word_at_position(&self, x: f32, y: f32) -> Option<&WordTokenLayout> {
        self.word_layouts
            .iter()
            .find(|layout| layout.token.is_word() && layout.highlight_rect.contains(vec2(x, y)))
    }
}

#[derive(Debug, Clone)]
pub struct ButtonLayout {
    pub label: String,
    pub rect: Rect,
    pub status: WordStatus,
}

pub struct DrawerLayout {
    pub rect: Rect,
    pub title_rect: Rect,
    pub translation_rect: Rect,
    pub buttons: Vec<ButtonLayout>,
}

impl DrawerLayout {
    /// Bottom section of the screen holding the selected word, its translation,
    /// and the three status buttons.
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        let drawer_h = if screen_h < 500.0 {
            screen_h * 0.5
        } else {
            screen_h * 0.333
        };
        let drawer_y = screen_h - drawer_h;

        let rect = Rect::new(0.0, drawer_y, screen_w, drawer_h);
        let title_rect = Rect::new(20.0, drawer_y + 20.0, screen_w - 40.0, 30.0);
        let translation_rect = Rect::new(20.0, drawer_y + 60.0, screen_w - 40.0, 60.0);

        let button_width = 100.0;
        let button_height = 40.0;
        let gap = 10.0;
        let total_width = button_width * 3.0 + gap * 2.0;
        let start_x = ((screen_w - total_width) / 2.0).max(0.0);
        let button_y = drawer_y + drawer_h - 70.0;

        let make_button = |label: &str, index: u32, status: WordStatus| ButtonLayout {
            label: label.to_string(),
            rect: Rect::new(
                start_x + index as f32 * (button_width + gap),
                button_y,
                button_width,
                button_height,
            ),
            status,
        };

        let buttons = vec![
            make_button("Unseen", 0, WordStatus::Unseen),
            make_button("Seen", 1, WordStatus::Seen),
            make_button("Known", 2, WordStatus::Known),
        ];

        DrawerLayout {
            rect,
            title_rect,
            translation_rect,
            buttons,
        }
    }

    pub fn is_point_in_button(&self, x: f32, y: f32) -> Option<&ButtonLayout> {
        self.buttons
            .iter()
            .find(|btn| btn.rect.contains(vec2(x, y)))
    }
}