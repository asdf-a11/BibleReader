use crate::domain::Token;
use crate::ui::layout::{
    chapter_buttons, grid_content_height, list_content_height, DrawerLayout, LayoutEngine,
    ReaderLayout, BOOK_HEIGHT, BUTTON_SIZE, GRID_SPACING, GRID_TOP, WORD_PADDING,
};

/// Font-free text measurer: width is proportional to char count. This lets the
/// layout engine be unit-tested without a GPU / macroquad context.
fn fake_measure(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.6
}

fn word(text: &str) -> Token {
    Token::Word {
        original: text.to_string(),
        clean_key: text.to_lowercase(),
    }
}

fn tokens(words: &[&str]) -> Vec<Token> {
    words.iter().map(|w| word(w)).collect()
}

#[test]
fn test_box_constant_computations() {
    assert_eq!(list_content_height(3), 3.0 * BOOK_HEIGHT);
    assert!(grid_content_height(6) >= grid_content_height(5));
}

// Stage 5 bug 4: chapter buttons must lay out as a grid (rows of equal height),
// not dribble diagonally downward.
#[test]
fn test_chapter_buttons_form_a_grid() {
    let chapters: Vec<u32> = (1..=13).collect();
    let buttons = chapter_buttons(&chapters, 1024.0, GRID_TOP);

    // Same row must share the same Y, and Y grows a full row-step per index.
    let step = BUTTON_SIZE + GRID_SPACING;
    for (index, &(chapter, rect)) in buttons.iter().enumerate() {
        let row = index / 6;
        let col = index % 6;
        let expected_x = (1024.0 - (6.0 * step - GRID_SPACING)) / 2.0 + col as f32 * step;
        let expected_y = GRID_TOP + row as f32 * step;

        assert_eq!(chapter, (index as u32) + 1);
        assert!(
            (rect.x - expected_x).abs() < 0.01,
            "chapter {} col {} wrong x: {} vs {}",
            chapter,
            col,
            rect.x,
            expected_x
        );
        assert!(
            (rect.y - expected_y).abs() < 0.01,
            "chapter {} row {} wrong y: {} vs {}",
            chapter,
            row,
            rect.y,
            expected_y
        );
    }

    // Row 1 is directly below row 0 (same column), proving left-to-right then
    // top-to-bottom; no diagonal drift.
    let row0_col0 = buttons[0].1;
    let row1_col0 = buttons[6].1;
    assert!((row1_col0.x - row0_col0.x).abs() < 0.01);
    assert!(row1_col0.y > row0_col0.y + 10.0);
}

// Stage 5/6: word highlighting boxes must not overlap adjacent words.
#[test]
fn test_highlight_boxes_do_not_overlap_adjacent_words() {
    let vs = tokens(&["В", "начале", "сотворил", "Бог"]);
    let layout = ReaderLayout::new(&vs, 24.0, 50.0, 100.0, 800.0, &fake_measure);

    for pair in layout.word_layouts.windows(2) {
        let prev = &pair[0].highlight_rect;
        let next = &pair[1].highlight_rect;
        assert!(
            prev.right() <= next.x + 0.01,
            "highlight boxes overlap: {} vs {}",
            prev.right(),
            next.x
        );
    }
}

#[test]
fn test_reader_layout_word_positions_advance_along_row() {
    let vs = tokens(&["В", "начале"]);
    let layout = ReaderLayout::new(&vs, 24.0, 50.0, 100.0, 800.0, &fake_measure);
    assert_eq!(layout.word_layouts.len(), 2);
    assert!(
        layout.word_layouts[1].rect.x > layout.word_layouts[0].rect.x,
        "second word should sit to the right of the first"
    );
    assert!(
        (layout.word_layouts[0].rect.y - layout.word_layouts[1].rect.y).abs() < 0.01,
        "same line words must share the same Y"
    );
    assert!(layout.content_height > 100.0);
}

#[test]
fn test_reader_layout_content_height_grows_with_lines() {
    let one_line = ReaderLayout::new(
        &tokens(&["небольшое", "предложение"]),
        24.0,
        50.0,
        100.0,
        800.0,
        &fake_measure,
    );
    let generated: Vec<String> = (0..40).map(|i| format!("слово{}", i)).collect();
    let many_words: Vec<&str> = generated.iter().map(|s| s.as_str()).collect();
    let wrapped = ReaderLayout::new(&tokens(&many_words), 24.0, 50.0, 100.0, 200.0, &fake_measure);

    assert!(wrapped.content_height > one_line.content_height);
}

#[test]
fn test_get_word_at_position_hits_words_not_punctuation() {
    let mut toks = tokens(&["В", "начале"]);
    toks.push(Token::Punctuation(",".to_string()));
    let layout = ReaderLayout::new(&toks, 24.0, 50.0, 100.0, 800.0, &fake_measure);

    // Click on the second word.
    let wl = layout.get_word_at_position(layout.word_layouts[1].rect.x + 1.0, layout.word_layouts[1].rect.y + 1.0);
    assert!(wl.is_some());
    assert_eq!(wl.unwrap().token.to_string(), "начале");
}

#[test]
fn test_punctuation_is_not_clickable() {
    let mut toks = tokens(&["В"]);
    toks.push(Token::Punctuation(",".to_string()));
    let layout = ReaderLayout::new(&toks, 24.0, 50.0, 100.0, 800.0, &fake_measure);

    let comma = layout.word_layouts.iter().find(|l| !l.token.is_word()).unwrap();
    let hit = layout.get_word_at_position(comma.rect.center().x, comma.rect.center().y);
    assert!(hit.is_none(), "clicking punctuation must not select a word");
}

#[test]
fn test_layout_engine_wraps_long_lines() {
    let toks = tokens(&["w1", "w2", "w3", "w4"]);
    let lines = LayoutEngine::compute_line_wrapping(&toks, 60.0, &fake_measure, 24.0);
    assert!(lines.len() >= 2, "wide tokens must wrap onto multiple lines");
}

#[test]
fn test_word_highlight_padding_is_small() {
    const { assert!(WORD_PADDING <= 2.0, "stage 3 requires just 1-2px padding") };
    let layout = ReaderLayout::new(&tokens(&["В"]), 24.0, 50.0, 100.0, 800.0, &fake_measure);
    let wl = &layout.word_layouts[0];

    assert!(wl.highlight_rect.x < wl.rect.x);
    assert!(wl.highlight_rect.y < wl.rect.y);
    assert!(wl.highlight_rect.w > wl.rect.w);
    assert!(wl.highlight_rect.h > wl.rect.h);
    assert!((wl.highlight_rect.w - (wl.rect.w + WORD_PADDING * 2.0)).abs() < 0.01);
}

#[test]
fn test_drawer_buttons_stay_within_screen() {
    for (w, h) in [(800.0f32, 600.0f32), (1024.0, 768.0), (400.0, 300.0)] {
        let drawer = DrawerLayout::new(w, h);
        assert!(drawer.rect.y >= 0.0);
        assert!(drawer.rect.y + drawer.rect.h <= h + 0.01);
        for button in &drawer.buttons {
            assert!(button.rect.x >= 0.0, "screen {w}x{h}");
            assert!(button.rect.x + button.rect.w <= w + 0.01, "screen {w}x{h}");
            assert!(button.rect.y + button.rect.h <= drawer.rect.y + drawer.rect.h + 0.01);
        }
    }
}

#[test]
fn test_drawer_button_hit_testing() {
    let drawer = DrawerLayout::new(1024.0, 768.0);
    for button in &drawer.buttons {
        let hit = drawer.is_point_in_button(button.rect.center().x, button.rect.center().y);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().status, button.status);
    }

    // Clicking far away from all buttons returns None.
    assert!(drawer.is_point_in_button(1.0, 1.0).is_none());
    assert_eq!(drawer.buttons.len(), 3);
}