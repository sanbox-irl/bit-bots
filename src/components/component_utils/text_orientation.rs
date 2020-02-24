use glyph_brush::{HorizontalAlign, VerticalAlign};
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, typename::TypeName, EnumIter)]
pub enum TextVerticalAlign {
    /// Characters/bounds start underneath the render position and progress downwards.
    Top,
    /// Characters/bounds center at the render position and progress outward equally.
    Center,
    /// Characters/bounds start above the render position and progress upward.
    Bottom,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, typename::TypeName, EnumIter)]
pub enum TextHorizontalAlign {
    /// Leftmost character is immediately to the right of the render position.
    /// Bounds start from the render position and advance rightwards.
    Left,
    /// Leftmost & rightmost characters are equidistant to the render position.
    /// Bounds start from the render position and advance equally left & right.
    Center,
    /// Rightmost character is immetiately to the left of the render position.
    /// Bounds start from the render position and advance leftwards.
    Right,
}

impl Default for TextVerticalAlign {
    fn default() -> TextVerticalAlign {
        TextVerticalAlign::Bottom
    }
}

impl Default for TextHorizontalAlign {
    fn default() -> TextHorizontalAlign {
        TextHorizontalAlign::Left
    }
}

impl From<TextHorizontalAlign> for HorizontalAlign {
    fn from(o: TextHorizontalAlign) -> HorizontalAlign {
        match o {
            TextHorizontalAlign::Center => HorizontalAlign::Center,
            TextHorizontalAlign::Left => HorizontalAlign::Left,
            TextHorizontalAlign::Right => HorizontalAlign::Right,
        }
    }
}

impl From<TextVerticalAlign> for VerticalAlign {
    fn from(o: TextVerticalAlign) -> VerticalAlign {
        match o {
            TextVerticalAlign::Center => VerticalAlign::Center,
            TextVerticalAlign::Top => VerticalAlign::Top,
            TextVerticalAlign::Bottom => VerticalAlign::Bottom,
        }
    }
}
