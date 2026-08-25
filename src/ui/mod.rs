use macroquad::prelude::*;

use crate::domain::{load_bible_from_rst, WordStateStore};
pub use crate::ui::layout::{AppState, FontContext, Theme};
pub use crate::ui::layout::{DrawerLayout, ReaderLayout};
pub use crate::ui::views::{AppStateContext, BookSelectionView, ChapterSelectionView, ReaderView};

mod layout;
mod views;

pub struct MouseState {
    pub left: bool,
    pub left_released: bool,
    pub left_pressed: bool,
    pub wheel_y: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            left: false,
            left_released: false,
            left_pressed: false,
            wheel_y: 0.0,
        }
    }

    pub fn update(&mut self) {
        let left_now = is_mouse_button_down(MouseButton::Left);
        self.left_released = !left_now && self.left;
        self.left_pressed = left_now && !self.left;
        self.left = left_now;
        self.wheel_y = mouse_wheel().1;
    }
}

pub async fn run_app() {
    let font_context = match FontContext::load_cyrillic_font().await {
        Ok(fc) => fc,
        Err(e) => {
            error!("Failed to load font: {}", e);
            return;
        }
    };

    let theme = Theme::new();
    let word_store = WordStateStore::new();
    let bible = match load_bible_from_rst("rst.json").await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to load Bible: {}", e);
            return;
        }
    };

    let mut state = AppState::BookSelection;
    let mut mouse = MouseState::new();
    let mut book_scroll_offset_y = 0.0;
    let mut last_word_selected: Option<String> = None;
    let mut _mouse_was_released = true;
    let mut is_scrolling_scrollbar = false;

    //You never loaded the cyrillic font I asked you to so I have done it here
    let font = load_ttf_font("NotoSans-VariableFont_wdth,wght.ttf")
        .await
        .unwrap();
    set_default_font(font);

    loop {
        //draw_text("привет", 20.0, 20.0, 30.0, DARKGRAY);
        //next_frame().await;
        //continue;

        let screen_width = screen_width();
        let screen_height = screen_height();

        mouse.update();

        match state {
            AppState::BookSelection => {
                if is_scrolling_scrollbar {
                    let mouse_y = mouse_position().1;
                    let scroll_area_top = 150.0;
                    let scroll_area_height = screen_height - 150.0 - 20.0;

                    let books: Vec<_> = bible.get_books_in_order();
                    let book_height = 30.0;
                    let total_height = books.len() as f32 * book_height;

                    let max_scroll = (total_height - scroll_area_height).max(0.0);
                    let scroll_y_normalized =
                        ((mouse_y - scroll_area_top) / scroll_area_height).clamp(0.0, 1.0);
                    book_scroll_offset_y = scroll_y_normalized * max_scroll;
                } else {
                    book_scroll_offset_y -= mouse.wheel_y * 15.0;
                    let books: Vec<_> = bible.get_books_in_order();
                    let book_height = 30.0;
                    let scroll_area_height = screen_height - 150.0 - 20.0;
                    let total_height = books.len() as f32 * book_height;
                    let max_scroll = (total_height - scroll_area_height).max(0.0);
                    book_scroll_offset_y = book_scroll_offset_y.clamp(0.0, max_scroll);
                }

                let scrollbar_x = screen_width - 20.0;
                let scrollbar_y = 150.0;
                let scrollbar_height = screen_height - 150.0 - 20.0;

                let books: Vec<_> = bible.get_books_in_order();
                let book_height = 30.0;
                let total_height = books.len() as f32 * book_height;
                let scroll_ratio = if total_height > scrollbar_height {
                    scrollbar_height / total_height
                } else {
                    1.0
                };
                let thumb_height = scrollbar_height * scroll_ratio;
                let max_scroll = (total_height - scrollbar_height).max(0.0);
                let thumb_y = scrollbar_y
                    + (book_scroll_offset_y / max_scroll.max(1.0))
                        * (scrollbar_height - thumb_height);

                let _scrollbar_rect = Rect {
                    x: scrollbar_x,
                    y: scrollbar_y,
                    w: 10.0,
                    h: scrollbar_height,
                };
                let thumb_rect = Rect {
                    x: scrollbar_x,
                    y: thumb_y,
                    w: 10.0,
                    h: thumb_height,
                };

                let mouse_pos = mouse_position();
                if mouse.left_pressed && thumb_rect.contains(vec2(mouse_pos.0, mouse_pos.1)) {
                    is_scrolling_scrollbar = true;
                }

                if mouse.left_released {
                    is_scrolling_scrollbar = false;
                }

                BookSelectionView::render(
                    &AppStateContext::new(
                        state.clone(),
                        bible.clone(),
                        word_store.clone(),
                        theme.clone(),
                        Some(font_context.clone()),
                    ),
                    screen_width,
                    screen_height,
                    &bible.books,
                    book_scroll_offset_y,
                );

                if mouse.left_pressed && !is_scrolling_scrollbar {
                    let mouse_pos = mouse_position();
                    let books: Vec<_> = bible.get_books_in_order();
                    let book_height = 30.0;
                    let scroll_y = 150.0;

                    for (i, book) in books.iter().enumerate() {
                        let book_y = scroll_y + (i as f32 * book_height) - book_scroll_offset_y;
                        if mouse_pos.1 >= book_y && mouse_pos.1 < book_y + book_height {
                            state = AppState::ChapterSelection {
                                book_id: book.id.clone(),
                            };
                            break;
                        }
                    }
                }
            }
            AppState::ChapterSelection { ref book_id } => {
                ChapterSelectionView::render(
                    &AppStateContext::new(
                        state.clone(),
                        bible.clone(),
                        word_store.clone(),
                        theme.clone(),
                        Some(font_context.clone()),
                    ),
                    screen_width,
                    screen_height,
                    book_id,
                );

                if mouse.left_released {
                    let mouse_pos = mouse_position();
                    if mouse_pos.1 > 150.0 && mouse_pos.1 < 400.0 {
                        state = AppState::Reader {
                            book_id: book_id.clone(),
                            chapter: 1,
                        };
                    }
                }

                let back_button_rect = Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 80.0,
                    h: 40.0,
                };

                let mouse_pos = mouse_position();
                if mouse.left_released && back_button_rect.contains(vec2(mouse_pos.0, mouse_pos.1))
                {
                    state = AppState::BookSelection;
                }

                draw_rectangle(
                    back_button_rect.x,
                    back_button_rect.y,
                    back_button_rect.w,
                    back_button_rect.h,
                    theme.seen_color,
                );
                draw_text(
                    "Back",
                    back_button_rect.x + 20.0,
                    back_button_rect.y + 28.0,
                    20.0,
                    theme.text_color,
                );
            }
            AppState::Reader {
                ref book_id,
                chapter,
            } => {
                let book = bible.books.get(book_id).unwrap();
                let chapter_data = book.chapters.get(&chapter).unwrap();
                let text = &chapter_data.verses.first().unwrap().text;
                let tokens = crate::domain::TextTokenizer::tokenize(text);
                let reader_x = 50.0;
                let reader_y = 100.0;
                let max_width = screen_width - 100.0;

                let layout = ReaderLayout::new(
                    &tokens,
                    &font_context.font,
                    24.0,
                    reader_x,
                    reader_y,
                    max_width,
                    screen_width,
                );

                let word_layout = if mouse.left_pressed {
                    let mouse_pos = mouse_position();
                    layout.get_word_at_position(mouse_pos.0, mouse_pos.1)
                } else {
                    None
                };

                let current_selected_word = word_layout.map(|wl| wl.token.to_string());

                let should_show = if let Some(ref current_word) = current_selected_word {
                    if let Some(ref last_word) = last_word_selected {
                        if current_word == last_word && mouse.left_released {
                            None
                        } else {
                            word_layout
                        }
                    } else {
                        word_layout
                    }
                } else {
                    word_layout
                };

                let word_layout = should_show;

                if current_selected_word.is_some() {
                    last_word_selected = current_selected_word;
                }

                let mut word_store_mut = word_store.clone();
                let selected_word = word_layout.map(|wl| wl.token.to_string());
                if let Some(word_text) = selected_word.clone() {
                    if mouse.left_pressed {
                        let mouse_pos = mouse_position();
                        let drawer = DrawerLayout::new(screen_width, screen_height);
                        if let Some(button) = drawer.is_point_in_button(mouse_pos.0, mouse_pos.1) {
                            word_store_mut.set_status(&word_text, button.status);
                            info!("Set status for '{}': {:?}", word_text, button.status);
                        }
                    }
                }

                ReaderView::render(
                    &AppStateContext::new(
                        state.clone(),
                        bible.clone(),
                        word_store_mut,
                        theme.clone(),
                        Some(font_context.clone()),
                    ),
                    screen_width,
                    screen_height,
                    book_id,
                    chapter,
                    Some(&layout),
                    word_layout,
                );

                let back_button_rect = Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 80.0,
                    h: 40.0,
                };

                let mouse_pos = mouse_position();
                if mouse.left_released && back_button_rect.contains(vec2(mouse_pos.0, mouse_pos.1))
                {
                    state = AppState::ChapterSelection {
                        book_id: book_id.clone(),
                    };
                }

                draw_rectangle(
                    back_button_rect.x,
                    back_button_rect.y,
                    back_button_rect.w,
                    back_button_rect.h,
                    theme.seen_color,
                );
                draw_text(
                    "Back",
                    back_button_rect.x + 20.0,
                    back_button_rect.y + 28.0,
                    20.0,
                    theme.text_color,
                );
            }
        }

        next_frame().await;
    }
}
