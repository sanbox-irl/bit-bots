use super::*;

pub const UPDATE_COMPONENT_DATABASE: bool = false;
// type SerializedComponentWrapper<T> = Option<(T, bool)>;

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
// #[serde(default)]
// struct Old {
//     pub name: SerializedComponentWrapper<Name>,
//     pub player: SerializedComponentWrapper<Player>,
//     pub transform: SerializedComponentWrapper<Transform>,
//     pub grid_object: SerializedComponentWrapper<GridObject>,
//     pub scene_switcher: SerializedComponentWrapper<SceneSwitcher>,
//     pub graph_node: SerializedComponentWrapper<GraphNode>,
//     pub velocity: SerializedComponentWrapper<Velocity>,
//     pub sprite: SerializedComponentWrapper<Sprite>,
//     pub sound_source: SerializedComponentWrapper<SoundSource>,
//     pub draw_rectangle: SerializedComponentWrapper<DrawRectangle>,
//     pub bounding_box: SerializedComponentWrapper<physics_components::BoundingBox>,
//     pub text_source: SerializedComponentWrapper<TextSource>,
//     pub tilemap: SerializedComponentWrapper<component_serialization::TilemapSerialized>,
//     pub follow: SerializedComponentWrapper<Follow>,
//     pub conversant_npc: SerializedComponentWrapper<ConversantNPC>,
//     pub prefab_marker: SerializedComponentWrapper<PrefabMarker>,

//     pub id: uuid::Uuid,
//     pub marker: Option<Marker>,
// }

// impl From<Old> for SerializedEntity {
//     fn from(o: Old) -> SerializedEntity {
//         SerializedEntity {
//             name: o.name.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             player: o.player.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             transform: o.transform.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             grid_object: o.grid_object.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             scene_switcher: o.scene_switcher.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             graph_node: o.graph_node.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             velocity: o.velocity.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             sprite: o.sprite.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             sound_source: o.sound_source.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             draw_rectangle: o.draw_rectangle.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             bounding_box: o.bounding_box.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             text_source: o.text_source.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             tilemap: o.tilemap.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             follow: o.follow.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             conversant_npc: o.conversant_npc.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),
//             prefab_marker: o.prefab_marker.map(|sc| SerializedComponent {
//                 inner: sc.0,
//                 active: sc.1,
//             }),

//             id: o.id,
//             marker: o.marker,
//         }
//     }
// }

pub fn update_component_database() -> Result<(), anyhow::Error> {
    // let scene_entity_path = serialization_util::entities::path();
    // let saved_entities: Vec<SerializedEntity> =
    //     serialization_util::load_serialized_file(&scene_entity_path)?;

    // let mut new_entities: HashMap<uuid::Uuid, SerializedEntity> = HashMap::new();
    // for se in saved_entities {
    //     new_entities.insert(se.id, se);
    // }
    // serialization_util::save_serialized_file(&new_entities, &scene_entity_path)?;

    Ok(())
}
