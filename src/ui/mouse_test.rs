use macroquad::math::vec2;

use crate::ui::mouse::{Input, MouseState};

// Stage 5 "click registers every frame" bug: a click must only fire once per
// press+release cycle, and holding the button must not re-trigger it.
#[test]
fn test_single_click_registers_once_per_press_release() {
    let mut mouse = MouseState::new();
    mouse.update(false, (0.0, 0.0));

    mouse.update(true, (0.0, 0.0)); // press frame
    assert!(mouse.left_pressed);
    assert!(mouse.left_down);
    assert!(!mouse.left_released);

    // Holding the button over several frames must NOT produce new clicks.
    for _ in 0..5 {
        mouse.update(true, (0.0, 0.0));
        assert!(!mouse.left_pressed, "held frames must not re-trigger a click");
        assert!(!mouse.left_released);
        assert!(mouse.left_down);
    }

    mouse.update(false, (0.0, 0.0)); // release frame
    assert!(mouse.left_released);
    assert!(!mouse.left_down);
    assert!(!mouse.left_pressed);

    // Pressing again is a fresh, separate click.
    mouse.update(true, (0.0, 0.0));
    assert!(mouse.left_pressed);
    assert_eq!(
        (mouse.left_pressed, mouse.left_released),
        (true, false),
        "a new press is one click"
    );
}

#[test]
fn test_wheel_y_passes_through_each_frame() {
    let mut mouse = MouseState::new();
    mouse.update(false, (0.0, 5.0));
    assert_eq!(mouse.wheel_y, 5.0);

    mouse.update(false, (0.0, -3.0));
    assert_eq!(mouse.wheel_y, -3.0);
}

#[test]
fn test_input_created_from_mouse() {
    let mut mouse = MouseState::new();
    mouse.update(true, (0.0, 1.0));

    let input = Input::from_mouse(&mouse, vec2(10.0, 20.0));
    assert_eq!(input.mouse_pos, vec2(10.0, 20.0));
    assert!(input.left_pressed);
    assert!(input.left_down);
    assert!(!input.left_released);
    assert_eq!(input.wheel_y, 1.0);
}

// Stage 5 bug: moving the mouse while holding the button (e.g. dragging a
// scrollbar) must not be interpreted as a fresh click.
#[test]
fn test_dragging_mouse_does_not_produce_extra_presses() {
    let mut mouse = MouseState::new();
    mouse.update(true, (0.0, 0.0)); // press
    assert!(mouse.left_pressed);

    for _ in 0..3 {
        mouse.update(true, (0.0, 0.0)); // held, moved around
        assert!(!mouse.left_pressed);
    }
}