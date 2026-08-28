use macroquad::math::Vec2;

/// Tracks the left mouse button across frames.
///
/// The important guarantees:
/// - `left_pressed` is only ever true for a single frame per press+release cycle.
/// - `left_released` is only ever true for a single frame on release.
/// - Holding the button never re-triggers `left_pressed`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseState {
    pub left_down: bool,
    pub left_pressed: bool,
    pub left_released: bool,
    pub wheel_y: f32,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            left_down: false,
            left_pressed: false,
            left_released: false,
            wheel_y: 0.0,
        }
    }

    /// Advance one frame. `wheel` is the raw `mouse_wheel()` tuple.
    pub fn update(&mut self, left_down: bool, wheel: (f32, f32)) {
        self.left_pressed = left_down && !self.left_down;
        self.left_released = !left_down && self.left_down;
        self.left_down = left_down;
        self.wheel_y = wheel.1;
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of all user input for the current frame, passed to the pure,
/// testable update logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    pub mouse_pos: Vec2,
    pub left_pressed: bool,
    pub left_down: bool,
    pub left_released: bool,
    pub wheel_y: f32,
}

impl Input {
    pub fn new(
        mouse_pos: Vec2,
        left_pressed: bool,
        left_down: bool,
        left_released: bool,
        wheel_y: f32,
    ) -> Self {
        Input {
            mouse_pos,
            left_pressed,
            left_down,
            left_released,
            wheel_y,
        }
    }

    pub fn from_mouse(mouse: &MouseState, mouse_pos: Vec2) -> Self {
        Input::new(
            mouse_pos,
            mouse.left_pressed,
            mouse.left_down,
            mouse.left_released,
            mouse.wheel_y,
        )
    }
}