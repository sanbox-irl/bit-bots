use super::{Color, DrawOrder, StandardPushConstants, Vec2};
use std::cmp::Ordering;

#[derive(PartialEq, Debug, Clone)]
pub struct StandardQuad {
    pub color: Color,
    pub pos: Vec2,
    pub draw_order: DrawOrder,
    pub image_size: Vec2,
    pub texture_info: TextureDescription,
}

impl StandardQuad {
    pub fn update_with(&self, spc: &mut StandardPushConstants) {
        spc.color = self.color;
        spc.entity_position = self.pos;
        spc.image_size = self.image_size;
    }
}

impl Eq for StandardQuad {}

impl PartialOrd for StandardQuad {
    fn partial_cmp(&self, rhs: &StandardQuad) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for StandardQuad {
    fn cmp(&self, rhs: &StandardQuad) -> Ordering {
        self.draw_order
            .cmp(&rhs.draw_order)
            .then_with(|| self.pos.y.partial_cmp(&rhs.pos.y).unwrap().reverse())
    }
}

#[derive(PartialEq, Debug, Default, Clone)]
pub struct StandardTexture {
    pub norm_image_coordinate: Vec2,
    pub norm_image_size: Vec2,
    pub texture_page: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TextureDescription {
    Standard(StandardTexture),
    White,
}

impl From<StandardTexture> for TextureDescription {
    fn from(o: StandardTexture) -> TextureDescription {
        TextureDescription::Standard(o)
    }
}

pub trait StandardQuadFactory {
    fn to_standard_quad(&self, pos: Vec2) -> StandardQuad;
}
