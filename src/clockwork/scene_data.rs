use super::{
    scene_graph::SerializedSceneGraph, serialization_util, ComponentDatabase, Entity, PrefabChildMap,
    PrefabId, ResourcesDatabase, Scene, SceneIsDraft, SceneMode, SerializationId, SerializedEntity,
    SingletonDatabase, TokenizedRwLock,
};
use anyhow::Result as AnyResult;
use std::collections::HashMap;
pub type TrackedEntitiesMap = HashMap<Entity, SerializationId>;
pub type SerializedHashMap = HashMap<SerializationId, SerializedEntity>;

pub struct SceneData {
    tracked_entities: TokenizedRwLock<TrackedEntitiesMap, SceneIsDraft>,
    serialized_scene_cache: TokenizedRwLock<SerializedSceneCache, SceneIsDraft>,
    scene: Scene,
}

impl SceneData {
    pub fn new(scene: Scene) -> AnyResult<SceneData> {
        Ok(SceneData {
            tracked_entities: TokenizedRwLock::new(Default::default()),
            serialized_scene_cache: TokenizedRwLock::new(SerializedSceneCache::new(&scene)?),
            scene,
        })
    }

    pub fn tracked_entities(&self) -> &TrackedEntitiesMap {
        self.tracked_entities.read()
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn saved_serialized_entities(&self) -> &SerializedHashMap {
        &self.serialized_scene_cache.read().entities
    }

    pub fn saved_serialized_scene_graph(&self) -> &SerializedSceneGraph {
        &self.serialized_scene_cache.read().serialized_scene_graph
    }

    pub fn saved_singleton_data(&self) -> &SingletonDatabase {
        &self.serialized_scene_cache.read().singleton_data
    }

    pub fn serialize_entity(
        &mut self,
        entity: Entity,
        serialized_entity: SerializedEntity,
    ) -> Option<SerializedEntity> {
        SceneIsDraft::new(self.scene().mode()).and_then(|scene_is_draft| {
            self.track_entity_with_token(scene_is_draft, entity, serialized_entity.id);

            self.serialized_scene_cache
                .read_mut(scene_is_draft)
                .serialize_entity(serialized_entity)
        })
    }

    pub fn unserialize_entity(
        &mut self,
        entity: &Entity,
        serialization_id: &SerializationId,
    ) -> Option<SerializedEntity> {
        SceneIsDraft::new(self.scene().mode()).and_then(|scene_is_draft| {
            self.stop_tracking_entity(scene_is_draft, entity);

            self.serialized_scene_cache
                .read_mut(scene_is_draft)
                .unserialize_entity(serialization_id)
        })
    }

    pub fn track_entity(&mut self, entity: Entity, serialization_id: SerializationId) {
        SceneIsDraft::new(self.scene().mode()).and_then(|scene_is_draft| {
            self.tracked_entities
                .read_mut(scene_is_draft)
                .insert(entity, serialization_id)
        });
    }

    pub fn stop_tracking_entity(&mut self, scene_is_draft: SceneIsDraft, entity: &Entity) {
        let _old = self.tracked_entities.read_mut(scene_is_draft).remove(entity);
    }

    pub fn serialized_entity_from_entity(&self, entity: &Entity) -> Option<&SerializedEntity> {
        self.tracked_entities()
            .get(entity)
            .and_then(|serialization_id| self.saved_serialized_entities().get(serialization_id))
    }

    pub fn serialized_entity_from_entity_mut(&mut self, entity: &Entity) -> Option<&mut SerializedEntity> {
        SceneIsDraft::new(self.scene().mode()).and_then(|scene_is_draft| {
            self.tracked_entities
                .read()
                .get(entity)
                .and_then(|serialization_id| {
                    self.serialized_scene_cache
                        .read_mut(scene_is_draft)
                        .entities
                        .get_mut(serialization_id)
                })
        })
    }

    pub fn get_scene_id_for_prefab_child(
        &mut self,
        parent_id: SerializationId,
        member_id: SerializationId,
    ) -> Option<SerializationId> {
        SceneIsDraft::new(self.scene().mode()).map(|scene_is_draft| {
            let prefab_map = self
                .serialized_scene_cache
                .read_mut(scene_is_draft)
                .prefab_child_map
                .entry(parent_id)
                .or_default();
            if let Some(id) = prefab_map.get_serialization_id_for_member(member_id) {
                *id
            } else {
                let new_id = SerializationId::new();
                prefab_map.set_serializaiton_id_for_member(member_id, SerializationId::new());
                new_id
            }
        })
    }

    fn track_entity_with_token(
        &mut self,
        scene_is_draft: SceneIsDraft,
        entity: Entity,
        serialization_id: SerializationId,
    ) {
        self.tracked_entities
            .read_mut(scene_is_draft)
            .insert(entity, serialization_id);
    }
}

impl SceneData {
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
            if let Some(serialization_id) = self.tracked_entities().get(entity) {
                if let Some(se) = SerializedEntity::new(
                    entity,
                    *serialization_id,
                    component_database,
                    singleton_database,
                    self,
                    resources,
                ) {
                    self.serialize_entity(*entity, se);
                }
            }
        }

        true
    }
}

/// This is a serialized scene cache, representing our scene,
/// while we are editing on it. Changes to the serialized scene
/// cache are not saved to disk automatically.
pub struct SerializedSceneCache {
    entities: SerializedHashMap,
    prefab_child_map: HashMap<SerializationId, PrefabChildMap>,
    singleton_data: SingletonDatabase,
    serialized_scene_graph: SerializedSceneGraph,

    /// If dirty, this SerializedSceneCache no longer directly reflects
    /// what is serialized to disk.
    dirty: bool,
}

impl SerializedSceneCache {
    pub fn new(scene: &Scene) -> AnyResult<Self> {
        Ok(Self {
            entities: serialization_util::scene_data::load_entities(scene)?,
            prefab_child_map: serialization_util::scene_data::load_prefab_child_map(scene)?,
            singleton_data: serialization_util::scene_data::load_singletons(scene)?,
            serialized_scene_graph: serialization_util::scene_data::load_serialized_scene_graph(scene)?,
            dirty: false,
        })
    }

    pub fn serialize_entity(&mut self, serialized_entity: SerializedEntity) -> Option<SerializedEntity> {
        self.dirty = true;

        self.entities.insert(serialized_entity.id, serialized_entity)
    }

    /// Returns the old serialized entity, if it was previously serialized.
    pub fn unserialize_entity(&mut self, serialization_id: &SerializationId) -> Option<SerializedEntity> {
        let result = self.entities.remove(serialization_id);

        // A little cheeky here...
        if result.is_some() {
            self.dirty = true;
        }

        result
    }
}
