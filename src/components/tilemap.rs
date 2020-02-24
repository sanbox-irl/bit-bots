use super::{
    component_serialization::TilemapSerialized, component_utils::EditingMode, imgui_system,
    serialization_util, tile_resources::*, Color, ComponentBounds, DrawOrder, InspectorParameters,
    PositionalRect, StandardQuad, TextureDescription, Tile, Vec2, Vec2Int,
};

// @techdebt This is pretty messy. Can we clean this up a bit?
#[derive(Debug, Clone, Default, PartialEq, typename::TypeName)]
pub struct Tilemap {
    pub tiles: Vec<Option<Tile>>,
    pub size: Vec2Int,
    pub tint: Color,
    pub draw_order: DrawOrder,
    pub rebuild_collision_boxes: bool,
    pub collision_bounding_boxes: Vec<PositionalRect>,
    pub tileset: Option<TileSet>,
    pub new_tileset: Option<Option<TileSetName>>,
    pub edit_mode: EditingMode<usize, Vec2Int>,
}

impl Tilemap {
    pub fn create_tile_quads(&self, root_position: Vec2, quad_buffer: &mut Vec<StandardQuad>) {
        if let Some(tileset) = &self.tileset {
            if tileset.visual_data.sprite_data.is_some() {
                for (i, this_tile) in self.tiles.iter().enumerate() {
                    if let Some(this_tile) = this_tile {
                        if let Some((texture_info, tile_native_size)) =
                            tileset.visual_data.tileset_texture_description(this_tile.index)
                        {
                            let pos = self.get_tile_position(i, root_position, tile_native_size);

                            let standard_quad = StandardQuad {
                                texture_info: TextureDescription::Standard(texture_info),
                                pos,
                                color: self.tint,
                                draw_order: self.draw_order,
                                image_size: tile_native_size,
                            };

                            quad_buffer.push(standard_quad);
                        }
                    }
                }
            }
        }
    }

    pub fn reset_tileset(&mut self, new_tileset: &TileSet) {
        // Take in the Tileset...
        self.tileset = Some(new_tileset.clone());
        self.rebuild_collision_boxes = true;
    }

    pub fn get_tile_offset(&self, position: i32) -> Vec2Int {
        Vec2Int::new(position % self.size.y, position / self.size.y)
    }

    pub fn get_tile_index(size: Vec2Int, offset: Vec2Int) -> usize {
        (offset.x + size.y * offset.y) as usize
    }

    pub fn get_tile_position(
        &self,
        tile_index: usize,
        tilemap_position: Vec2,
        tile_native_size: Vec2,
    ) -> Vec2 {
        let offset_position: Vec2 = self.get_tile_offset(tile_index as i32).into();
        tilemap_position + tile_native_size.cwise_product(offset_position)
    }

    pub fn set_tile_at_offset(
        tiles: &mut Vec<Option<Tile>>,
        size: Vec2Int,
        shifted_position: Vec2Int,
        new_tile_index_maybe: Option<usize>,
    ) {
        let position_in_vec: usize = Self::get_tile_index(size, shifted_position);

        match &mut tiles[position_in_vec] {
            Some(tile_struct) => match new_tile_index_maybe {
                Some(index) => {
                    tile_struct.index = index;
                }

                None => {
                    tiles[position_in_vec] = None;
                }
            },

            None => match new_tile_index_maybe {
                Some(index) => {
                    tiles[position_in_vec] = Some(Tile { index });
                }
                _ => {}
            },
        }
    }
}

// use imgui::im_str;
// impl ComponentBounds for Tilemap {
//     fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
//         // Set New Tile Set...
//         self.new_tileset =
//             imgui_system::typed_enum_selection_option(ip.ui, &self.tileset.as_ref().map(|i| i.name), ip.uid);

//         self.size.vec2int_inspector_like_ints(
//             ip.ui,
//             &imgui::im_str!("Rows##{}", ip.uid),
//             &imgui::im_str!("Columns##{}", ip.uid),
//         );

//         self.tint.inspect(ip.ui, "Tint", ip.uid);
//         self.draw_order.inspect(ip.ui, ip.uid);

//         if let Some(tileset) = &self.tileset {
//             if ip.is_open == false {
//                 self.edit_mode = EditingMode::NoEdit;
//             }

//             match &mut self.edit_mode {
//                 EditingMode::NoEdit => {
//                     if ip.ui.button(im_str!("Edit Tilemap"), [0.0, 0.0]) {
//                         self.edit_mode = EditingMode::Editing(Some(0), vec![]);
//                     }
//                 }

//                 EditingMode::Editing(tile_to_edit, _) => {
//                     if ip.ui.button(im_str!("Stop Editing"), [0.0, 0.0]) {
//                         self.edit_mode = EditingMode::NoEdit;
//                     } else {
//                         imgui_system::typed_option_selection(
//                             "Edit Tiles on Tilemap",
//                             "Insert Tile",
//                             "Remove Tile",
//                             ip.ui,
//                             ip.uid,
//                             tile_to_edit,
//                             |tile_to_edit| {
//                                 if let Some(sprite_data) = &tileset.visual_data.sprite_data {
//                                     let max_size = {
//                                         let tileset_dims = sprite_data.size;
//                                         let tile_size =
//                                             tileset_dims.cwise_div(tileset.visual_data.rows_and_columns);

//                                         (tileset_dims.x / tile_size.x).max(tileset_dims.y / tile_size.y) - 1
//                                     };

//                                     let mut temp: i32 = *tile_to_edit as i32;
//                                     if ip.ui.input_int(&im_str!("Tile##{}", ip.uid), &mut temp).build() {
//                                         if temp < 0 {
//                                             temp = 0;
//                                         }
//                                         if temp > max_size {
//                                             temp = max_size;
//                                         }
//                                         *tile_to_edit = temp as usize;
//                                     }
//                                 } else {
//                                     ip.ui
//                                         .text_disabled("Tilset has No Associated Sprite. Cannot Edit!");
//                                 }

//                                 false
//                             },
//                             |_| false,
//                         );
//                     }
//                 }
//             };
//         }
//     }

//     fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
//         // serialized_entity.tilemap.as_ref().map_or(false, |s| {
//         //     if s.active == active {
//         //         let tiles: Vec<Option<Tile>> = serialization_util::tilemaps::load_tiles(&s.inner.tiles)
//         //             .map_err(|e| {
//         //                 error!(
//         //                     "Couldn't retrieve tilemaps for {}. Error: {}",
//         //                     &s.inner.tiles.relative_path, e
//         //                 )
//         //             })
//         //             .ok()
//         //             .unwrap_or_default();

//         //         let tilemap = s.inner.clone().to_tilemap(tiles);
//         //         &tilemap == self
//         //     } else {
//         //         false
//         //     }
//         // })
//         true
//     }

//     fn commit_to_scene(
//         &self,
//         se: &mut super::SerializedEntity,
//         active: bool,
//         _: &super::ComponentList<super::SerializationMarker>,
//     ) {
//         se.tilemap = TilemapSerialized::from_tilemap(self.clone(), &se.id)
//             .map_err(|e| {
//                 error!(
//                     "Error Serializing Tiles in Tilemap. Warning: our data might not be saved! {}",
//                     e
//                 )
//             })
//             .ok()
//             .map(|tmap_s| super::SerializedComponent {
//                 inner: tmap_s,
//                 active,
//             });
//     }

//     fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
//         se.tilemap = None;
//     }
// }
