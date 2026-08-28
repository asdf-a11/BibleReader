use macroquad::math::Rect;

use crate::ui::mouse::Input;

pub const SCROLLBAR_WIDTH: f32 = 12.0;
pub const SCROLLBAR_MARGIN: f32 = 8.0;
/// Pixels scrolled per mouse-wheel notch.
pub const WHEEL_SPEED: f32 = 40.0;

/// Vertical scroll position plus drag state. Pure logic, fully testable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollState {
    pub offset: f32,
    pub dragging: bool,
    pub grab_offset: f32,
}

impl ScrollState {
    pub fn new() -> Self {
        ScrollState {
            offset: 0.0,
            dragging: false,
            grab_offset: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.dragging = false;
    }

    pub fn clamp(&mut self, max_scroll: f32) {
        self.offset = self.offset.max(0.0).min(max_scroll.max(0.0));
    }

    /// The scroll wheel: scrolling down (negative `wheel_y`) moves content down.
    pub fn apply_wheel(&mut self, wheel_y: f32, max_scroll: f32) {
        self.offset -= wheel_y * WHEEL_SPEED;
        self.clamp(max_scroll);
    }

    /// Geometry of the scrollbar track/thumb for the current offset.
    pub fn geometry(&self, content_height: f32, viewport: Rect) -> ScrollbarGeometry {
        let visible = viewport.h.max(1.0);
        let ratio = if content_height > visible {
            visible / content_height
        } else {
            1.0
        };
        let thumb_height = (visible * ratio).clamp(12.0, visible);
        let max_scroll = (content_height - visible).max(0.0);
        let move_range = (visible - thumb_height).max(0.0);
        let thumb_y = viewport.y
            + if max_scroll > 0.0 {
                (self.offset / max_scroll) * move_range
            } else {
                0.0
            };
        ScrollbarGeometry {
            track: Rect::new(viewport.x, viewport.y, viewport.w, visible),
            thumb: Rect::new(viewport.x, thumb_y, viewport.w, thumb_height),
            thumb_height,
            max_scroll,
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarGeometry {
    pub track: Rect,
    pub thumb: Rect,
    pub thumb_height: f32,
    pub max_scroll: f32,
}

/// Process scrollbar & wheel input for one frame.
///
/// Returns `true` when the scrollbar consumed the input (the mouse is being
/// used to drag the thumb or a scrollbar click was handled); in that case the
/// caller must not process any other clicks that frame. This is what prevents
/// dragging the scrollbar from also clicking whatever is underneath the mouse.
pub fn update_scroll(
    scroll: &mut ScrollState,
    content_height: f32,
    viewport: Rect,
    input: &Input,
) -> bool {
    if !scroll.dragging && input.wheel_y != 0.0 {
        let geometry = scroll.geometry(content_height, viewport);
        scroll.apply_wheel(input.wheel_y, geometry.max_scroll);
    }

    if scroll.dragging {
        let geometry = scroll.geometry(content_height, viewport);
        if input.left_released {
            scroll.dragging = false;
        } else {
            let move_range = (geometry.track.h - geometry.thumb_height).max(1.0);
            let thumb_top = (input.mouse_pos.y - scroll.grab_offset)
                .clamp(geometry.track.y, geometry.track.y + move_range);
            let ratio = ((thumb_top - geometry.track.y) / move_range).clamp(0.0, 1.0);
            scroll.offset = ratio * geometry.max_scroll;
        }
        return true;
    }

    let geometry = scroll.geometry(content_height, viewport);

    if input.left_pressed && geometry.thumb.contains(input.mouse_pos) {
        scroll.dragging = true;
        scroll.grab_offset = input.mouse_pos.y - geometry.thumb.y;
        return true;
    }

    if input.left_pressed && geometry.track.contains(input.mouse_pos) {
        let move_range = (geometry.track.h - geometry.thumb_height).max(1.0);
        let target_thumb_top = (input.mouse_pos.y - geometry.thumb_height * 0.5)
            .clamp(geometry.track.y, geometry.track.y + move_range);
        let ratio = ((target_thumb_top - geometry.track.y) / move_range).clamp(0.0, 1.0);
        scroll.offset = ratio * geometry.max_scroll;
        scroll.dragging = true;
        scroll.grab_offset = geometry.thumb_height * 0.5;
        return true;
    }

    false
}