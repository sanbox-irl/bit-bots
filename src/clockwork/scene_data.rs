use super::{
    imgui_component_utils::{EntitySerializationCommand, EntitySerializationCommandType},
    scene_graph::SerializedSceneGraph,
    serialization_util, ComponentDatabase, Ecs, Entity, GuardedRwLock, Name, ResourcesDatabase, Scene,
    SceneIsDraft, SceneMode, SerializationId, SerializedEntity, SingletonDatabase,
};
use anyhow::Result as AnyResult;
pub type TrackedEntitiesMap = std::collections::HashMap<Entity, SerializationId>;
pub type SerializedHashMap = std::collections::HashMap<SerializationId, SerializedEntity>;
type PrefabMemberId = SerializationId;

pub struct SceneData {
    tracked_entities: GuardedRwLock<TrackedEntitiesMap, SceneIsDraft>,
    serialized_scene_cache: GuardedRwLock<SerializedSceneCache, SceneIsDraft>,
    scene: Scene,
}

impl SceneData {
    pub fn new(scene: Scene) -> AnyResult<SceneData> {
        Ok(SceneData {
            tracked_entities: GuardedRwLock::new(Default::default()),
            serialized_scene_cache: GuardedRwLock::new(SerializedSceneCache::new(&scene)?),
            scene,
        })
    }

    pub fn serialize_entity(
        &mut self,
        entity: Entity,
        serialized_entity: SerializedEntity,
    ) -> Option<SerializedEntity> {
        SceneIsDraft::new(self.scene().mode()).and_then(|scene_is_draft| {
            self.track_entity(scene_is_draft, entity, serialized_entity.id);

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

    pub fn track_entity(
        &mut self,
        scene_is_draft: SceneIsDraft,
        entity: Entity,
        serialization_id: SerializationId,
    ) {
        self.tracked_entities
            .read_mut(scene_is_draft)
            .insert(entity, serialization_id);
    }

    pub fn stop_tracking_entity(&mut self, scene_is_draft: SceneIsDraft, entity: &Entity) {
        let _old = self.tracked_entities.read_mut(scene_is_draft).remove(entity);
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn tracked_entities(&self) -> &TrackedEntitiesMap {
        self.tracked_entities.read()
    }

    pub fn saved_serialized_entities(&self) -> &SerializedHashMap {
        &self.serialized_scene_cache.read().entities
    }

    pub fn saved_serialized_scene_graph(&self) -> &SerializedSceneGraph {
        &self.serialized_scene_cache.read().serialized_scene_graph
    }

    pub fn serialized_entity_from_entity(&self, entity: &Entity) -> Option<&SerializedEntity> {
        self.tracked_entities()
            .get(entity)
            .and_then(|serialization_id| self.saved_serialized_entities().get(serialization_id))
    }

    pub fn saved_singleton_data(&self) -> &SingletonDatabase {
        &self.serialized_scene_cache.read().singleton_data
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
}

/// This is a serialized scene cache, representing our scene,
/// while we are editing on it. Changes to the serialized scene
/// cache are not saved to disk automatically.
pub struct SerializedSceneCache {
    entities: SerializedHashMap,
    prefab_child_map: PrefabChildMap,
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PrefabChildMap(std::collections::HashMap<PrefabMemberId, SerializationId>);
impl PrefabChildMap {}

impl SceneData {
    pub fn process_serialized_command(
        command: EntitySerializationCommand,
        ecs: &mut Ecs,
        resources: &ResourcesDatabase,
    ) -> AnyResult<()> {
        match &command.command_type {
            EntitySerializationCommandType::Revert => {
                let serialized_entity = ecs
                    .scene_data
                    .serialized_scene_cache
                    .read()
                    .entities
                    .get(&command.id)
                    .cloned()
                    .ok_or_else(|| {
                        format_err!(
                            "We couldn't find {}.",
                            Name::get_name_quick(&ecs.component_database.names, &command.entity)
                        )
                    })?;

                // Reload the Entity
                let post =
                    ecs.load_serialized_entity(&command.entity, serialized_entity, resources.prefabs());

                if let Some(post) = post {
                    let scene_graph = &ecs.scene_graph;
                    ecs.component_database.post_deserialization(
                        post,
                        ecs.scene_data.tracked_entities(),
                        |component_list, sl| {
                            if let Some((inner, _)) =
                                component_list.get_for_post_deserialization(&command.entity)
                            {
                                inner.post_deserialization(command.entity, sl, scene_graph);
                            }
                        },
                    );
                }
            }

            EntitySerializationCommandType::Overwrite => {
                if let Some(se) = SerializedEntity::new(
                    &command.entity,
                    command.id,
                    &ecs.component_database,
                    &ecs.singleton_database,
                    &ecs.scene_data,
                    resources,
                ) {
                    ecs.scene_data.serialize_entity(command.entity, se);
                }
            }

            EntitySerializationCommandType::StopSerializing => {
                let result = ecs.scene_data.unserialize_entity(&command.entity, &command.id);
                if result.is_none() {
                    bail!(
                        "We couldn't find {} to stop serializing them.",
                        Name::get_name_quick(&ecs.component_database.names, &command.entity)
                    );
                }
            }
        }

        Ok(())
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
