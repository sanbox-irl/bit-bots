use super::physics_components::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, typename::TypeName)]
pub struct TileSetPhysicsData {
    pub bounding_boxes: Option<Vec<RelativeBoundingBox>>,
    pub dirty: bool,
}
