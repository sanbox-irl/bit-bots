use super::{
    sprite_resources::SpriteName, TileSet, TileSetName, TileSetPhysicsData, TileSetVisualData, Vec2Int,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, typename::TypeName)]
pub struct TileSetSerialized {
    pub name: TileSetName,
    pub size: usize,
    pub visual_data_serialized: TileSetVisualDataSerialized,
    pub physics_data: TileSetPhysicsData,
}

impl From<TileSetSerialized> for TileSet {
    fn from(o: TileSetSerialized) -> TileSet {
        TileSet {
            name: o.name,
            size: o.size,
            visual_data: o.visual_data_serialized.into(),
            physics_data: o.physics_data,
            revert: false,
            editing_mode: super::EditingMode::NoEdit,
        }
    }
}

impl From<TileSet> for TileSetSerialized {
    fn from(o: TileSet) -> TileSetSerialized {
        TileSetSerialized {
            name: o.name,
            size: o.size,
            physics_data: o.physics_data,
            visual_data_serialized: o.visual_data.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, typename::TypeName)]
pub struct TileSetVisualDataSerialized {
    pub sprite_name: Option<SpriteName>,
    pub rows_and_columns: Vec2Int,
}

impl From<TileSetVisualDataSerialized> for TileSetVisualData {
    fn from(o: TileSetVisualDataSerialized) -> TileSetVisualData {
        TileSetVisualData {
            next_sprite_name: Some(o.sprite_name),
            rows_and_columns: o.rows_and_columns,
            sprite_data: None,
            dirty: true,
        }
    }
}

impl From<TileSetVisualData> for TileSetVisualDataSerialized {
    fn from(o: TileSetVisualData) -> TileSetVisualDataSerialized {
        TileSetVisualDataSerialized {
            sprite_name: o.sprite_data.map(|sd| sd.sprite_name),
            rows_and_columns: o.rows_and_columns,
        }
    }
}
