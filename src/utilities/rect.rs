use super::Vec2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn point_width(point: Vec2, dimenions: Vec2) -> Self {
        Self {
            min: point,
            max: point + dimenions,
        }
    }

    pub fn from_zero_width(dimenions: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max: dimenions,
        }
    }

    pub fn rect_inspector(&mut self, ui: &imgui::Ui<'_>, uid: &str) -> bool {
        let mut changed = false;
        if self.min.inspector(ui, &imgui::im_str!("Min Offset##{}", uid)) {
            changed = true;
        }
        if self.max.inspector(ui, &imgui::im_str!("Max Offset##{}", uid)) {
            changed = true
        }
        changed
    }

    #[deprecated]
    pub fn clone_at_pos(&self, pos: Vec2) -> Self {
        let mut ret = self.clone();
        ret.min += pos;
        ret.max += pos;
        ret
    }

    pub fn size(&self) -> Vec2 {
        self.max.cwise_subtraction(self.min)
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.x - self.min.x
    }
}

use std::fmt;
impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[MIN: {}, {}]", self.min.x, self.min.y)?;
        write!(f, "[MAX: {}, {}]", self.max.x, self.max.y)
    }
}

impl std::ops::Add<Vec2> for Rect {
    type Output = Rect;

    fn add(self, rhs: Vec2) -> Rect {
        Rect {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl std::ops::Add<Rect> for Vec2 {
    type Output = Rect;

    fn add(self, rhs: Rect) -> Rect {
        rhs + self
    }
}

impl std::ops::Sub<Vec2> for Rect {
    type Output = Rect;

    fn sub(self, rhs: Vec2) -> Rect {
        Rect {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}
