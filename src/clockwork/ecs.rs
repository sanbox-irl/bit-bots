use super::{
    components::{ComponentDatabase, Entity},
    entities::EntityAllocator,
    hardware_interfaces::HardwareInterface,
    imgui_component_utils::*,
    resources::{PrefabMap, ResourcesDatabase},
    scene_graph::SceneGraph,
    systems::*,
    ActionMap, GameWorldDrawCommands, SingletonDatabase,
};
use anyhow::Result as AnyResult;

pub struct Ecs {
    pub component_database: ComponentDatabase,
    pub singleton_database: SingletonDatabase,
    pub entities: Vec<Entity>,
    pub entity_allocator: EntityAllocator,
    pub scene_data: SceneData,
    pub scene_graph: SceneGraph,
}

impl Ecs {
    pub fn new(scene_data: SceneData, prefabs: &PrefabMap) -> AnyResult<Self> {
        let mut ecs = Ecs {
            component_database: ComponentDatabase::default(),
            scene_graph: SceneGraph::new(),
            singleton_database: *scene_data.saved_singleton_data().clone(),
            entities: Vec::new(),
            entity_allocator: EntityAllocator::new(),
            scene_data,
        };

        // Load in the SceneGraph...
        ecs.scene_data
            .saved_serialized_scene_graph()
            .walk_tree_generically(|s_node| {
                match ecs.scene_data.saved_serialized_entities().get(s_node.inner()) {
                    Some(serialized_entity) => {
                        let new_id = ecs.create_entity();

                        let _ = ecs.load_serialized_entity(
                            &new_id,
                            serialized_entity.id,
                            serialized_entity.clone(),
                            prefabs,
                        );

                        // Load in our Prefab Parent NodeID.
                        let parent_id = {
                            s_node.parent().and_then(|s_uuid| {
                                ecs.scene_data
                                    .saved_serialized_scene_graph()
                                    .get(s_uuid)
                                    .and_then(|parent_uuid| {
                                        scene_graph_system::find_transform_from_serialization_id(
                                            &mut ecs.component_database.transforms,
                                            ecs.scene_data.tracked_entities(),
                                            *parent_uuid.inner(),
                                        )
                                        .and_then(
                                            |parent_transform| parent_transform.inner().scene_graph_node_id(),
                                        )
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
                            ecs.scene_data.scene(),
                            s_node.inner()
                        );
                    }
                }
            });

        // #[cfg(debug_assertions)]
        // {
        //     if serialized_scene_graph.iter().all(|s_node| {
        //         let id = s_node.inner();

        //         ecs.component_database
        //             .serialization_markers
        //             .iter()
        //             .any(|smc| smc.inner().id == *id)
        //     }) == false
        //     {
        //         error!(
        //             "Not all members of the SerializedGraph for {} have been placed into the scene.",
        //             scene_system::current_scene_name()
        //         )
        //     }
        // }

        // For the Non-SceneGraph entities too:
        for (_, s_entity) in ecs.scene_data.saved_serialized_entities() {
            let new_entity = ecs.create_entity();

            let _ = ecs.load_serialized_entity(&new_entity, s_entity.id, s_entity.clone(), prefabs);
        }

        // Post Deserialization Work!
        let scene_graph = &mut ecs.scene_graph;
        ecs.component_database.post_deserialization(
            PostDeserializationRequired,
            ecs.scene_data.tracked_entities(),
            |component_list, serialization_markers| {
                component_list.post_deserialization(serialization_markers, scene_graph);
            },
        );

        Ok(ecs)
    }

    /// The difference between GameStart and New is that everyting in initialized by now.
    pub fn game_start(&mut self, resources: &ResourcesDatabase, hardware_interfaces: &HardwareInterface) {
        self.singleton_database
            .initialize_with_runtime_resources(resources, hardware_interfaces);

        // tilemap_system::initialize_tilemaps(&mut self.component_database.tilemaps, &resources.tilesets);

        player_system::initialize_players(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
        );
    }

    pub fn update(&mut self, actions: &ActionMap) -> AnyResult<()> {
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

    /// Logs the Component Database to Console using the Debug of Dump to Log.
    pub fn log_component_database(&self) {
        for entity in self.entities.iter() {
            self.component_database
                .foreach_component_list(NonInspectableEntities::all(), |comp_list| {
                    comp_list.dump_to_log(&entity);
                });
        }
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
    /// We can load anything using this function. This will start TRACKING this serialized entity
    /// using the serialization entity id provided.
    #[must_use]
    pub fn load_serialized_entity(
        &mut self,
        entity: &Entity,
        serialization_id: SerializationId,
        serialized_entity: SerializedEntity,
        prefabs: &PrefabMap,
    ) -> Option<PostDeserializationRequired> {
        // Track it
        self.scene_data.track_entity(*entity, serialization_id);

        // Load in Original Prefab
        if let Some(serialized_prefab_marker) = &serialized_entity.prefab_marker {
            if let Some(prefab) = prefabs.get(&serialized_prefab_marker.inner.prefab_id()).cloned() {
                if self.load_serialized_prefab(entity, prefab, prefabs).is_none() {
                    if self.remove_entity(entity) == false {
                        error!("We couldn't correctly load the prefab, and we couldn't delete the malformed Entity either!");
                    }
                    return None;
                }
            }
        }

        // Load in the SE itself
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
        mut prefab: Prefab,
        prefab_map: &PrefabMap,
    ) -> Option<PostDeserializationRequired> {
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
                PrefabMarker::new(prefab_id, prefab_main_entity_id),
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
                    let serialized_id = self
                        .scene_data
                        .get_scene_id_for_prefab_child(prefab_main_entity_id, serialized_entity.id)
                        .unwrap_or_else(|| SerializationId::new());

                    let new_id = self.create_entity();

                    // Load in the Prefab
                    self.load_serialized_entity(&new_id, serialized_id, serialized_entity, prefab_map);

                    // Load in our Prefab Parent NodeID.
                    let parent_id = {
                        // Never none -- graph issue if so
                        s_node.parent().and_then(|serialized_node_id| {
                            // Probably never None -- indicates graph issue
                            serialized_graph.get(serialized_node_id).and_then(|parent_snode| {
                                scene_graph_system::find_transform_from_prefab_node(
                                    &mut self.component_database.transforms,
                                    &self.component_database.prefab_markers,
                                    &parent_snode,
                                )
                                .and_then(|parent_transform| parent_transform.inner().scene_graph_node_id())
                            })
                        })
                    };

                    // Did we find a PrefabParentNodeID? We basically always should!
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
                        PrefabMarker::new(prefab_id, serialized_entity.id),
                        &mut self.scene_graph,
                    );
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
    }

    pub fn process_serialized_command(
        &mut self,
        command: EntitySerializationCommand,
        resources: &ResourcesDatabase,
    ) -> AnyResult<()> {
        match &command.command_type {
            EntitySerializationCommandType::Revert => {
                let serialized_entity = self
                    .scene_data
                    .saved_serialized_entities()
                    .get(&command.id)
                    .cloned()
                    .ok_or_else(|| {
                        format_err!(
                            "We couldn't find {}.",
                            Name::get_name_quick(&self.component_database.names, &command.entity)
                        )
                    })?;

                // Reload the Entity
                let post = self.load_serialized_entity(
                    &command.entity,
                    serialized_entity.id,
                    serialized_entity,
                    resources.prefabs(),
                );

                if let Some(post) = post {
                    let scene_graph = &self.scene_graph;
                    self.component_database.post_deserialization(
                        post,
                        self.scene_data.tracked_entities(),
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
                    &self.component_database,
                    &self.singleton_database,
                    &self.scene_data,
                    resources,
                ) {
                    self.scene_data.serialize_entity(command.entity, se);
                }
            }

            EntitySerializationCommandType::StopSerializing => {
                let result = self.scene_data.unserialize_entity(&command.entity, &command.id);
                if result.is_none() {
                    bail!(
                        "We couldn't find {} to stop serializing them.",
                        Name::get_name_quick(&self.component_database.names, &command.entity)
                    );
                }
            }
        }

        Ok(())
    }
}

/*

else {
            error!(
                "Prefab does not exist, but we tried to load it into entity {}. We cannot complete this operation.",
                Name::get_name_quick(&self.component_database.names, root_entity_id)
            );

            None
        }

        */
