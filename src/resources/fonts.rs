use glyph_brush::GlyphBrush;
use strum_macros::{Display, EnumCount, EnumIter};

#[derive(
    Debug,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    Display,
    EnumCount,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
#[strum(serialize_all = "snake_case")]
pub enum FontName {
    Muli,
    SpaceLoot,
    ExpressMono,
}

impl Default for FontName {
    fn default() -> Self {
        Self::Muli
    }
}

#[derive(Debug)]
pub struct FontData {
    pub glyph: GlyphBrush<'static, super::StandardQuad>,
    pub texture_page: Option<usize>,
}

impl FontData {
    pub fn new(glyph: GlyphBrush<'static, super::StandardQuad>) -> Self {
        FontData {
            glyph,
            texture_page: None,
        }
    }
}
