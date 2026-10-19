//! Pure, font-agnostic layout math.
//!
//! Nothing in here talks to a font: every function takes an explicit `measure`
//! closure, so the geometry can be unit tested headless and the same code is
//! used by the GUI tests.

use egui::{pos2, vec2, Rect, Vec2};

use crate::domain::Token;

/// Font size of the bible text, in points.
pub const READER_FONT_SIZE: f32 = 24.0;
/// Line box height as a multiple of the font size.
pub const LINE_HEIGHT_FACTOR: f32 = 1.5;
/// Horizontal space inserted between two words. Punctuation that hugs a word
/// (",", "." ...) does not get one.
pub const WORD_GAP: f32 = 12.0;
/// Space around a word that is drawn as its highlight / tap target.
pub const WORD_PADDING: f32 = 3.0;
/// Space kept below the last line so it never touches the translation drawer.
pub const READER_BOTTOM_PADDING: f32 = 12.0;
/// Font size of the English shown under each Russian sentence.
pub const GLOSS_FONT_SIZE: f32 = 17.0;
/// Line box height of the English, as a multiple of [`GLOSS_FONT_SIZE`].
pub const GLOSS_LINE_HEIGHT_FACTOR: f32 = 1.35;
/// Space between a Russian line and the English under it.
pub const GLOSS_TOP_GAP: f32 = 3.0;
/// Space between the English and the next Russian sentence.
pub const GLOSS_GAP: f32 = 14.0;

pub const CHAPTER_BUTTON_SIZE: f32 = 56.0;
pub const CHAPTER_BUTTON_SPACING: f32 = 12.0;
pub const BOOK_ROW_HEIGHT: f32 = 34.0;

/// User zoom range (Settings screen). `egui::Context::set_zoom_factor` applies
/// it, so every size in this module is in points and scales automatically.
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 2.0;
pub const ZOOM_STEP: f32 = 0.25;

pub const BIBLE_PATH: &str = "rst.json";
pub const TRANSLATIONS_PATH: &str = "translations.json";
pub const VERSE_TRANSLATIONS_PATH: &str = "verse_translations.json";

/// Measures the advance width of `text` rendered at `font_size`.
pub type Measure<'a> = &'a dyn Fn(&str, f32) -> f32;

/// Punctuation that needs breathing room on both sides (dashes and friends).
const SPACED_PUNCTUATION: [char; 6] = ['—', '–', '-', '*', '·', '/'];
/// Opening punctuation that hugs the word behind it, as in `«слово»`.
const OPENING_PUNCTUATION: [char; 4] = ['«', '“', '‘', '('];

fn punct_char(token: &Token) -> Option<char> {
    match token {
        Token::Punctuation(text) => text.chars().next(),
        Token::Word { .. } => None,
    }
}

/// The gap to insert in front of `next`, given the token before it.
///
/// Only dashes and similar symbols get a gap of their own: ordinary
/// punctuation stays glued to the word in front of it (`"землю."`) and
/// opening quotes stay glued to the word behind it (`"«слово»"`).
fn gap_before(prev: Option<&Token>, next: &Token) -> f32 {
    let prev = match prev {
        Some(prev) => prev,
        None => return 0.0,
    };

    if let Some(c) = punct_char(next) {
        return if is_one_of(&SPACED_PUNCTUATION, c) {
            WORD_GAP
        } else {
            0.0
        };
    }

    if let Some(c) = punct_char(prev) {
        if is_one_of(&OPENING_PUNCTUATION, c) {
            return 0.0;
        }
    }

    WORD_GAP
}

fn is_one_of(chars: &[char], c: char) -> bool {
    chars.contains(&c)
}

/// A piece of text positioned in the reader's content coordinate space.
///
/// A single [`Token`] can produce several items when the word is too wide for
/// one line: it is then split at character boundaries and the parts continue on
/// the following lines. Every part of a word is tappable, and `full_text` is
/// always the whole word, so selecting a fragment selects the right word.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutToken {
    /// Index into the source token list.
    pub token_index: usize,
    /// The (possibly partial) text to draw.
    pub text: String,
    /// The complete token this item belongs to.
    pub full_text: String,
    /// Whether the source token is a word (punctuation is not selectable).
    pub is_word: bool,
    /// Index of the line this item sits on.
    pub line: usize,
    /// Whether this item is the first part of its token.
    pub starts_word: bool,
    /// Whether this item is the last part of its token.
    pub ends_word: bool,
    /// Text box, relative to the top-left of the reader content.
    pub rect: Rect,
}

impl LaidOutToken {
    /// True when this item can be tapped/clicked to show a translation.
    pub fn is_clickable(&self) -> bool {
        self.is_word
    }
}

/// A run of Russian text and, when there is one, the English translation
/// shown underneath it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Passage {
    pub tokens: Vec<Token>,
    pub gloss: Option<String>,
}

/// One wrapped line of English. Glosses are only painted: they are never tap
/// targets, so the English cannot end up in the word lists.
#[derive(Debug, Clone, PartialEq)]
pub struct GlossLine {
    pub text: String,
    /// Text box, relative to the top-left of the reader content.
    pub rect: Rect,
}

/// Word wrapped layout of a chapter, ready to be painted and hit tested.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextLayout {
    pub items: Vec<LaidOutToken>,
    /// The English under the Russian, in reading order.
    pub glosses: Vec<GlossLine>,
    pub line_height: f32,
    /// Number of lines of Russian text.
    pub line_count: usize,
    /// Width the text was laid out for.
    pub width: f32,
    /// Height of the Russian lines and the English between them.
    pub height: f32,
}

impl TextLayout {
    /// Lay `tokens` out left to right, wrapping at `max_width`.
    ///
    /// Guarantees, relied upon by the reader and asserted in the tests:
    /// - no item sticks out past `max_width` (this is what used to push words
    ///   off the right edge of the window),
    /// - a token wider than a whole line is split instead of overflowing.
    pub fn build(
        tokens: &[Token],
        font_size: f32,
        line_height: f32,
        max_width: f32,
        measure: Measure<'_>,
    ) -> Self {
        let passage = Passage {
            tokens: tokens.to_vec(),
            gloss: None,
        };
        Self::build_passages(&[passage], font_size, line_height, max_width, measure)
    }

    /// Lay out several passages. The Russian of consecutive passages flows on
    /// as one text, except that a passage's English gloss is put on its own
    /// lines right under it and the next passage starts on a fresh line.
    pub fn build_passages(
        passages: &[Passage],
        font_size: f32,
        line_height: f32,
        max_width: f32,
        measure: Measure<'_>,
    ) -> Self {
        // The highlight / tap target extends `WORD_PADDING` beyond the text, so
        // the text itself has to stay that much narrower than the panel.
        let wrap_width = (max_width - WORD_PADDING * 2.0).max(1.0);
        let gloss_line_height = GLOSS_FONT_SIZE * GLOSS_LINE_HEIGHT_FACTOR;
        let mut layout = TextLayout {
            items: Vec::new(),
            glosses: Vec::new(),
            line_height,
            line_count: 0,
            width: max_width,
            height: 0.0,
        };

        let mut x = 0.0f32;
        // Index and top of the line being filled; `line_open` is whether
        // anything has been put on it yet.
        let mut line = 0usize;
        let mut y = 0.0f32;
        let mut line_open = false;
        let mut prev: Option<&Token> = None;
        let mut index = 0usize;

        for passage in passages {
            for token in &passage.tokens {
                let full_text = token.to_string();
                let is_word = token.is_word();
                let parts = split_to_fit(&full_text, wrap_width, font_size, measure);

                for (part_index, part) in parts.iter().enumerate() {
                    let width = measure(part, font_size);
                    let mut gap = if part_index == 0 {
                        gap_before(prev, token)
                    } else {
                        0.0
                    };

                    if x > 0.0 && x + gap + width > wrap_width {
                        line += 1;
                        y += line_height;
                        x = 0.0;
                        gap = 0.0;
                    }

                    x += gap;
                    layout.items.push(LaidOutToken {
                        token_index: index,
                        text: part.clone(),
                        full_text: full_text.clone(),
                        is_word,
                        line,
                        starts_word: part_index == 0,
                        ends_word: part_index + 1 == parts.len(),
                        rect: Rect::from_min_size(pos2(x, y), vec2(width, line_height)),
                    });
                    line_open = true;
                    x += width;
                }

                prev = Some(token);
                index += 1;
            }

            let Some(gloss) = passage.gloss.as_deref().filter(|g| !g.trim().is_empty()) else {
                continue;
            };
            if line_open {
                line += 1;
                y += line_height;
                line_open = false;
            }
            y += GLOSS_TOP_GAP;
            for text in wrap_words(gloss, GLOSS_FONT_SIZE, wrap_width, measure) {
                let width = measure(&text, GLOSS_FONT_SIZE).min(wrap_width);
                layout.glosses.push(GlossLine {
                    text,
                    rect: Rect::from_min_size(pos2(0.0, y), vec2(width, gloss_line_height)),
                });
                y += gloss_line_height;
            }
            y += GLOSS_GAP;
            x = 0.0;
            prev = None;
        }

        if line_open {
            line += 1;
            y += line_height;
        }
        layout.line_count = line;
        layout.height = y;
        layout
    }

    /// Total height of the laid out text, in points.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The slice of [`TextLayout::items`] that overlaps the band from `top`
    /// to `bottom`. Items are pushed top to bottom, so this is two binary
    /// searches.
    pub fn item_range_for_y(&self, top: f32, bottom: f32) -> std::ops::Range<usize> {
        let start = self.items.partition_point(|item| item.rect.bottom() < top);
        let end = self
            .items
            .partition_point(|item| item.rect.top() <= bottom)
            .max(start);
        start..end
    }

    /// The glosses that overlap the band from `top` to `bottom`.
    pub fn glosses_for_y(&self, top: f32, bottom: f32) -> &[GlossLine] {
        let start = self.glosses.partition_point(|g| g.rect.bottom() < top);
        let end = self
            .glosses
            .partition_point(|g| g.rect.top() <= bottom)
            .max(start);
        &self.glosses[start..end]
    }

    /// The highlight / tap target of an item.
    pub fn highlight_rect(&self, item: &LaidOutToken) -> Rect {
        Rect::from_min_max(
            pos2(
                item.rect.left() - WORD_PADDING,
                item.rect.top() - WORD_PADDING * 0.5,
            ),
            pos2(
                item.rect.right() + WORD_PADDING,
                item.rect.bottom() + WORD_PADDING * 0.5,
            ),
        )
    }

    /// The items of one line, in reading order.
    pub fn line(&self, line: usize) -> impl Iterator<Item = &LaidOutToken> {
        self.items.iter().filter(move |item| item.line == line)
    }

    /// The slice of [`TextLayout::items`] holding lines `first..=last`.
    ///
    /// Items are pushed in reading order, so every line occupies one contiguous
    /// run and the run is found by binary searching for the first item on
    /// `first`. That keeps the reader's per-frame painting proportional to the
    /// number of visible words instead of the length of the chapter.
    pub fn item_range_for_lines(&self, first: usize, last: usize) -> std::ops::Range<usize> {
        if self.items.is_empty() || first > last {
            return 0..0;
        }
        let start = self
            .items
            .partition_point(|item| item.line < first)
            .min(self.items.len());
        let end = self
            .items
            .partition_point(|item| item.line <= last)
            .max(start);
        start..end
    }

    /// The first tappable occurrence of `word`, matched exactly as drawn.
    pub fn item_for_word(&self, word: &str) -> Option<&LaidOutToken> {
        self.items
            .iter()
            .find(|item| item.is_clickable() && item.full_text == word)
    }
}

/// Split `text` into chunks that each fit within `max_width`, breaking at
/// character boundaries. Text that already fits comes back as a single chunk.
fn split_to_fit(text: &str, max_width: f32, font_size: f32, measure: Measure<'_>) -> Vec<String> {
    if measure(text, font_size) <= max_width {
        return vec![text.to_string()];
    }

    let mut parts = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if measure(&current, font_size) <= max_width {
            continue;
        }
        // The character we just added does not fit: close the current chunk
        // here and carry the character over to the next one.
        let carried = current.pop();
        match carried {
            Some(carried) if !current.is_empty() => {
                current.push(carried);
                parts.push(std::mem::take(&mut current));
            }
            // A single character is wider than the line; splitting cannot help.
            _ => return vec![text.to_string()],
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Number of buttons per row that fit in `available_width`.
pub fn grid_columns(available_width: f32, size: f32, spacing: f32) -> usize {
    if available_width <= 0.0 || size <= 0.0 {
        return 1;
    }
    (((available_width + spacing) / (size + spacing)).floor() as usize).max(1)
}

/// Rect of the `index`-th button of a grid centred in `available_width`.
pub fn grid_button_rect(
    index: usize,
    available_width: f32,
    top: f32,
    size: f32,
    spacing: f32,
) -> Rect {
    let columns = grid_columns(available_width, size, spacing);
    let row = index / columns;
    let col = index % columns;
    let row_width = columns as f32 * size + (columns as f32 - 1.0) * spacing;
    let start_x = ((available_width - row_width) / 2.0).max(0.0);
    Rect::from_min_size(
        pos2(
            start_x + col as f32 * (size + spacing),
            top + row as f32 * (size + spacing),
        ),
        vec2(size, size),
    )
}

/// Rect of every button of the chapter grid, in the same order as `chapters`.
pub fn chapter_grid(
    chapters: &[u32],
    available_width: f32,
    top: f32,
    size: f32,
    spacing: f32,
) -> Vec<(u32, Rect)> {
    chapters
        .iter()
        .enumerate()
        .map(|(index, &chapter)| {
            (
                chapter,
                grid_button_rect(index, available_width, top, size, spacing),
            )
        })
        .collect()
}

/// Total height of a `rows` tall grid, including the spacing after the last
/// row, so the scroll area reserves the right amount of room.
pub fn grid_height(rows: usize, size: f32, spacing: f32) -> f32 {
    rows as f32 * (size + spacing)
}

/// Number of text lines `text` needs in a box `width` wide.
pub fn wrapped_line_count(text: &str, font_size: f32, width: f32, measure: Measure<'_>) -> usize {
    wrap_words(text, font_size, width, measure).len().max(1)
}

/// Break `text` into lines at spaces so each fits in `width`. A single word
/// wider than `width` gets a line of its own.
pub fn wrap_words(text: &str, font_size: f32, width: f32, measure: Measure<'_>) -> Vec<String> {
    let space = measure(" ", font_size);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0.0;
    for word in text.split_whitespace() {
        let word_width = measure(word, font_size);
        if current.is_empty() {
            current.push_str(word);
            current_width = word_width;
        } else if current_width + space + word_width > width {
            lines.push(std::mem::replace(&mut current, word.to_string()));
            current_width = word_width;
        } else {
            current.push(' ');
            current.push_str(word);
            current_width += space + word_width;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Height of the translation drawer that covers the bottom of the reader.
///
/// `available_height` is the height of the reader panel; the drawer never takes
/// more than [`DRAWER_MAX_FRACTION`] of it so the text always stays usable.
pub const DRAWER_MAX_FRACTION: f32 = 0.6;
pub const DRAWER_TITLE_SIZE: f32 = 22.0;
pub const DRAWER_TEXT_SIZE: f32 = 18.0;
pub const DRAWER_PADDING: f32 = 12.0;

pub fn drawer_height(translation_lines: usize, font_size: f32, available_height: f32) -> f32 {
    let title_h = DRAWER_TITLE_SIZE * 1.4;
    let text_h = DRAWER_TEXT_SIZE * 1.4 * translation_lines.max(1) as f32;
    let buttons_h = font_size * 1.6;
    let wanted = DRAWER_PADDING * 2.0 + title_h + text_h + buttons_h;
    wanted
        .min((available_height * DRAWER_MAX_FRACTION).max(0.0))
        .max(0.0)
}

/// Extra scroll room reserved below the last line so it can be scrolled clear
/// of the translation drawer.
///
/// The room is there whether or not the drawer is open, and is enough for the
/// tallest drawer the reader panel (`panel_height` tall) can show: a word at
/// the bottom of the chapter can then always be scrolled above the drawer it
/// opens, and the end of the text does not jump as the drawer opens and closes.
pub fn reader_bottom_space(drawer_height: f32, panel_height: f32) -> f32 {
    let tallest_drawer = (panel_height * DRAWER_MAX_FRACTION).max(0.0);
    drawer_height.max(tallest_drawer) + READER_BOTTOM_PADDING
}

/// A `height` tall box pinned to the bottom edge of `area`.
pub fn bottom_band(area: Rect, height: f32) -> Rect {
    let height = height.clamp(0.0, area.height());
    Rect::from_min_size(
        pos2(area.left(), area.bottom() - height),
        Vec2::new(area.width(), height),
    )
}
