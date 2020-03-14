use super::{
    scene_graph::SerializedSceneGraph, serialization_util, ComponentDatabase, Entity, ResourcesDatabase,
    Scene, SceneMode, SerializationId, SerializedEntity, SingletonDatabase,
};
use anyhow::Result as AnyResult;
use std::collections::HashMap;

pub struct SceneData {
    entity_to_serialization_id: HashMap<Entity, SerializationId>,
    serialized_scene_cache: SerializedSceneCache,
    scene: Scene,
}

impl SceneData {
    pub fn new(scene: Scene) -> AnyResult<SceneData> {
        Ok(SceneData {
            entity_to_serialization_id: Default::default(),
            serialized_scene_cache: SerializedSceneCache::new(&scene)?,
            scene,
        })
    }

    pub fn entity_to_serialization_id(&self) -> &HashMap<Entity, SerializationId> {
        &self.entity_to_serialization_id
    }

    pub fn overwrite_serialization_with_all_entities(
        &mut self,
        entities: &[Entity],
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
        resources: &ResourcesDatabase,
    ) -> bool {
        if self.scene().mode() != SceneMode::Draft {
            return false;
        }

        for entity in entities {
            if let Some(serialization_id) = self.entity_to_serialization_id().get(entity) {
                if let Some(se) = SerializedEntity::new(
                    entity,
                    *serialization_id,
                    component_database,
                    singleton_database,
                    resources,
                ) {
                    self.serialize_entity(*entity, se);
                }
            }
        }

        true
    }

    pub fn serialize_entity(&mut self, entity: Entity, serialized_entity: SerializedEntity) -> bool {
        if self.track_entity(entity, serialized_entity.id) {
            self.serialized_scene_cache
                .entities
                .insert(serialized_entity.id, serialized_entity);
            self.serialized_scene_cache.dirty = true;

            true
        } else {
            false
        }
    }

    pub fn unserialize_entity(&mut self, serialization_id: &SerializationId) -> Result<bool, Error> {}

    pub fn track_entity(&mut self, entity: Entity, serialization_id: SerializationId) -> bool {
        if self.scene.mode() == SceneMode::Draft {
            self.entity_to_serialization_id.insert(entity, serialization_id);

            true
        } else {
            false
        }
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }
}

// structured access idea!

pub struct SerializedSceneCache {
    entities: HashMap<SerializationId, SerializedEntity>,
    prefab_child_map: HashMap<SerializationId, PrefabChildMap>,
    singleton_data: SingletonDatabase,
    scene_graph: SerializedSceneGraph,

    dirty: bool,
}

impl SerializedSceneCache {
    pub fn new(scene: &Scene) -> AnyResult<Self> {
        let mut saved_entities = serialization_util::entities::load_all_entities(scene)?;
        let serialized_scene_graph = serialization_util::serialized_scene_graph::load_scene_graph()?;

        unimplemented!()
    }
}

pub type PrefabMemberId = SerializationId;
pub struct PrefabChildMap(HashMap<PrefabMemberId, SerializationId>);
impl PrefabChildMap {}
