use macroquad::math::{Rect, vec2};

use crate::ui::mouse::Input;
use crate::ui::scroll::{ScrollState, update_scroll};

const CONTENT_HEIGHT: f32 = 2000.0;
const VIEWPORT: Rect = Rect {
    x: 900.0,
    y: 150.0,
    w: 12.0,
    h: 400.0,
};

fn idle(wheel_y: f32) -> Input {
    Input {
        mouse_pos: vec2(0.0, 0.0),
        left_pressed: false,
        left_down: false,
        left_released: false,
        wheel_y,
    }
}

fn press(mouse_pos_x: f32, mouse_pos_y: f32) -> Input {
    Input {
        mouse_pos: vec2(mouse_pos_x, mouse_pos_y),
        left_pressed: true,
        left_down: true,
        left_released: false,
        wheel_y: 0.0,
    }
}

fn move_holding(mouse_pos_x: f32, mouse_pos_y: f32) -> Input {
    Input {
        mouse_pos: vec2(mouse_pos_x, mouse_pos_y),
        left_pressed: false,
        left_down: true,
        left_released: false,
        wheel_y: 0.0,
    }
}

fn release() -> Input {
    Input {
        mouse_pos: vec2(0.0, 0.0),
        left_pressed: false,
        left_down: false,
        left_released: true,
        wheel_y: 0.0,
    }
}

// Stage 5 bug 2: the scroll wheel should scroll the way one expects.
#[test]
fn test_wheel_down_moves_content_down() {
    let mut scroll = ScrollState::new();
    update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &idle(-1.0));
    assert!(scroll.offset > 0.0, "scrolling down must increase the offset");
}

#[test]
fn test_wheel_up_moves_content_up_and_clamps_at_zero() {
    let mut scroll = ScrollState {
        offset: 100.0,
        ..ScrollState::new()
    };
    update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &idle(1.0));
    assert!(scroll.offset < 100.0, "scrolling up must decrease the offset");

    update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &idle(1000.0));
    assert_eq!(scroll.offset, 0.0, "offset must not go negative");
}

#[test]
fn test_offset_clamped_to_max_scroll() {
    let mut scroll = ScrollState {
        offset: 1_000_000.0,
        ..ScrollState::new()
    };
    let max_scroll = CONTENT_HEIGHT - VIEWPORT.h; // 1600px
    update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &idle(1.0));
    assert_eq!(scroll.offset, max_scroll, "offset must not exceed max scroll");
}

// Stage 5 bug 3: dragging the thumb must scroll, and consume the input so no
// other control (e.g. a book) is clicked while dragging.
#[test]
fn test_drag_thumb_follows_mouse_and_consumes_input() {
    let mut scroll = ScrollState::new();
    let geometry = scroll.geometry(CONTENT_HEIGHT, VIEWPORT);

    // Press on the thumb.
    let consumed = update_scroll(
        &mut scroll,
        CONTENT_HEIGHT,
        VIEWPORT,
        &press(geometry.thumb.x + 1.0, geometry.thumb.y + 2.0),
    );
    assert!(consumed, "clicking the thumb must consume the click");
    assert!(scroll.dragging);

    // Move the mouse down 200px while holding; thumb follows.
    let consumed = update_scroll(
        &mut scroll,
        CONTENT_HEIGHT,
        VIEWPORT,
        &move_holding(geometry.thumb.x + 1.0, geometry.thumb.y + 200.0),
    );
    assert!(consumed, "while dragging, every frame is consumed");
    assert!(scroll.offset > 0.0);

    // Releasing ends the drag.
    let _ = update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &release());
    assert!(!scroll.dragging);
}

#[test]
fn test_no_inputs_scrolls_nothing_and_does_not_consume() {
    let mut scroll = ScrollState::new();
    let consumed = update_scroll(&mut scroll, CONTENT_HEIGHT, VIEWPORT, &idle(0.0));
    assert!(!consumed);
    assert!(!scroll.dragging);
    assert_eq!(scroll.offset, 0.0);
}

#[test]
fn test_click_on_track_jumps_and_starts_drag() {
    let mut scroll = ScrollState::new();
    let geometry = scroll.geometry(CONTENT_HEIGHT, VIEWPORT);

    // Click far down the track, away from the thumb.
    let consumed = update_scroll(
        &mut scroll,
        CONTENT_HEIGHT,
        VIEWPORT,
        &press(geometry.track.x + 1.0, geometry.track.y + 300.0),
    );
    assert!(consumed);
    assert!(scroll.offset > 0.0, "clicking the track should jump the scroll");
    assert!(scroll.dragging);
}

#[test]
fn test_press_outside_scrollbar_does_not_scroll() {
    let mut scroll = ScrollState::new();
    let consumed = update_scroll(
        &mut scroll,
        CONTENT_HEIGHT,
        VIEWPORT,
        &press(10.0, 10.0),
    );
    assert!(!consumed);
    assert_eq!(scroll.offset, 0.0);
}