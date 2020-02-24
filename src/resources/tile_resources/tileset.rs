use super::{EditingMode, TileSetName, TileSetPhysicsData, TileSetVisualData};

#[derive(Debug, Clone, Default, PartialEq, typename::TypeName)]
pub struct TileSet {
    pub name: TileSetName,
    pub size: usize,
    pub physics_data: TileSetPhysicsData,
    pub visual_data: TileSetVisualData,
    pub revert: bool,
    pub editing_mode: EditingMode<i32, i32>,
}
