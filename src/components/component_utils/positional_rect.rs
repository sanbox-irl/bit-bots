use super::{Rect, Vec2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PositionalRect {
    position: Vec2,
    rect: Rect,
    rect_at_position: Rect,
}

impl PositionalRect {
    pub fn new(position: Vec2, bb: Rect) -> Self {
        Self {
            position,
            rect: bb,
            rect_at_position: position + bb,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn rect_at_position(&self) -> Rect {
        self.rect_at_position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.reset_rect_at_position();
    }

    pub fn set_rect_size(&mut self, rect: Rect) {
        self.rect = rect;
        self.reset_rect_at_position();
    }

    #[inline]
    fn reset_rect_at_position(&mut self) {
        self.rect_at_position = self.rect + self.position;
    }
}

use std::fmt;
impl fmt::Display for PositionalRect {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
