// use super::{
//     physics_components::*,
//     physics_system::phy_collisions,
//     sprite_resources::{SpriteData, SpriteName},
//     tile_resources::*,
//     tilemap::*,
//     ComponentList, EditingMode, Input, MouseButton, PositionalRect, SingletonDatabase, Transform, Vec2,
//     Vec2Int,
// };
// use std::collections::HashMap;

// pub fn initialize_tilemaps(
//     tilemaps: &mut ComponentList<Tilemap>,
//     resource_tilesets: &HashMap<TileSetName, TileSet>,
// ) {
//     give_dirty_tilemaps_new_tilesets(tilemaps, resource_tilesets);
// }

// pub fn update_tilemaps_and_tilesets(
//     tilemaps: &mut ComponentList<Tilemap>,
//     transforms: &mut ComponentList<Transform>,
//     resource_tilesets: &mut HashMap<TileSetName, TileSet>,
//     sprites: &HashMap<SpriteName, SpriteData>,
//     input: &Input,
//     singleton_database: &SingletonDatabase,
// ) {
//     // CFG IF
//     propogate_dirty_tilesets_to_tilemaps(tilemaps, resource_tilesets, sprites);

//     // NOT CFG IF
//     give_dirty_tilemaps_new_tilesets(tilemaps, resource_tilesets);

//     // CFG IF
//     tilemap_editing(tilemaps, transforms, singleton_database, input);

//     for this_tilemap_comp in tilemaps.iter_mut() {
//         if this_tilemap_comp.inner().rebuild_collision_boxes {
//             let root_position = {
//                 // @techdebt -- is there a better way to abstract over this?
//                 match transforms.get(&this_tilemap_comp.entity_id()) {
//                     Some(comp) => comp.inner().world_position(),
//                     None => {
//                         transforms.set_component(&this_tilemap_comp.entity_id(), Transform::default());

//                         Vec2::ZERO
//                     }
//                 }
//             };
//             let tilemap: &mut Tilemap = this_tilemap_comp.inner_mut();
//             tilemap.collision_bounding_boxes.clear();

//             if let Some(tileset) = &tilemap.tileset {
//                 if tileset.visual_data.sprite_data.is_some() {
//                     if let Some(relative_bbs) = &tileset.physics_data.bounding_boxes {
//                         for (i, this_tile) in tilemap.tiles.iter().enumerate() {
//                             if let Some(this_tile) = this_tile {
//                                 let tile_native_size =
//                                     tileset.visual_data.tileset_real_size().unwrap().into();
//                                 let pos = tilemap.get_tile_position(i, root_position, tile_native_size);

//                                 let reference: RelativeBoundingBox = relative_bbs[this_tile.index];
//                                 let positional_rect =
//                                     PositionalRect::new(pos, reference.into_rect(tile_native_size));

//                                 tilemap.collision_bounding_boxes.push(positional_rect);
//                             }
//                         }
//                     }
//                 }
//             }

//             tilemap.rebuild_collision_boxes = false;
//         }
//     }
// }

// fn propogate_dirty_tilesets_to_tilemaps(
//     tilemaps: &mut ComponentList<Tilemap>,
//     resource_tilesets: &mut HashMap<TileSetName, TileSet>,
//     sprites: &HashMap<SpriteName, SpriteData>,
// ) {
//     // Update our Resources...
//     for (_, this_tileset) in resource_tilesets.iter_mut() {
//         let mut propogate = false;

//         if this_tileset.visual_data.dirty || this_tileset.revert {
//             if let Some(new_sprite_command) = this_tileset.visual_data.next_sprite_name.take() {
//                 this_tileset.visual_data.sprite_data =
//                     new_sprite_command.map(|new_sprite_name| sprites.get(&new_sprite_name).unwrap().clone());
//             }

//             propogate = true;
//         }

//         if this_tileset.physics_data.dirty || this_tileset.revert {
//             propogate = true;
//         }

//         // Update our Existing Tilemaps...
//         if propogate {
//             for this_tilemap_component in tilemaps.iter_mut() {
//                 let this_tilemap: &mut Tilemap = this_tilemap_component.inner_mut();

//                 if let Some(tileset) = &mut this_tilemap.tileset {
//                     if tileset.name == this_tileset.name {
//                         this_tilemap.reset_tileset(this_tileset);
//                     }
//                 }
//             }

//             this_tileset.visual_data.dirty = false;
//             this_tileset.revert = false;
//             this_tileset.physics_data.dirty = false;
//         }
//     }
// }

// fn give_dirty_tilemaps_new_tilesets(
//     tilemaps: &mut ComponentList<Tilemap>,
//     resource_tilesets: &HashMap<TileSetName, TileSet>,
// ) {
//     for this_tilemap_component in tilemaps.iter_mut() {
//         let this_tilemap: &mut Tilemap = this_tilemap_component.inner_mut();

//         if let Some(new_tset_name_maybe) = this_tilemap.new_tileset.take() {
//             this_tilemap.tileset = new_tset_name_maybe
//                 .map(|tset_name| resource_tilesets.get(&tset_name).unwrap())
//                 .cloned();
//         }
//     }
// }

// fn tilemap_editing(
//     tilemaps: &mut ComponentList<Tilemap>,
//     transforms: &ComponentList<Transform>,
//     singleton_database: &SingletonDatabase,
//     input: &Input,
// ) {
//     // Get Camera and Position
//     let camera = &singleton_database.camera;

//     if let Some(associated_entity) = singleton_database.associated_entities.get(&camera.marker()) {
//         if let Some(camera_transform) = transforms.get(associated_entity) {
//             let camera_pos = camera_transform.inner().world_position();

//             // Final iteration and check if we're editing, and if so, let us select a new tile!
//             for this_tilemap_comp in tilemaps.iter_mut() {
//                 let entity_id = this_tilemap_comp.entity_id();
//                 let this_tilemap: &mut Tilemap = this_tilemap_comp.inner_mut();

//                 let tile_size = (this_tilemap.size.x * this_tilemap.size.y) as usize;
//                 if tile_size != this_tilemap.tiles.len() {
//                     this_tilemap.tiles.resize_with(tile_size, Default::default);
//                 }

//                 if let EditingMode::Editing(tile, memo) = &mut this_tilemap.edit_mode {
//                     if let Some(tileset) = &this_tilemap.tileset {
//                         if input.mouse_input.is_held(MouseButton::Left) {
//                             let relative_position: Vec2 = {
//                                 let tilemap_root_position =
//                                     transforms.get(&entity_id).unwrap().inner().world_position();
//                                 let world_position = camera
//                                     .inner()
//                                     .display_to_world_position(input.mouse_input.mouse_position, camera_pos);

//                                 world_position - tilemap_root_position
//                             };

//                             let tile_size: Vec2 = tileset.visual_data.tileset_real_size().unwrap().into();
//                             let tilemap_dims = Vec2::from(this_tilemap.size).cwise_product(tile_size);

//                             if phy_collisions::point_in_vec(tilemap_dims, &relative_position) {
//                                 let shifted_position = Vec2Int {
//                                     x: relative_position.x.div_euclid(tile_size.x) as i32,
//                                     y: relative_position.y.div_euclid(tile_size.y) as i32,
//                                 };

//                                 if memo.contains(&shifted_position) == false {
//                                     Tilemap::set_tile_at_offset(
//                                         &mut this_tilemap.tiles,
//                                         this_tilemap.size,
//                                         shifted_position,
//                                         *tile,
//                                     );
//                                     this_tilemap.rebuild_collision_boxes = true;
//                                     memo.push(shifted_position);
//                                 }
//                             }
//                         }

//                         if input.mouse_input.is_released(MouseButton::Left) {
//                             memo.clear();
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
