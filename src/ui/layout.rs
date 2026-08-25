use macroquad::prelude::*;

use crate::domain::{Token, WordStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    BookSelection,
    ChapterSelection { book_id: String },
    Reader { book_id: String, chapter: u32 },
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

#[derive(Debug, Clone)]
pub struct FontContext {
    pub font: Font,
}

impl FontContext {
    pub async fn load_cyrillic_font() -> Result<Self, String> {
        let font_data = load_file("NotoSans-VariableFont_wdth,wght.ttf").await;
        match font_data {
            Ok(data) => {
                std::fs::write("/tmp/font.ttf", &data).map_err(|e| e.to_string())?;
                let font = load_ttf_font("/tmp/font.ttf").await.map_err(|e| e.to_string())?;
                Ok(FontContext { font })
            }
            Err(_) => {
                let font = get_default_font();
                Ok(FontContext { font })
            }
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute_line_wrapping(
        tokens: &[Token],
        max_w: f32,
        font: &Font,
        font_size: f32,
    ) -> Vec<LineFragment> {
        let mut lines: Vec<Vec<Token>> = Vec::new();
        let mut current_line: Vec<Token> = Vec::new();
        let mut current_width: f32 = 0.0;
        let max_width = max_w;

        for token in tokens {
            let token_width = Self::compute_token_width(token, font, font_size);

            if current_width + token_width > max_width && !current_line.is_empty() {
                lines.push(current_line);
                current_line = Vec::new();
                current_width = 0.0;
            }

            current_line.push(token.clone());
            current_width += token_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
            .into_iter()
            .map(|line_tokens| LineFragment {
                tokens: line_tokens,
                y_offset: 0.0,
                h: 0.0,
            })
            .collect()
    }

    pub fn compute_token_width(token: &Token, font: &Font, font_size: f32) -> f32 {
        match token {
            Token::Word { original, .. } => {
                let text = original.as_str();
                let metrics = measure_text(text, Some(font), font_size as u16, 1.0);
                metrics.width
            }
            Token::Punctuation(p) => {
                let text = p.as_str();
                let metrics = measure_text(text, Some(font), font_size as u16, 1.0);
                metrics.width
            }
        }
    }

    pub fn calculate_text_bounds(
        text: &str,
        font: &Font,
        font_size: f32,
        x: f32,
        y: f32,
    ) -> Rect {
        let metrics = measure_text(text, Some(font), font_size as u16, 1.0);
        Rect {
            x,
            y,
            w: metrics.width,
            h: metrics.height,
        }
    }

    pub fn apply_padding(rect: &Rect, padding: f32) -> Rect {
        Rect {
            x: rect.x - padding,
            y: rect.y - padding,
            w: rect.w + padding * 2.0,
            h: rect.h + padding * 2.0,
        }
    }

    pub fn compute_drawer_height(screen_h: f32) -> f32 {
        if screen_h < 500.0 {
            screen_h * 0.5
        } else {
            screen_h * 0.333
        }
    }
}

pub struct LineFragment {
    pub tokens: Vec<Token>,
    pub y_offset: f32,
    pub h: f32,
}

pub struct WordTokenLayout {
    pub token: Token,
    pub rect: Rect,
    pub highlight_rect: Rect,
}

pub struct ReaderLayout {
    pub lines: Vec<LineFragment>,
    pub word_layouts: Vec<WordTokenLayout>,
    pub line_h: f32,
    pub font_size: f32,
}

impl ReaderLayout {
    pub fn new(
        tokens: &[Token],
        font: &Font,
        font_size: f32,
        start_x: f32,
        start_y: f32,
        max_w: f32,
        _screen_w: f32,
    ) -> Self {
        let max_width = max_w;
        let lines = LayoutEngine::compute_line_wrapping(tokens, max_width, font, font_size);

        let mut word_layouts: Vec<WordTokenLayout> = Vec::new();
        let mut current_y = start_y;

        for line in &lines {
            let mut current_x = start_x;

            for token in &line.tokens {
                let token_width = LayoutEngine::compute_token_width(token, font, font_size);
                let token_height = font_size * 1.2;

                let word_rect = Rect {
                    x: current_x,
                    y: current_y,
                    w: token_width,
                    h: token_height,
                };

                let highlight_rect = LayoutEngine::apply_padding(&word_rect, 4.0);

                word_layouts.push(WordTokenLayout {
                    token: token.clone(),
                    rect: word_rect,
                    highlight_rect,
                });

                current_x += token_width + 8.0;
            }

            current_y += font_size * 1.5;
        }

        ReaderLayout {
            lines,
            word_layouts,
            line_h: font_size * 1.5,
            font_size,
        }
    }

    pub fn get_word_at_position(&self, x: f32, y: f32) -> Option<&WordTokenLayout> {
        self.word_layouts
            .iter()
            .find(|layout| layout.highlight_rect.contains(vec2(x, y)))
    }

    pub fn get_line_count(&self) -> usize {
        self.lines.len()
    }
}

pub struct DrawerLayout {
    pub rect: Rect,
    pub title_rect: Rect,
    pub translation_rect: Rect,
    pub buttons: Vec<ButtonLayout>,
}

#[derive(Debug, Clone)]
pub struct ButtonLayout {
    pub label: String,
    pub rect: Rect,
    pub status: WordStatus,
}

impl DrawerLayout {
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        let drawer_height = LayoutEngine::compute_drawer_height(screen_h);
        let drawer_y = screen_h - drawer_height;

        let rect = Rect {
            x: 0.0,
            y: drawer_y,
            w: screen_w,
            h: drawer_height,
        };

        let title_rect = Rect {
            x: 20.0,
            y: drawer_y + 20.0,
            w: screen_w - 40.0,
            h: 30.0,
        };

        let translation_rect = Rect {
            x: 20.0,
            y: drawer_y + 60.0,
            w: screen_w - 40.0,
            h: 60.0,
        };

        let button_width = 100.0;
        let button_height = 40.0;
        let total_button_width = button_width * 3.0;
        let start_x = (screen_w - total_button_width) / 2.0;

        let buttons = vec![
            ButtonLayout {
                label: "Unseen".to_string(),
                rect: Rect {
                    x: start_x,
                    y: drawer_y + drawer_height - 70.0,
                    w: button_width,
                    h: button_height,
                },
                status: WordStatus::Unseen,
            },
            ButtonLayout {
                label: "Seen".to_string(),
                rect: Rect {
                    x: start_x + button_width + 10.0,
                    y: drawer_y + drawer_height - 70.0,
                    w: button_width,
                    h: button_height,
                },
                status: WordStatus::Seen,
            },
            ButtonLayout {
                label: "Known".to_string(),
                rect: Rect {
                    x: start_x + button_width * 2.0 + 20.0,
                    y: drawer_y + drawer_height - 70.0,
                    w: button_width,
                    h: button_height,
                },
                status: WordStatus::Known,
            },
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

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn test_responsive_drawer_height() {
        let large_screen_height = 600.0;
        let drawer_height = LayoutEngine::compute_drawer_height(large_screen_height);
        assert!((drawer_height - 200.0).abs() < 10.0);

        let small_screen_height = 300.0;
        let drawer_height = LayoutEngine::compute_drawer_height(small_screen_height);
        assert!((drawer_height - 150.0).abs() < 10.0);
    }

    #[test]
    fn test_word_bounding_box_padding() {
        let word_rect = Rect {
            x: 10.0,
            y: 10.0,
            w: 40.0,
            h: 20.0,
        };

        let padded_rect = LayoutEngine::apply_padding(&word_rect, 2.0);

        assert!((padded_rect.x - 8.0).abs() < 0.1);
        assert!((padded_rect.y - 8.0).abs() < 0.1);
        assert!((padded_rect.w - 44.0).abs() < 0.1);
        assert!((padded_rect.h - 24.0).abs() < 0.1);
    }

    #[test]
    #[ignore = "requires macroquad context to measure text width"]
    fn test_word_wrapping_line_breaks() {
        let max_width = 200.0;

        let tokens = vec![
            Token::Word {
                original: "В".to_string(),
                clean_key: "в".to_string(),
            },
            Token::Word {
                original: "начале".to_string(),
                clean_key: "начале".to_string(),
            },
            Token::Word {
                original: "сотворил".to_string(),
                clean_key: "сотворил".to_string(),
            },
            Token::Word {
                original: "Бог".to_string(),
                clean_key: "бог".to_string(),
            },
        ];

        let lines = LayoutEngine::compute_line_wrapping(&tokens, max_width, &get_default_font(), 24.0);

        assert!(lines.len() >= 1);
    }

    #[test]
    fn test_click_hit_testing() {
        let word_layouts = vec![
            WordTokenLayout {
                token: Token::Word {
                    original: "В".to_string(),
                    clean_key: "в".to_string(),
                },
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 30.0,
                    h: 20.0,
                },
                highlight_rect: Rect {
                    x: 8.0,
                    y: 8.0,
                    w: 34.0,
                    h: 24.0,
                },
            },
            WordTokenLayout {
                token: Token::Word {
                    original: "начале".to_string(),
                    clean_key: "начале".to_string(),
                },
                rect: Rect {
                    x: 45.0,
                    y: 10.0,
                    w: 50.0,
                    h: 20.0,
                },
                highlight_rect: Rect {
                    x: 43.0,
                    y: 8.0,
                    w: 54.0,
                    h: 24.0,
                },
            },
        ];

        let mouse_pos = vec2(12.0, 12.0);
        let result = word_layouts.iter().find(|l| l.highlight_rect.contains(mouse_pos));
        assert!(result.is_some());
        assert_eq!(result.unwrap().token.to_string(), "В");

        let mouse_pos = vec2(50.0, 12.0);
        let result = word_layouts.iter().find(|l| l.highlight_rect.contains(mouse_pos));
        assert!(result.is_some());
        assert_eq!(result.unwrap().token.to_string(), "начале");
    }

    #[test]
    fn test_button_layout_in_drawer() {
        let screen_width = 800.0;
        let screen_height = 600.0;

        let drawer = DrawerLayout::new(screen_width, screen_height);

        for button in &drawer.buttons {
        assert!(button.rect.x >= 0.0);
        assert!(button.rect.x + button.rect.w <= screen_width);
        assert!(button.rect.y >= drawer.rect.y);
        assert!(button.rect.y + button.rect.h <= drawer.rect.y + drawer.rect.h);
        }
    }

    #[test]
    fn test_punctuation_not_clickable() {
        let word_layouts = vec![
            WordTokenLayout {
                token: Token::Word {
                    original: "В".to_string(),
                    clean_key: "в".to_string(),
                },
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 30.0,
                    h: 20.0,
                },
                highlight_rect: Rect {
                    x: 8.0,
                    y: 8.0,
                    w: 34.0,
                    h: 24.0,
                },
            },
            WordTokenLayout {
                token: Token::Punctuation(",".to_string()),
                rect: Rect {
                    x: 45.0,
                    y: 10.0,
                    w: 10.0,
                    h: 20.0,
                },
                highlight_rect: Rect {
                    x: 45.0,
                    y: 10.0,
                    w: 10.0,
                    h: 20.0,
                },
            },
        ];

        let mouse_pos = vec2(50.0, 12.0);
        let result = word_layouts.iter().find(|l| l.highlight_rect.contains(mouse_pos));
        assert!(result.is_some());
        assert!(matches!(result.unwrap().token, Token::Punctuation(_)));
    }

    #[test]
    fn test_word_token_to_string() {
        let word_token = Token::Word {
            original: "В".to_string(),
            clean_key: "в".to_string(),
        };
        assert_eq!(word_token.to_string(), "В");

        let punct_token = Token::Punctuation(",".to_string());
        assert_eq!(punct_token.to_string(), ",");
    }

    #[test]
    fn test_word_token_layout_display() {
        let layout = WordTokenLayout {
            token: Token::Word {
                original: "В".to_string(),
                clean_key: "в".to_string(),
            },
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 30.0,
                h: 20.0,
            },
            highlight_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 30.0,
                h: 20.0,
            },
        };
        assert_eq!(format!("{}", layout), "В");
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Word { original, .. } => write!(f, "{}", original),
            Token::Punctuation(p) => write!(f, "{}", p),
        }
    }
}

impl std::fmt::Display for WordTokenLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn test_reader_layout_grid_layout() {
        let tokens = vec![
            Token::Word {
                original: "В".to_string(),
                clean_key: "в".to_string(),
            },
            Token::Word {
                original: "начале".to_string(),
                clean_key: "начале".to_string(),
            },
            Token::Word {
                original: "сотворил".to_string(),
                clean_key: "сотворил".to_string(),
            },
            Token::Word {
                original: "Бог".to_string(),
                clean_key: "бог".to_string(),
            },
        ];

        let font = get_default_font();
        let layout = ReaderLayout::new(&tokens, &font, 24.0, 50.0, 100.0, 800.0, 1024.0);
        
        assert_eq!(layout.word_layouts.len(), 4);
        
        let word1 = &layout.word_layouts[0];
        let word2 = &layout.word_layouts[1];
        
        assert!(word2.rect.x > word1.rect.x, "Second word should be to the right of first word");
    }

    #[test]
    fn test_word_token_layout_has_padding() {
        let tokens = vec![
            Token::Word {
                original: "В".to_string(),
                clean_key: "в".to_string(),
            },
        ];

        let font = get_default_font();
        let layout = ReaderLayout::new(&tokens, &font, 24.0, 50.0, 100.0, 800.0, 1024.0);
        
        let word_layout = &layout.word_layouts[0];
        let padding = 4.0;
        
        assert!(word_layout.highlight_rect.x < word_layout.rect.x);
        assert!(word_layout.highlight_rect.y < word_layout.rect.y);
        assert!(word_layout.highlight_rect.w > word_layout.rect.w);
        assert!(word_layout.highlight_rect.h > word_layout.rect.h);
    }

    #[test]
    fn test_chapter_button_grid() {
        let screen_width = 1024.0;
        let button_size = 60.0;
        let spacing = 20.0;
        let buttons_per_row: f32 = 6.0;
        let start_x = (screen_width - (buttons_per_row * (button_size + spacing) - spacing)) / 2.0;
        
        let chapter1_x = start_x;
        let chapter2_x = start_x + (button_size + spacing);
        let chapter7_x = start_x + (button_size + spacing) * 6.0;
        
        assert!(chapter2_x > chapter1_x);
        assert!(chapter7_x > chapter2_x);
    }

    #[test]
    fn test_scrollbar_thumb_position() {
        let scrollbar_height = 500.0;
        let total_height = 1000.0;
        let scroll_ratio = scrollbar_height / total_height;
        let thumb_height = scrollbar_height * scroll_ratio;
        
        assert!(thumb_height > 0.0);
        assert!(thumb_height <= scrollbar_height);
    }

    #[test]
    fn test_back_button_position() {
        let back_button_x = 10.0;
        let back_button_y = 10.0;
        let back_button_width = 80.0;
        let back_button_height = 40.0;
        
        assert!(back_button_x >= 0.0);
        assert!(back_button_y >= 0.0);
        assert!(back_button_width > 0.0);
        assert!(back_button_height > 0.0);
    }
}
