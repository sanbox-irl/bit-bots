use super::{
    serialization_util, tile_resources::*, tilemap::*, Color, ComponentBounds, DrawOrder, EditingMode,
    FragmentedData, InspectorParameters, Tile, Vec2Int,
};
use failure::Error;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(rename = "Tilemap", default)]
pub struct TilemapSerialized {
    pub tiles: FragmentedData<Vec<Option<usize>>>,
    pub tileset_name: Option<TileSetName>,
    pub color: Color,
    pub size: Vec2Int,
    pub draw_order: DrawOrder,
}

impl TilemapSerialized {
    pub fn from_tilemap(tilemap: Tilemap, id: &uuid::Uuid) -> Result<TilemapSerialized, Error> {
        let fragmented_data = serialization_util::tilemaps::serialize_tiles(&tilemap, &id.to_string())?;

        Ok(TilemapSerialized {
            tiles: fragmented_data,
            tileset_name: tilemap.tileset.map(|tset| tset.name),
            size: tilemap.size,
            color: tilemap.tint,
            draw_order: tilemap.draw_order,
        })
    }

    pub fn to_tilemap(self, tiles: Vec<Option<Tile>>) -> Tilemap {
        Tilemap {
            tiles,
            tileset: None,
            new_tileset: Some(self.tileset_name),
            edit_mode: EditingMode::NoEdit,
            rebuild_collision_boxes: true,
            collision_bounding_boxes: Vec::new(),
            size: self.size,
            tint: self.color,
            draw_order: self.draw_order,
        }
    }
}

// impl ComponentBounds for TilemapSerialized {
//     fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
//         ip.ui
//             .text("Tilemaps cannot currently be added as a Prefab because I'm lazy as shit!");
//     }

//     fn is_serialized(&self, se: &super::SerializedEntity, active: bool) -> bool {
//         // se.tilemap
//         //     .as_ref()
//         //     .map_or(false, |se| se.active == active && &se.inner == self)
//         true
//     }

//     fn commit_to_scene(
//         &self,
//         _: &mut super::SerializedEntity,
//         _: bool,
//         _: &super::ComponentList<super::SerializationMarker>,
//     ) {
//         unimplemented!()
//     }

//     fn uncommit_to_scene(&self, _: &mut super::SerializedEntity) {
//         unimplemented!()
//     }
// }
