use super::{
    imgui_component_utils::{EntitySerializationCommand, EntitySerializationCommandType},
    SerializationId, *,
};
pub type SerializedHashMap = std::collections::HashMap<SerializationId, SerializedEntity>;

// /// Loads all the entities as a SerializedHashMap. If this is a Prefab Scene, it will
// /// load `Prefab.Members`, or the `entities_data.yaml` otherwise.
// pub fn load_all_entities(scene: &Scene) -> Result<SerializedHashMap, Error> {
//     let (scene_entity_path, is_prefab) = path(scene);
//     if is_prefab {
//         let prefab: Prefab = load_serialized_file(&scene_entity_path)?;
//         Ok(prefab.members)
//     } else {
//         load_serialized_file(&scene_entity_path)
//     }
// }

// /// A wrapper about `load_entity_by_id`, pulling out `serialized_data`'s `.id`.
// /// Use this as a convenience function!
// pub fn load_committed_entity(
//     serialized_data: &SerializationMarker,
//     scene: &Scene,
// ) -> Result<Option<SerializedEntity>, Error> {
//     load_entity_by_id(&serialized_data.id, scene)
// }

// /// Gets an Entity from the SerializedHashMap by ID.
// pub fn load_entity_by_id(id: &SerializationId, scene: &Scene) -> Result<Option<SerializedEntity>, Error> {
//     let mut entities: SerializedHashMap = load_all_entities(scene)?;
//     Ok(entities.remove(id))
// }

// /// The Path to the current Scene. Just a Helper
// fn path(scene: &Scene) -> (String, bool) {
//     (scene.entity_path(), scene.is_prefab())
// }
