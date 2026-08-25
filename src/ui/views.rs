use macroquad::prelude::*;

use crate::domain::{Bible, WordStateStore, WordStatus};
use crate::ui::layout::{
    AppState, FontContext, Theme, DrawerLayout, ReaderLayout,
    WordTokenLayout,
};

#[derive(Clone)]
pub struct AppStateContext {
    pub state: AppState,
    pub bible: Bible,
    pub word_store: WordStateStore,
    pub theme: Theme,
    pub font_context: Option<FontContext>,
}

impl AppStateContext {
    pub fn new(state: AppState, bible: Bible, word_store: WordStateStore, theme: Theme, font_context: Option<FontContext>) -> Self {
        AppStateContext {
            state,
            bible,
            word_store,
            theme,
            font_context,
        }
    }

    pub fn set_state(&mut self, new_state: AppState) {
        self.state = new_state;
    }

    pub fn get_font(&self) -> &Font {
        &self.font_context.as_ref().unwrap().font
    }

    pub fn get_font_size(&self) -> f32 {
        24.0
    }

    pub fn get_states(&self) -> &WordStateStore {
        &self.word_store
    }
}

pub struct BookSelectionView;

impl BookSelectionView {
    pub fn render(ctx: &AppStateContext, screen_width: f32, screen_height: f32, books: &std::collections::HashMap<String, crate::domain::Book>, scroll_offset: f32) {
        let font = ctx.get_font();
        let font_size = ctx.get_font_size();
        let theme = &ctx.theme;

        clear_background(theme.background_color);

        let title = "Books";
        let title_metrics = measure_text(title, Some(font), font_size as u16, 1.0);
        let title_x = (screen_width - title_metrics.width) / 2.0;
        let title_y = 100.0;

        draw_text(
            title,
            title_x,
            title_y,
            font_size,
            theme.text_color,
        );

        let underline_y = title_y + 10.0;
        draw_line(
            title_x - 20.0,
            underline_y,
            title_x + title_metrics.width + 20.0,
            underline_y,
            2.0,
            theme.seen_color,
        );

        let book_list_y = 150.0;
        let book_height = 30.0;
        
        let books_vec: Vec<_> = books.values().collect();
        let _max_visible = ((screen_height - book_list_y - 20.0) / book_height).floor() as usize;
        let total_book_height = books_vec.len() as f32 * book_height;
        let max_scroll = (total_book_height - (screen_height - book_list_y - 20.0)).max(0.0);
        let visible_start = (scroll_offset / book_height).floor() as usize;
        let visible_end = ((scroll_offset + screen_height - book_list_y - 20.0) / book_height).ceil() as usize;
        let start_idx = visible_start.min(books_vec.len());
        let end_idx = visible_end.min(books_vec.len());
        
        for i in start_idx..end_idx {
            let book = &books_vec[i];
            let book_y = book_list_y + (i as f32 * book_height) - scroll_offset;
            
            draw_text(
                &book.name,
                50.0,
                book_y + font_size - 4.0,
                font_size - 4.0,
                theme.text_color,
            );
        }
        
        let scrollbar_y = book_list_y;
        let scrollbar_height = screen_height - book_list_y - 20.0;
        let scroll_ratio = if total_book_height > scrollbar_height {
            scrollbar_height / total_book_height
        } else {
            1.0
        };
        let thumb_height = scrollbar_height * scroll_ratio;
        let thumb_y = scrollbar_y + (scroll_offset / max_scroll.max(1.0)) * (scrollbar_height - thumb_height);
        
        draw_rectangle(
            screen_width - 20.0,
            scrollbar_y,
            10.0,
            scrollbar_height,
            theme.seen_color,
        );
        
        draw_rectangle(
            screen_width - 20.0,
            thumb_y,
            10.0,
            thumb_height,
            theme.unseen_color,
        );
    }
}

pub struct ChapterSelectionView;

impl ChapterSelectionView {
    pub fn render(ctx: &AppStateContext, screen_width: f32, _screen_height: f32, _book_id: &str) {
        let font = ctx.get_font();
        let font_size = ctx.get_font_size();
        let theme = &ctx.theme;

        clear_background(theme.background_color);

        let title = "Chapters";
        let title_metrics = measure_text(title, Some(font), font_size as u16, 1.0);
        let title_x = (screen_width - title_metrics.width) / 2.0;
        let title_y = 100.0;

        draw_text(
            title,
            title_x,
            title_y,
            font_size,
            theme.text_color,
        );

        let underline_y = title_y + 10.0;
        draw_line(
            title_x - 20.0,
            underline_y,
            title_x + title_metrics.width + 20.0,
            underline_y,
            2.0,
            theme.seen_color,
        );

        let max_chapters = 50;
        let button_size = 60.0;
        let spacing = 20.0;
        let buttons_per_row: f32 = 6.0;
        let start_x = (screen_width - (buttons_per_row * (button_size + spacing) - spacing)) / 2.0;
        let start_y = 150.0;

        for chapter in 1..=max_chapters {
            let row = (chapter - 1) / buttons_per_row as u32;
            let col = (chapter - 1) % buttons_per_row as u32;

            let x = start_x + col as f32 * (button_size + spacing);
            let y = start_y + row as f32 * (button_size + spacing);

            let button_rect = Rect {
                x,
                y,
                w: button_size,
                h: button_size,
            };

            draw_rectangle(
                button_rect.x,
                button_rect.y,
                button_rect.w,
                button_rect.h,
                theme.panel_color,
            );

            draw_text(
                &chapter.to_string(),
                x + button_size / 2.0 - 10.0,
                y + button_size / 2.0 + 8.0,
                font_size - 4.0,
                theme.text_color,
            );

            if is_mouse_button_down(MouseButton::Left) && ChapterSelectionView::is_mouse_over(button_rect) {
                debug!("Chapter {} clicked", chapter);
            }
        }
    }

    fn is_mouse_over(rect: Rect) -> bool {
        let mouse_pos = mouse_position();
        let mouse_x = mouse_pos.0;
        let mouse_y = mouse_pos.1;

        mouse_x >= rect.x
            && mouse_x <= rect.x + rect.w
            && mouse_y >= rect.y
            && mouse_y <= rect.y + rect.h
    }
}

pub struct ReaderView;

impl ReaderView {
    pub fn render(
        ctx: &AppStateContext,
        screen_width: f32,
        screen_height: f32,
        book_id: &str,
        chapter: u32,
        layout: Option<&ReaderLayout>,
        word_layout: Option<&WordTokenLayout>,
    ) {
        let font = ctx.get_font();
        let font_size = ctx.get_font_size();
        let theme = &ctx.theme;
        let states = ctx.get_states();

        clear_background(theme.background_color);

        let _reader_x = 50.0;
        let _reader_y = 100.0;
        let _max_width = screen_width - 100.0;

        let book = ctx.bible.books.get(book_id);
        let (_text, tokens) = if let Some(b) = book {
            if let Some(chapter_data) = b.chapters.get(&chapter) {
                let full_text = chapter_data.verses.iter().map(|v| v.text.as_str()).collect::<Vec<_>>().join(" ");
                let tokens = crate::domain::TextTokenizer::tokenize(&full_text);
                (full_text, tokens)
            } else {
                ("Chapter not found".to_string(), Vec::new())
            }
        } else {
            ("Book not found".to_string(), Vec::new())
        };

        let layout = layout.expect("ReaderLayout should be provided");

        for word_layout in &layout.word_layouts {
            let status = states.get_status(&word_layout.token.to_string());
            let color = match status {
                WordStatus::Unseen => theme.unseen_color,
                WordStatus::Seen => theme.seen_color,
                WordStatus::Known => theme.text_color,
            };

            draw_rectangle(
                word_layout.highlight_rect.x,
                word_layout.highlight_rect.y,
                word_layout.highlight_rect.w,
                word_layout.highlight_rect.h,
                color,
            );
        }

        for word_layout in &layout.word_layouts {
            let text_to_draw = word_layout.token.to_string();
            draw_text(
                &text_to_draw,
                word_layout.rect.x,
                word_layout.rect.y + font_size,
                font_size,
                theme.text_color,
            );
        }

        if let Some(selected) = word_layout {
            let status = states.get_status(&selected.token.to_string());
            draw_rectangle(
                selected.highlight_rect.x,
                selected.highlight_rect.y,
                selected.highlight_rect.w,
                selected.highlight_rect.h,
                if status == WordStatus::Unseen {
                    theme.unseen_color
                } else if status == WordStatus::Seen {
                    theme.seen_color
                } else {
                    theme.text_color
                },
            );
        }

        if let Some(selected) = word_layout {
            Self::render_drawer(ctx, screen_width, screen_height, selected, &tokens);
        }
    }

    pub fn render_drawer(
        ctx: &AppStateContext,
        screen_width: f32,
        screen_height: f32,
        word_layout: &WordTokenLayout,
        _all_tokens: &[crate::domain::Token],
    ) {
        let _font = ctx.get_font();
        let font_size = ctx.get_font_size();
        let theme = &ctx.theme;

        let drawer = DrawerLayout::new(screen_width, screen_height);

        draw_rectangle(
            drawer.rect.x,
            drawer.rect.y,
            drawer.rect.w,
            drawer.rect.h,
            theme.panel_color,
        );

        let word_text = word_layout.token.to_string();
        draw_text(
            &word_text,
            drawer.title_rect.x,
            drawer.title_rect.y + font_size,
            font_size,
            theme.text_color,
        );

        let translation = Self::get_translation(&word_text);
        draw_text(
            translation,
            drawer.translation_rect.x,
            drawer.translation_rect.y + font_size,
            font_size - 4.0,
            theme.text_color,
        );

        for button in &drawer.buttons {
            draw_rectangle(
                button.rect.x,
                button.rect.y,
                button.rect.w,
                button.rect.h,
                theme.unseen_color,
            );

            draw_text(
                &button.label,
                button.rect.x + 10.0,
                button.rect.y + button.rect.h / 2.0 + 8.0,
                font_size - 8.0,
                theme.text_color,
            );
        }
    }

    fn get_translation(word: &str) -> &'static str {
        match word {
            "В" => "In",
            "начале" => "beginning",
            "сотворил" => "created",
            "Бог" => "God",
            _ => "translation not found",
        }
    }
}
