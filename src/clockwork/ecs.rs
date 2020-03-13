use super::{
    components::{ComponentDatabase, Entity},
    entities::EntityAllocator,
    hardware_interfaces::HardwareInterface,
    resources::{PrefabMap, ResourcesDatabase},
    scene_graph::SceneGraph,
    systems::*,
    ActionMap, GameWorldDrawCommands, SingletonDatabase,
};
use anyhow::Error;

pub struct Ecs {
    pub component_database: ComponentDatabase,
    pub scene_graph: SceneGraph,
    pub singleton_database: SingletonDatabase,
    pub entities: Vec<Entity>,
    pub entity_allocator: EntityAllocator,
}

impl Ecs {
    pub fn new(prefabs: &PrefabMap) -> Result<Self, Error> {
        let mut ecs = Ecs {
            component_database: ComponentDatabase::default(),
            scene_graph: SceneGraph::new(),
            singleton_database: SingletonDatabase::new()?,
            entities: Vec::new(),
            entity_allocator: EntityAllocator::new(),
        };

        let mut saved_entities = serialization_util::entities::load_all_entities()?;
        let serialized_scene_graph = serialization_util::serialized_scene_graph::load_scene_graph()?;

        // Load in the SceneGraph...
        serialized_scene_graph.walk_tree_generically(|s_node| {
            match saved_entities.remove(s_node.inner()) {
                Some(serialized_entity) => {
                    let new_id = ecs.create_entity();

                    // Load the SE. Note: if it's in the SceneGraph, then it'll almost certainly have a transform
                    let _ = ecs.load_serialized_entity(&new_id, serialized_entity, prefabs);

                    // Load in our Prefab Parent NodeID.
                    let parent_id = {
                        s_node.parent().and_then(|s_uuid| {
                            serialized_scene_graph.get(s_uuid).and_then(|parent_uuid| {
                                scene_graph_system::find_transform_from_serialized_node(
                                    &mut ecs.component_database,
                                    &parent_uuid,
                                )
                                .and_then(|parent_transform| parent_transform.inner().scene_graph_node_id())
                            })
                        })
                    };

                    // Did we find a PrefabParentNodeID?
                    if let Some(parent_id) = parent_id {
                        if let Some(transform) = &mut ecs.component_database.transforms.get_mut(&new_id) {
                            if let Some(node_id) = transform.inner_mut().scene_graph_node_id() {
                                parent_id.append(node_id, &mut ecs.scene_graph);
                            }
                        }
                    }
                }

                None => {
                    error!(
                        "Our SceneGraph for {} had a child {} but we couldn't find it in the EntityList?",
                        scene_system::current_scene_name(),
                        s_node.inner()
                    );
                }
            }
        });

        #[cfg(debug_assertions)]
        {
            if serialized_scene_graph.iter().all(|s_node| {
                let id = s_node.inner();

                ecs.component_database
                    .serialization_markers
                    .iter()
                    .any(|smc| smc.inner().id == *id)
            }) == false
            {
                error!(
                    "Not all members of the SerializedGraph for {} have been placed into the scene.",
                    scene_system::current_scene_name()
                )
            }
        }

        // For the Non-SceneGraph entities too:
        for (_, s_entity) in saved_entities.into_iter() {
            let new_entity = ecs.create_entity();

            let _ = ecs.load_serialized_entity(&new_entity, s_entity, prefabs);
        }

        // Post Deserialization Work!
        let scene_graph = &mut ecs.scene_graph;
        ecs.component_database.post_deserialization(
            PostDeserializationRequired,
            |component_list, serialization_markers| {
                component_list.post_deserialization(serialization_markers, scene_graph);
            },
        );

        Ok(ecs)
    }

    /// The difference between GameStart and New is that everyting in initialized by now.
    pub fn game_start(
        &mut self,
        resources: &ResourcesDatabase,
        hardware_interfaces: &HardwareInterface,
    ) -> Result<(), Error> {
        self.singleton_database
            .initialize_with_runtime_resources(resources, hardware_interfaces);

        // tilemap_system::initialize_tilemaps(&mut self.component_database.tilemaps, &resources.tilesets);

        player_system::initialize_players(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
        );

        Ok(())
    }

    pub fn update(&mut self, actions: &ActionMap) -> Result<(), Error> {
        player_system::player_update(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
            &mut self.component_database.velocities,
            &mut self.scene_graph,
            actions,
        );

        Ok(())
    }

    pub fn update_resources(&mut self, resources: &ResourcesDatabase, delta_time: f32) {
        sprite_system::update_sprites(&mut self.component_database.sprites, resources, delta_time);
        cross_cutting_system::cross_cutting_system(self, resources);
    }

    pub fn render<'a, 'b>(
        &'a mut self,
        draw_commands: &'b mut DrawCommand<'a>,
        resources: &'a ResourcesDatabase,
    ) {
        draw_commands.game_world = Some(GameWorldDrawCommands {
            text_sources: &self.component_database.text_sources,
            sprites: &self.component_database.sprites,
            rects: &self.component_database.draw_rectangles,
            // tilemaps: &self.component_database.tilemaps,
            transforms: &self.component_database.transforms,
            camera_entity: self
                .singleton_database
                .associated_entities
                .get(&self.singleton_database.camera.marker()),
            camera: self.singleton_database.camera.inner(),
            rendering_utility: &mut self.singleton_database.rendering_utility,
            resources,
        })
    }
}

impl Ecs {
    /// This is the standard method to create a new Entity in the Ecs. Try to
    /// always use this one. The returned Entity is the ID, or index, of the new
    /// entity.
    pub fn create_entity(&mut self) -> Entity {
        Ecs::create_entity_raw(
            &mut self.component_database,
            &mut self.entity_allocator,
            &mut self.entities,
        )
    }

    /// For use during creation and startup, before we have an Ecs
    /// to do anything with
    fn remove_entity_raw(
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        component_database: &mut ComponentDatabase,
        scene_graph: &SceneGraph,
        entity_to_delete: &Entity,
    ) -> bool {
        let is_dealloc = entity_allocator.deallocate(entity_to_delete);
        if is_dealloc {
            component_database.deregister_entity(&entity_to_delete, scene_graph);
            entities
                .iter()
                .position(|i| i == entity_to_delete)
                .map(|i| entities.remove(i));
        }
        is_dealloc
    }

    /// Use this only in weird situations. Otherwise, prefer to pass
    /// the entire Ecs around now that we have a leaner top level
    /// struct.
    fn create_entity_raw(
        component_database: &mut ComponentDatabase,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
    ) -> Entity {
        let entity = entity_allocator.allocate();
        component_database.register_entity(entity);
        entities.push(entity);
        entity
    }

    pub fn remove_entity(&mut self, entity_to_delete: &Entity) -> bool {
        // If it's in the SceneGraph, we're going to delete its children too.
        // children are, and this is a fact, the worst. Jk, I wnat to be a father one day.
        if let Some(transform) = self.component_database.transforms.get(entity_to_delete) {
            if let Some(node_id) = transform.inner().scene_graph_node_id() {
                let scene_graph_raw_handle: *mut SceneGraph = &mut self.scene_graph;
                for descendant in node_id
                    .descendants(&self.scene_graph)
                    .collect::<Vec<_>>()
                    .iter()
                    .rev()
                {
                    if descendant == &node_id {
                        continue;
                    }

                    let id = self.scene_graph.get(*descendant).unwrap().inner();
                    Ecs::remove_entity_raw(
                        &mut self.entity_allocator,
                        &mut self.entities,
                        &mut self.component_database,
                        &self.scene_graph,
                        id,
                    );
                    descendant.remove(unsafe { &mut *scene_graph_raw_handle });
                }
                node_id.remove(&mut self.scene_graph);
            }
        }

        Ecs::remove_entity_raw(
            &mut self.entity_allocator,
            &mut self.entities,
            &mut self.component_database,
            &mut self.scene_graph,
            entity_to_delete,
        )
    }

    pub fn clone_entity(&mut self, original: &Entity) -> Entity {
        let new_entity = self.create_entity();
        self.component_database
            .clone_components(original, &new_entity, &mut self.scene_graph);

        new_entity
    }
}

impl Ecs {
    /// We can load anything using this function. The key thing to note here,
    /// however, is that this adds a `SerializationMarker` component to whatever is being
    /// loaded. Ie -- if you load something with this function, it is now serialized.
    #[must_use]
    pub fn load_serialized_entity(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
        prefabs: &PrefabMap,
    ) -> Option<PostDeserializationRequired> {
        // Make a serialization data thingee on it...
        self.component_database.serialization_markers.set_component(
            &entity,
            SerializationMarker::with_id(serialized_entity.id),
            &mut self.scene_graph,
        );

        // If it's got a prefab, load the prefab. Otherwise,
        // load it like a normal serialized entity:
        if let Some(serialized_prefab_marker) = &serialized_entity.prefab_marker {
            // Base Prefab
            let success = self.load_serialized_prefab(
                entity,
                prefabs.get(&serialized_prefab_marker.inner.prefab_id()).cloned(),
                serialized_prefab_marker.inner.child_map(),
            );

            if success.is_none() {
                if self.remove_entity(entity) == false {
                    error!(
                        "We couldn't remove the entity either! Watch out -- weird stuff might happen there."
                    );
                }
                return None;
            }
        }

        // If it had a prefab, now we'll be loading in the overrides...
        Some(self.component_database.load_serialized_entity_into_database(
            entity,
            serialized_entity,
            &mut self.scene_graph,
            &mut self.singleton_database.associated_entities,
        ))
    }

    /// This function loads a prefab directly. Note though, it will not make the resulting
    /// Scene Entity be serialized. To do that, please use `load_serialized_entity`, which
    /// will load the prefab and keep it serialized.
    ///
    /// This function should be used by editor code to instantiate a prefab!
    #[must_use]
    pub fn load_serialized_prefab(
        &mut self,
        root_entity_id: &Entity,
        mut prefab_maybe: Option<Prefab>,
        // we're making this a guarded hashmap, I'm tired of being confused by this...
        child_map: &Option<std::collections::HashMap<SerializationId, SerializationId>>,
    ) -> Option<PostDeserializationRequired> {
        if let Some(mut prefab) = prefab_maybe.take() {
            // Load in the Top Level
            let prefab_main_entity_id = prefab.root_id();
            let prefab_id = prefab.prefab_id();

            if let Some(se) = prefab.members.remove(&prefab_main_entity_id) {
                let _ = self.component_database.load_serialized_entity_into_database(
                    root_entity_id,
                    se,
                    &mut self.scene_graph,
                    &mut self.singleton_database.associated_entities,
                );

                self.component_database.prefab_markers.set_component(
                    root_entity_id,
                    PrefabMarker::new(prefab_id, prefab_main_entity_id, None),
                    &mut self.scene_graph,
                );
            } else {
                error!("We couldn't find our main entity in our Prefab. That's weird!");
                return None;
            }

            let members = &mut prefab.members;
            let serialized_graph = &prefab.serialized_graph;

            // Load in the Children
            serialized_graph.walk_tree_generically(|s_node| {
                // A bit crude
                if *s_node.inner() == prefab_main_entity_id {
                    return;
                }

                match members.remove(s_node.inner()) {
                    Some(serialized_entity) => {
                        let serialized_id = serialized_entity.id;
                        let new_id = self.create_entity();

                        // Load in the Prefab
                        let _ = self.component_database.load_serialized_entity_into_database(
                            &new_id,
                            serialized_entity,
                            &mut self.scene_graph,
                            &mut self.singleton_database.associated_entities,
                        );

                        // Load in our Prefab Parent NodeID.
                        let parent_id = {
                            // None when Root Entity
                            s_node.parent().and_then(|serialized_node_id| {
                                // Probably never None -- indicates graph issue
                                serialized_graph.get(serialized_node_id).and_then(|parent_snode| {
                                    scene_graph_system::find_transform_from_prefab_node(
                                        &mut self.component_database.transforms,
                                        &self.component_database.prefab_markers,
                                        &parent_snode,
                                    )
                                    .and_then(|parent_transform| {
                                        parent_transform.inner().scene_graph_node_id()
                                    })
                                })
                            })
                        };

                        // Did we find a PrefabParentNodeID?
                        if let Some(parent_id) = parent_id {
                            if let Some(transform) = self.component_database.transforms.get_mut(&new_id) {
                                if let Some(node_id) = transform.inner_mut().scene_graph_node_id() {
                                    parent_id.append(node_id, &mut self.scene_graph);
                                }
                            }
                        }

                        // Set our Prefab
                        self.component_database.prefab_markers.set_component(
                            &new_id,
                            PrefabMarker::new(prefab_id, serialized_id, None),
                            &mut self.scene_graph,
                        );

                        // If the Parent is Serialized, then we'll add a Serialization ourselves...
                        if self
                            .component_database
                            .serialization_markers
                            .get(root_entity_id)
                            .is_some()
                        {
                            // self.component_database.serialization_markers.set_component(&new_id, SerializationMarker::with_id(prefab_info.))
                        }
                    }

                    None => {
                        error!(
                            "Our Root ID for Prefab {} had a lost child {}",
                            Name::get_name_quick(&self.component_database.names, root_entity_id),
                            s_node.inner()
                        );
                    }
                }
            });

            #[cfg(debug_assertions)]
            {
                // Check here that all the Members within the Prefab were placed into the Scene!
                if prefab.members.is_empty() == false {
                    error!(
                        "Not all members of Prefab {prefab_name} were assigned into the Scene! Prefab {prefab_name} does not make a true Scene Graph!",
                        prefab_name = prefab_id,
                    );
                    error!("The following were outside the Graph: {:#?}", prefab.members);
                }
            }

            Some(PostDeserializationRequired)
        } else {
            error!(
                "Prefab does not exist, but we tried to load it into entity {}. We cannot complete this operation.",
                Name::get_name_quick(&self.component_database.names, root_entity_id)
            );

            None
        }
    }
}
