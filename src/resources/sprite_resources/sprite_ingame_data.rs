use super::cardinals::{FacingHorizontal, FacingVertical};
use super::sprite_resources::*;
use clockwork_build_shared::sprite_packing::shared::SpriteResource;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpriteInGameData {
    pub sprite_name: SpriteName,
    pub origin: Origin,
    pub facing_horizontal: FacingHorizontal,
    pub facing_vertical: FacingVertical,
    pub frame_durations: Vec<f32>,
}

impl From<SpriteData> for SpriteInGameData {
    fn from(o: SpriteData) -> Self {
        let mut frame_durations = Vec::new();
        for this_frame in o.frames {
            frame_durations.push(this_frame.duration);
        }

        Self {
            frame_durations,
            origin: o.origin,
            sprite_name: o.sprite_name,
            facing_horizontal: o.facing_horizontal,
            facing_vertical: o.facing_vertical,
        }
    }
}

impl SpriteInGameData {
    pub fn create_default(o: &SpriteResource, name: SpriteName) -> Self {
        let mut frame_durations = Vec::new();
        for _ in &o.frames {
            frame_durations.push(0.1);
        }

        Self {
            frame_durations,
            sprite_name: name,
            ..Default::default()
        }
    }
}
