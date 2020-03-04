use super::{scene_graph::*, *};

/// Loads the `SerializedSceneGraph` entirely and returns it. It will need to be
/// processed into a standard scene graph.
pub fn load_scene_graph() -> Result<SerializedSceneGraph, Error> {
    let (scene_entity_path, is_prefab) = path();
    if is_prefab {
        let prefab: Prefab = load_serialized_file(&scene_entity_path)?;
        Ok(prefab.serialized_graph)
    } else {
        load_serialized_file(&scene_entity_path)
    }
}

/// Saves a `SerializedSceneGraph`. Right now, it does not return an old version,
/// as we do not have a way, as yet, to optionally load things. That will come!
pub fn save_serialized_scene_graph(
    serialized_graph: SerializedSceneGraph,
) -> Result<(), Error> {
    let (scene_entity_path, is_prefab) = path();
    if is_prefab {
        let mut prefab: Prefab = load_serialized_file(&scene_entity_path)?;
        prefab.serialized_graph = serialized_graph;
        super::prefabs::serialize_prefab(&prefab)
    } else {
        // let old_graph = load_serialized_file(&scene_entity_path)?;
        save_serialized_file(&serialized_graph, &scene_entity_path)
    }
}

/// The Path to the current Scene. Just a Helper
fn path() -> (String, bool) {
    let scene: &Scene = &scene_system::CURRENT_SCENE.lock().unwrap();
    (scene.scene_graph_path(), scene.is_prefab())
}
