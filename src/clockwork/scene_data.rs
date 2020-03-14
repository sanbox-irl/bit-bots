use super::{
    scene_graph::SerializedSceneGraph,
    serialization_util::{self, entities::SerializedHashMap},
    ComponentDatabase, Ecs, Entity, GuardedRwLock, ResourcesDatabase, Scene, SceneIsDraft, SceneMode,
    SerializationId, SerializedEntity, SingletonDatabase,
};
use anyhow::Result as AnyResult;
pub type EntitySerializationMap = std::collections::HashMap<Entity, SerializationId>;

pub struct SceneData {
    entity_to_serialization_id: GuardedRwLock<EntitySerializationMap, SceneIsDraft>,
    serialized_scene_cache: GuardedRwLock<SerializedSceneCache, SceneIsDraft>,
    scene: Scene,
}

impl SceneData {
    pub fn new(scene: Scene) -> AnyResult<SceneData> {
        Ok(SceneData {
            entity_to_serialization_id: GuardedRwLock::new(Default::default()),
            serialized_scene_cache: GuardedRwLock::new(SerializedSceneCache::new(&scene)?),
            scene,
        })
    }

    pub fn entity_to_serialization_id(&self) -> &EntitySerializationMap {
        &self.entity_to_serialization_id.read()
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
                    self,
                    resources,
                ) {
                    self.serialize_entity(*entity, se);
                }
            }
        }

        true
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
        self.entity_to_serialization_id
            .read_mut(scene_is_draft)
            .insert(entity, serialization_id);
    }

    pub fn stop_tracking_entity(&mut self, scene_is_draft: SceneIsDraft, entity: &Entity) {
        let _old = self
            .entity_to_serialization_id
            .read_mut(scene_is_draft)
            .remove(entity);
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn process_serialized_command(
        command: EntitySerializationCommand,
        ecs: &mut Ecs,
        resources: &ResourcesDatabase,
    ) -> Result<(), Error> {
        match &command.command_type {
            EntitySerializationCommandType::Revert => {
                let serialized_entity =
                    load_entity_by_id(&command.id, ecs.scene_data.scene())?.ok_or_else(|| {
                        format_err!(
                            "We couldn't find {}. Is it in the YAML?",
                            Name::get_name_quick(&ecs.component_database.names, &command.entity)
                        )
                    })?;

                // Reload the Entity
                let post =
                    ecs.load_serialized_entity(&command.entity, serialized_entity, resources.prefabs());

                if let Some(post) = post {
                    let scene_graph = &ecs.scene_graph;
                    ecs.component_database
                        .post_deserialization(post, |component_list, sl| {
                            if let Some((inner, _)) =
                                component_list.get_for_post_deserialization(&command.entity)
                            {
                                inner.post_deserialization(command.entity, sl, scene_graph);
                            }
                        });
                }
            }

            EntitySerializationCommandType::Overwrite => {
                if let Some(se) = SerializedEntity::new(
                    &command.entity,
                    command.id,
                    &ecs.component_database,
                    &ecs.singleton_database,
                    resources,
                ) {
                    match commit_entity_to_serialized_scene(se, ecs.scene_data.scene()) {
                        Ok(_old_entity) => {}
                        Err(e) => {
                            error!("COULDN'T SERIALIZE! {}", e);
                        }
                    }
                }
            }

            EntitySerializationCommandType::StopSerializing => {
                let result = unserialize_entity(&command.id, ecs.scene_data.scene())?;
                if result == false {
                    bail!(
                        "We couldn't find {}. Is it in the YAML?",
                        Name::get_name_quick(&ecs.component_database.names, &command.entity)
                    );
                }
            }
        }

        Ok(())
    }
}

pub struct SerializedSceneCache {
    entities: SerializedHashMap,
    prefab_child_map: PrefabChildMap,
    singleton_data: SingletonDatabase,
    scene_graph: SerializedSceneGraph,

    dirty: bool,
}

impl SerializedSceneCache {
    pub fn new(scene: &Scene) -> AnyResult<Self> {
        // let mut saved_entities = serialization_util::entities::load_all_entities(scene)?;
        let serialized_scene_graph = serialization_util::serialized_scene_graph::load_scene_graph()?;

        unimplemented!()
    }

    pub fn serialize_entity(&mut self, serialized_entity: SerializedEntity) -> Option<SerializedEntity> {
        self.entities.insert(serialized_entity.id, serialized_entity)
    }

    pub fn unserialize_entity(&mut self, serialization_id: &SerializationId) -> Option<SerializedEntity> {
        self.entities.remove(serialization_id)
    }
}

pub type PrefabMemberId = SerializationId;
pub struct PrefabChildMap(std::collections::HashMap<PrefabMemberId, SerializationId>);
impl PrefabChildMap {}
