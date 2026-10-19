//! Tests for the pure layout math in [`crate::ui::layout`].
//!
//! `TextLayout` is where the "words run off the right edge of the window" bug
//! is fixed, so the central invariant — nothing may stick out past the width it
//! was given — is asserted here rather than eyeballed in a window.

use egui::{pos2, vec2, Rect};

use crate::domain::{TextTokenizer, Token};
use crate::ui::layout::*;

/// A deterministic stand-in for the real font: every character is one unit
/// wide, so the expected geometry can be written down by hand.
fn fake_measure(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size
}

const SIZE: f32 = 10.0;
const LINE: f32 = 20.0;
/// A character in [`fake_measure`] is `SIZE` wide, so this fits 8 of them.
const WIDTH: f32 = 8.0 * SIZE;

fn tokenize(text: &str) -> Vec<Token> {
    TextTokenizer::tokenize(text)
}

fn build(text: &str) -> TextLayout {
    TextLayout::build(&tokenize(text), SIZE, LINE, WIDTH, &fake_measure)
}

// ---------------------------------------------------------------- basic layout

#[test]
fn empty_input_produces_no_lines() {
    let layout = build("");
    assert!(layout.items.is_empty());
    assert_eq!(layout.line_count, 0);
    assert_eq!(layout.height(), 0.0);
}

#[test]
fn a_single_short_word_stays_on_one_line() {
    let layout = build("бог");
    assert_eq!(layout.line_count, 1);
    assert_eq!(layout.items.len(), 1);
    assert_eq!(layout.items[0].text, "бог");
    assert_eq!(layout.items[0].rect.min, pos2(0.0, 0.0));
    assert_eq!(layout.items[0].rect.max.y, LINE);
}

#[test]
fn everything_fits_on_one_line_when_it_can() {
    // "аб" is two characters wide and "вг" two more, plus a 12pt gap:
    // 4*10 + 12 == 52, which fits in 80.
    let layout = build("аб вг");
    assert_eq!(layout.line_count, 1);
    assert_eq!(layout.items.len(), 2);
    assert!((layout.items[1].rect.left() - 2.0 * SIZE - WORD_GAP).abs() < 0.001);
}

#[test]
fn words_wrap_onto_the_next_line_instead_of_overflowing() {
    // Nine characters no longer fit, so "девять" moves down.
    let layout = build("одно слово девять");
    let last = layout.items.last().expect("some items");
    assert!(last.line > 0, "the text should have wrapped");
    assert!(last.rect.right() <= layout.width);
}

#[test]
fn the_word_gap_is_used_between_words() {
    assert_eq!(WORD_GAP, 12.0, "the reported gap was too small to read");
    let layout = build("аб вг");
    let gap = layout.items[1].rect.left() - layout.items[0].rect.right();
    assert!((gap - WORD_GAP).abs() < 0.001, "gap was {gap}");
}

// ------------------------------------------------- the overflow bug, directly

#[test]
fn no_item_ever_sticks_out_of_the_right_edge() {
    // A long paragraph at a width that is far too small for the words.
    let text = "В начале сотворил Бог небо и землю, и сказал Бог да будет свет";
    for width in [40.0, 80.0, 137.0, 400.0] {
        let layout = TextLayout::build(&tokenize(text), SIZE, LINE, width, &fake_measure);
        for item in &layout.items {
            assert!(
                item.rect.right() <= width + 0.001,
                "{:?} ends at {} but the text may only use {width}",
                item.text,
                item.rect.right()
            );
        }
    }
}

#[test]
fn a_word_wider_than_the_line_is_split_instead_of_overflowing() {
    let layout = build("сверхдлинное");
    assert!(layout.line_count > 1, "the long word should be split");
    assert!(
        layout.items.len() > 1,
        "splitting should produce several pieces"
    );
    for item in &layout.items {
        assert!(item.rect.right() <= layout.width + 0.001);
    }
    // The pieces, read in order, are the original word again.
    let rejoined: String = layout.items.iter().map(|item| item.text.as_str()).collect();
    assert_eq!(rejoined, "сверхдлинное");
}

#[test]
fn a_single_character_wider_than_the_line_is_left_alone() {
    // Nothing can be done about this, but it must not loop forever.
    let layout = TextLayout::build(&tokenize("я"), SIZE, LINE, 1.0, &fake_measure);
    assert_eq!(layout.items.len(), 1);
    assert_eq!(layout.items[0].text, "я");
}

#[test]
fn items_are_ordered_by_line() {
    let layout = build("один два три четыре пять шесть семь восемь девять десять");
    for pair in layout.items.windows(2) {
        assert!(
            pair[0].line <= pair[1].line,
            "items must stay in reading order"
        );
    }
}

// -------------------------------------------------------------- word identity

#[test]
fn a_split_word_keeps_the_full_text_on_every_piece() {
    let layout = build("сверхдлинное");
    for item in &layout.items {
        assert_eq!(item.full_text, "сверхдлинное");
        assert!(item.is_clickable());
    }
    assert!(layout.items.first().expect("items").starts_word);
    assert!(layout.items.last().expect("items").ends_word);
}

#[test]
fn only_words_are_clickable() {
    let layout = build("бог, ");
    let punctuation: Vec<&LaidOutToken> = layout.items.iter().filter(|i| !i.is_word).collect();
    assert!(!punctuation.is_empty());
    assert!(punctuation.iter().all(|item| !item.is_clickable()));
}

#[test]
fn a_word_can_be_found_by_the_text_it_shows() {
    let layout = build("В начале сотворил Бог небо");
    let item = layout.item_for_word("Бог").expect("Бог is in the text");
    assert_eq!(item.text, "Бог");
    assert!(item.is_clickable());
    assert!(layout.item_for_word("бог").is_none(), "match is exact");
}

// ------------------------------------------------------------------- spacing

#[test]
fn punctuation_hugs_the_word_in_front_of_it() {
    let layout = build("землю.");
    assert_eq!(layout.items.len(), 2);
    assert_eq!(
        layout.items[1].rect.left(),
        layout.items[0].rect.right(),
        "a full stop must not be pushed away from the word"
    );
}

#[test]
fn dashes_get_a_gap_on_both_sides() {
    let layout = build("слов — дел");
    let dash = layout
        .items
        .iter()
        .find(|item| item.text.contains('—'))
        .expect("the dash is in the text");
    let before = layout
        .items
        .iter()
        .find(|item| item.text == "слов")
        .expect("слов is in the text");
    assert!((dash.rect.left() - before.rect.right() - WORD_GAP).abs() < 0.001);
}

#[test]
fn opening_quotes_hug_the_word_behind_them() {
    let layout = build("«небо»");
    let quote = layout
        .items
        .iter()
        .find(|item| item.text == "«")
        .expect("the quote is in the text");
    let word = layout
        .items
        .iter()
        .find(|item| item.text == "небо")
        .expect("небо is in the text");
    assert_eq!(quote.rect.right(), word.rect.left());
}

#[test]
fn the_first_item_of_a_line_has_no_leading_gap() {
    let layout = build("аб вг");
    assert_eq!(
        layout.items[1].rect.left() - layout.items[0].rect.right(),
        WORD_GAP
    );
    let wrapped = build("одно слово девять");
    for line in wrapped.items.iter().filter(|item| item.line > 0) {
        if line.starts_word {
            assert_eq!(line.rect.left(), 0.0);
        }
    }
}

// ---------------------------------------------------------------- highlighting

#[test]
fn the_highlight_is_wider_than_the_text_but_still_inside() {
    let layout = TextLayout::build(&tokenize("бог небо"), SIZE, LINE, WIDTH, &fake_measure);
    for item in &layout.items {
        let highlight = layout.highlight_rect(item);
        assert!(highlight.left() < item.rect.left());
        assert!(highlight.right() > item.rect.right());
        assert!(
            highlight.left() >= -WORD_PADDING - 0.001 && highlight.right() <= layout.width,
            "the highlight must stay inside the panel"
        );
    }
}

// ---------------------------------------------------------------- line lookup

#[test]
fn a_line_can_be_walked_in_reading_order() {
    let layout = build("аб вг");
    let line: Vec<&str> = layout.line(0).map(|item| item.text.as_str()).collect();
    assert_eq!(line, vec!["аб", "вг"]);
    assert_eq!(layout.line(5).count(), 0);
}

#[test]
fn item_ranges_pick_out_exactly_the_asked_for_lines() {
    let layout = TextLayout::build(
        &tokenize("один два три четыре пять шесть семь восемь"),
        SIZE,
        LINE,
        WIDTH,
        &fake_measure,
    );
    assert!(layout.line_count >= 3, "the test needs several lines");

    for first in 0..layout.line_count {
        for last in first..layout.line_count {
            let range = layout.item_range_for_lines(first, last);
            for item in &layout.items[range.clone()] {
                assert!(
                    item.line >= first && item.line <= last,
                    "line {} leaked into {first}..={last}",
                    item.line
                );
            }
            // Nothing from the requested lines was left out.
            for line in first..=last {
                assert!(
                    layout.items[range.clone()]
                        .iter()
                        .any(|item| item.line == line),
                    "line {line} is missing from the range"
                );
            }
        }
    }
}

#[test]
fn an_empty_layout_has_no_items() {
    let layout = TextLayout::default();
    assert_eq!(layout.item_range_for_lines(0, 5), 0..0);
    assert_eq!(layout.height(), 0.0);
}

// ---------------------------------------------------------------------- grid

#[test]
fn the_chapter_grid_fills_the_width_it_is_given() {
    assert_eq!(
        grid_columns(
            3.0 * (CHAPTER_BUTTON_SIZE + CHAPTER_BUTTON_SPACING),
            CHAPTER_BUTTON_SIZE,
            CHAPTER_BUTTON_SPACING
        ),
        3
    );
    assert_eq!(
        grid_columns(1.0, CHAPTER_BUTTON_SIZE, CHAPTER_BUTTON_SPACING),
        1
    );
    assert_eq!(
        grid_columns(0.0, CHAPTER_BUTTON_SIZE, CHAPTER_BUTTON_SPACING),
        1
    );
}

#[test]
fn chapter_buttons_stay_inside_the_panel() {
    let chapters: Vec<u32> = (1..=50).collect();
    for width in [120.0, 480.0, 1000.0] {
        let rects = chapter_grid(
            &chapters,
            width,
            0.0,
            CHAPTER_BUTTON_SIZE,
            CHAPTER_BUTTON_SPACING,
        );
        assert_eq!(rects.len(), chapters.len());
        for (chapter, rect) in &rects {
            assert!(
                rect.left() >= -0.001,
                "chapter {chapter} hangs off the left"
            );
            assert!(
                rect.right() <= width + 0.001,
                "chapter {chapter} hangs off the right at {} (width {width})",
                rect.right()
            );
        }
    }
}

#[test]
fn the_grid_is_centred() {
    let chapters: Vec<u32> = (1..=4).collect();
    let width = 4.0 * (CHAPTER_BUTTON_SIZE + CHAPTER_BUTTON_SPACING) + 40.0;
    let rects = chapter_grid(
        &chapters,
        width,
        0.0,
        CHAPTER_BUTTON_SIZE,
        CHAPTER_BUTTON_SPACING,
    );
    let left_gap = rects[0].1.left();
    let right_gap = width - rects[3].1.right();
    assert!(
        (left_gap - right_gap).abs() < 0.001,
        "the grid should be centred"
    );
}

#[test]
fn grid_height_covers_every_row() {
    let rows = 3;
    let height = grid_height(rows, CHAPTER_BUTTON_SIZE, CHAPTER_BUTTON_SPACING);
    let rects = chapter_grid(
        &(1..=rows as u32).collect::<Vec<_>>(),
        4.0 * (CHAPTER_BUTTON_SIZE + CHAPTER_BUTTON_SPACING),
        0.0,
        CHAPTER_BUTTON_SIZE,
        CHAPTER_BUTTON_SPACING,
    );
    let last = rects.last().expect("a row").1;
    assert!(height >= last.bottom(), "the last row must fit in {height}");
}

// -------------------------------------------------------------------- drawer

#[test]
fn a_taller_translation_makes_a_taller_drawer() {
    let small = drawer_height(1, READER_FONT_SIZE, 600.0);
    let large = drawer_height(4, READER_FONT_SIZE, 600.0);
    assert!(large > small);
    // An empty translation still has room for the buttons.
    assert_eq!(drawer_height(0, READER_FONT_SIZE, 600.0), small);
}

#[test]
fn the_drawer_never_covers_more_than_its_share_of_the_reader() {
    for height in [100.0, 300.0, 900.0] {
        let drawer = drawer_height(50, READER_FONT_SIZE, height);
        assert!(
            drawer <= height * DRAWER_MAX_FRACTION + 0.001,
            "{drawer} is too much of {height}"
        );
        assert!(drawer >= 0.0);
    }
}

#[test]
fn a_tall_reader_lets_the_drawer_grow_to_fit_its_text() {
    // The reader is 2000 points tall, so the drawer is only limited by the
    // text it has to show: a longer translation needs a taller drawer.
    let mut previous = 0.0;
    for lines in 1..=6 {
        let height = drawer_height(lines, READER_FONT_SIZE, 2000.0);
        assert!(
            height > previous,
            "{lines} lines should be taller than less text"
        );
        assert!(
            height < 2000.0 * DRAWER_MAX_FRACTION,
            "a tall reader should not clamp the drawer"
        );
        previous = height;
    }
}

#[test]
fn the_bottom_band_is_pinned_to_the_bottom_and_clamped() {
    let area = Rect::from_min_size(pos2(10.0, 10.0), vec2(200.0, 100.0));
    let band = bottom_band(area, 30.0);
    assert!((band.bottom() - area.bottom()).abs() < 0.001);
    assert!((band.left() - area.left()).abs() < 0.001);
    assert!((band.width() - area.width()).abs() < 0.001);

    // A drawer taller than the reader is clipped to it rather than escaping.
    let huge = bottom_band(area, 500.0);
    assert!((huge.height() - area.height()).abs() < 0.001);
}

#[test]
fn the_reserved_space_covers_the_drawer_and_a_gap() {
    let panel = 600.0;
    let tallest = panel * DRAWER_MAX_FRACTION;
    // Room for the tallest drawer even while none is open, so the last line
    // can be scrolled up before a word is tapped.
    assert_eq!(reader_bottom_space(0.0, panel), tallest + READER_BOTTOM_PADDING);
    assert_eq!(reader_bottom_space(120.0, panel), tallest + READER_BOTTOM_PADDING);
    // A drawer can never be taller than that, but if it were it still fits.
    assert_eq!(reader_bottom_space(500.0, panel), 500.0 + READER_BOTTOM_PADDING);
    assert_eq!(reader_bottom_space(0.0, 0.0), READER_BOTTOM_PADDING);
}

// -------------------------------------------------------------- wrapped labels

#[test]
fn wrapped_line_count_grows_as_the_box_narrows() {
    let measure = fake_measure;
    let text = "one two three four five six seven eight";
    let wide = wrapped_line_count(text, SIZE, 800.0, &measure);
    let narrow = wrapped_line_count(text, SIZE, 100.0, &measure);
    assert_eq!(wide, 1);
    assert!(narrow > wide, "{narrow} lines should be more than {wide}");
    assert_eq!(wrapped_line_count("   ", SIZE, 100.0, &measure), 1);
}

// ------------------------------------------------------------- english glosses

fn passage(text: &str, gloss: Option<&str>) -> Passage {
    Passage {
        tokens: tokenize(text),
        gloss: gloss.map(str::to_string),
    }
}

fn build_passages(passages: &[Passage]) -> TextLayout {
    TextLayout::build_passages(passages, SIZE, LINE, WIDTH, &fake_measure)
}

#[test]
fn a_gloss_sits_between_its_sentence_and_the_next() {
    let layout = build_passages(&[
        passage("бог.", Some("God.")),
        passage("свет.", None),
    ]);
    assert_eq!(layout.line_count, 2);
    let first_line_bottom = layout.line(0).map(|i| i.rect.bottom()).fold(0.0, f32::max);
    let gloss = &layout.glosses[0];
    assert!(gloss.rect.top() >= first_line_bottom);
    let next = layout.line(1).next().unwrap();
    assert_eq!(next.full_text, "свет");
    assert_eq!(next.rect.left(), 0.0, "no gap in front of a fresh line");
    assert!(next.rect.top() >= layout.glosses.last().unwrap().rect.bottom());
    assert!((layout.height() - (next.rect.bottom())).abs() < 0.001);
}

#[test]
fn passages_without_a_gloss_flow_on_like_one_text() {
    let joined = build("бог небо. свет и тьма.");
    let split = build_passages(&[passage("бог небо.", None), passage("свет и тьма.", None)]);
    assert_eq!(joined, split);
}

#[test]
fn a_long_gloss_wraps_inside_the_width() {
    let gloss = "In the beginning God created the heavens and the earth";
    let layout = build_passages(&[passage("бог", Some(gloss))]);
    assert!(layout.glosses.len() > 1);
    let rejoined: Vec<_> = layout.glosses.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(rejoined.join(" "), gloss);
    for line in &layout.glosses {
        assert!(line.rect.right() <= WIDTH, "{:?} sticks out", line.text);
    }
}

#[test]
fn glosses_are_never_tap_targets() {
    let layout = build_passages(&[passage("бог", Some("God"))]);
    assert!(layout.items.iter().all(|item| item.full_text != "God"));
    assert_eq!(layout.items.iter().filter(|i| i.is_clickable()).count(), 1);
}

#[test]
fn a_blank_gloss_is_ignored() {
    let with_blank = build_passages(&[passage("бог.", Some("  ")), passage("свет", None)]);
    assert!(with_blank.glosses.is_empty());
    assert_eq!(with_blank, build("бог. свет"));
}

#[test]
fn the_visible_band_finds_items_and_glosses_by_height() {
    let layout = build_passages(&[
        passage("бог.", Some("God.")),
        passage("свет.", Some("light.")),
    ]);
    let all = layout.item_range_for_y(f32::NEG_INFINITY, f32::INFINITY);
    assert_eq!(all, 0..layout.items.len());
    assert_eq!(layout.glosses_for_y(f32::NEG_INFINITY, f32::INFINITY).len(), 2);

    // Just inside the second gloss line: band edges are inclusive, and the
    // Russian line above ends exactly where the gloss starts.
    let second = layout.glosses[1].rect.shrink(1.0);
    let band = layout.item_range_for_y(second.top(), second.bottom());
    assert!(band.is_empty(), "no Russian on the English line");
    assert_eq!(layout.glosses_for_y(second.top(), second.bottom()).len(), 1);
    assert_eq!(layout.item_range_for_y(-100.0, -50.0), 0..0);
}


#[test]
fn stats_axis_maximum_is_round_and_covers_the_data() {
    use crate::ui::stats::nice_max;
    assert_eq!(nice_max(0), 4);
    assert_eq!(nice_max(3), 4);
    assert_eq!(nice_max(7), 8);
    assert_eq!(nice_max(37), 40);
    assert_eq!(nice_max(123), 200);
    for max in 0..2000 {
        let nice = nice_max(max);
        assert!(nice >= max && nice.is_multiple_of(4), "{} -> {}", max, nice);
    }
}
