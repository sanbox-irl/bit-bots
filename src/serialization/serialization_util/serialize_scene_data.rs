use super::*;

pub fn load_singletons(scene: &Scene) -> AnyResult<SingletonDatabase> {
    load_serialized_file(&scene.singleton_path())
}

pub fn load_entities(scene: &Scene) -> AnyResult<SerializedHashMap> {
    load_file(&scene.entity_path(), scene.is_prefab(), |prefab| prefab.members)
}

pub fn load_prefab_child_map(scene: &Scene) -> AnyResult<PrefabChildMap> {
    load_file(&scene.prefab_child_map_path(), scene.is_prefab(), |prefab| {
        prefab.child_map
    })
}

pub fn load_serialized_scene_graph(scene: &Scene) -> AnyResult<scene_graph::SerializedSceneGraph> {
    load_file(&scene.scene_graph_path(), scene.is_prefab(), |prefab| {
        prefab.serialized_graph
    })
}

fn load_file<T: Default>(path: &str, is_prefab: bool, handle_prefab: impl Fn(Prefab) -> T) -> AnyResult<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    if is_prefab {
        let prefab: Prefab = load_serialized_file(path)?;
        Ok(handle_prefab(prefab))
    } else {
        load_serialized_file(path)
    }
}
