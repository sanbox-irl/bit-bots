use super::{sprite_resources::SpriteName, ResourcesDatabase, StandardQuad, StandardTexture};
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, Default)]
pub struct RenderingUtility {
    pub quad_buffer: Vec<StandardQuad>,
    pub basic_textures: [StandardTexture; BASICTEXTURES_COUNT],
}

#[derive(Debug, EnumCount, EnumIter)]
pub enum BasicTextures {
    White,
}

impl RenderingUtility {
    pub fn initialize(&mut self, resources: &ResourcesDatabase) {
        // Enumerate
        for basic_texture in BasicTextures::iter() {
            let sprite_name = match basic_texture {
                BasicTextures::White => SpriteName::WhitePixel,
            };

            let sprite_data = resources.sprites.get(&sprite_name).unwrap();
            let tex_info = StandardTexture {
                norm_image_coordinate: sprite_data.frames[0].normalized_coord,
                norm_image_size: sprite_data.normalized_dimensions.into(),
                texture_page: sprite_data.texture_page.unwrap(),
            };

            self.basic_textures[basic_texture as usize] = tex_info;
        }
    }
}
