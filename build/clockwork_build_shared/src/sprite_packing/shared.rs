use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheet {
    pub texture_width: u32,
    pub texture_height: u32,
    pub sprites: Vec<SpriteResource>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteResource {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<FrameResource>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameResource {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
