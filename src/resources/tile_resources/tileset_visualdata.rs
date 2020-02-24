use super::{
    imgui_system,
    sprite_resources::{SpriteData, SpriteName},
    StandardTexture, Vec2, Vec2Int,
};

#[derive(Debug, Clone, Default, PartialEq, typename::TypeName)]
pub struct TileSetVisualData {
    pub rows_and_columns: Vec2Int,
    pub next_sprite_name: Option<Option<SpriteName>>,
    pub sprite_data: Option<SpriteData>,
    pub dirty: bool,
}

impl TileSetVisualData {
    pub fn tileset_real_size(&self) -> Option<Vec2Int> {
        self.sprite_data.as_ref().map(|sprite_data| {
            let full_sprite_dimensions = sprite_data.size;
            full_sprite_dimensions.cwise_div(self.rows_and_columns)
        })
    }

    pub fn tileset_texture_description(&self, tile_index: usize) -> Option<(StandardTexture, Vec2)> {
        #[cfg(debug_assertions)]
        {
            if self.rows_and_columns.iter().any(|&dimension| dimension <= 0) {
                log_once::error_once!(
                    "Tileset has tile dimensions {}. Having a 0-or-less-length dimension is not allowed.",
                    self.rows_and_columns
                );
                return None;
            }
        }

        self.sprite_data.as_ref().map(|sprite_data| {
            let full_sprite_dimensions: Vec2 = sprite_data.size.into();
            let real_size_of_tile: Vec2 = full_sprite_dimensions.cwise_div(self.rows_and_columns.into());

            let norm_tile_size: Vec2 = {
                let percentage_of_sprite_data_dims: Vec2 =
                    real_size_of_tile.cwise_div(full_sprite_dimensions);

                sprite_data
                    .normalized_dimensions
                    .cwise_product(percentage_of_sprite_data_dims)
            };

            // @techdebt When we have columns/rows for real, this will need to be updated
            let normalized_pos_offset = {
                let mut simple_offset = norm_tile_size.cwise_product(Vec2::new(tile_index as f32, 0.0));

                #[cfg(debug_assertions)]
                {
                    simple_offset.x = simple_offset
                        .x
                        .min(sprite_data.normalized_dimensions.x - norm_tile_size.x);
                    simple_offset.y = simple_offset
                        .y
                        .min(sprite_data.normalized_dimensions.y - norm_tile_size.y);
                }

                simple_offset
            };

            let tex_info = StandardTexture {
                norm_image_coordinate: sprite_data.frames[0].normalized_coord + normalized_pos_offset,
                norm_image_size: norm_tile_size,
                texture_page: sprite_data.texture_page.unwrap(),
            };

            (tex_info, real_size_of_tile)
        })
    }

    pub fn inspector(&mut self, ui: &imgui::Ui<'_>, unique_id: &str) {
        // Make New Sprite
        let current_sprite_name = self.sprite_data.as_ref().map(|sd| sd.sprite_name);

        if let Some(new_sprite) =
            imgui_system::typed_enum_selection_option(ui, &current_sprite_name, &unique_id)
        {
            self.dirty = true;
            self.next_sprite_name = Some(new_sprite);
        }

        // Alter The Vec2Int
        if self.rows_and_columns.vec2int_inspector_like_ints(
            ui,
            &imgui::im_str!("Rows##{}", unique_id),
            &imgui::im_str!("Columns##{}", unique_id),
        ) {
            if self.rows_and_columns.y != 1 {
                self.rows_and_columns.y = 1;
                log_once::error_once!("The tileset must have a row and column of 1!")
            }
            self.dirty = true;
        }
    }
}
