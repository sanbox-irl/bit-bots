use super::{scene_graph::*, *};
use anyhow::Error;

#[derive(Default)]
pub struct ComponentDatabase {
    pub names: ComponentList<Name>,
    pub prefab_markers: ComponentList<PrefabMarker>,
    pub transforms: ComponentList<Transform>,
    pub players: ComponentList<Player>,
    pub velocities: ComponentList<Velocity>,
    pub sprites: ComponentList<Sprite>,
    pub sound_sources: ComponentList<SoundSource>,
    pub bounding_boxes: ComponentList<physics_components::BoundingBox>,
    pub draw_rectangles: ComponentList<DrawRectangle>,
    // pub tilemaps: ComponentList<tilemap::Tilemap>,
    pub text_sources: ComponentList<TextSource>,
    pub follows: ComponentList<Follow>,
    pub conversant_npcs: ComponentList<ConversantNPC>,
    pub scene_switchers: ComponentList<SceneSwitcher>,
    pub serialization_markers: ComponentList<SerializationMarker>,
    size: usize,
}

impl ComponentDatabase {
    /// This creates a ComponentDatabase, loading its information from Disk,
    /// and recreates the SceneGraph! This is basically the primary loading
    /// operation that we'll be doing this iteration.
    pub fn new(
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut AssociatedEntityMap,
        prefabs: &PrefabMap,
        scene_graph: &mut SceneGraph,
    ) -> Result<ComponentDatabase, Error> {
        // Update the database...
        #[cfg(debug_assertions)]
        {
            if update_serialization::UPDATE_COMPONENT_DATABASE {
                update_serialization::update_component_database()?;
            }
        }

        let mut saved_entities = serialization_util::entities::load_all_entities()?;
        let serialized_scene_graph = serialization_util::serialized_scene_graph::load_scene_graph()?;
        let mut component_database = ComponentDatabase::default();

        // Load in the SceneGraph...
        serialized_scene_graph.walk_tree_generically(|s_node| {
            match saved_entities.remove(s_node.inner()) {
                Some(serialized_entity) => {
                    let new_id = Ecs::create_entity_raw(&mut component_database, entity_allocator, entities);

                    // Load the SE. Note: if it's in the SceneGraph, then it'll almost certainly have a transform
                    let _ = component_database.load_serialized_entity(
                        &new_id,
                        serialized_entity,
                        scene_graph,
                        entity_allocator,
                        entities,
                        marker_map,
                        prefabs,
                    );

                    // Load in our Prefab Parent NodeID.
                    let parent_id: Option<NodeId> = {
                        s_node.parent().and_then(|s_uuid| {
                            serialized_scene_graph.get(s_uuid).and_then(|parent_uuid| {
                                scene_graph_system::find_transform_from_serialized_node(
                                    &mut component_database,
                                    &parent_uuid,
                                )
                                .and_then(|parent_transform| parent_transform.inner().scene_graph_node_id())
                            })
                        })
                    };

                    // Did we find a PrefabParentNodeID?
                    if let Some(parent_id) = parent_id {
                        // Assuming *we* have a transform...
                        if let Some(transform) = component_database.transforms.get_mut(&new_id) {
                            if let Some(node_id) = transform.inner_mut().scene_graph_node_id() {
                                parent_id.append(node_id, scene_graph);
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

                component_database
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
            let new_entity = Ecs::create_entity_raw(&mut component_database, entity_allocator, entities);

            let _ = component_database.load_serialized_entity(
                &new_entity,
                s_entity,
                scene_graph,
                entity_allocator,
                entities,
                marker_map,
                prefabs,
            );
        }

        // Post Deserialization Work!
        component_database.post_deserialization(
            PostDeserializationRequired,
            |component_list, serialization_markers| {
                component_list.post_deserialization(serialization_markers, scene_graph);
            },
        );

        Ok(component_database)
    }

    pub fn register_entity(&mut self, entity: Entity) {
        let index = entity.index();
        if index == self.size {
            self.foreach_component_list_mut(NonInspectableEntities::all(), |list| list.expand_list());
            self.size = index + 1;
        }
    }

    pub fn deregister_entity(&mut self, entity: &Entity, scene_graph: &mut SceneGraph) {
        self.foreach_component_list_mut(NonInspectableEntities::all(), |list| {
            list.unset_component(entity, scene_graph);
        });
    }

    pub fn clone_components(&mut self, original: &Entity, new_entity: &Entity, scene_graph: &mut SceneGraph) {
        self.foreach_component_list_mut(NonInspectableEntities::all(), |component_list| {
            component_list.clone_entity(original, new_entity, scene_graph);
        });

        // @update_components exceptions
    }

    // @update_components
    /// This loops over every component, including the non-inspectable ones.
    pub fn foreach_component_list_mut(
        &mut self,
        non_inspectable_entities: NonInspectableEntities,
        mut f: impl FnMut(&mut dyn ComponentListBounds),
    ) {
        if non_inspectable_entities.contains(NonInspectableEntities::NAME) {
            f(&mut self.names);
        }

        self.foreach_component_list_inspectable_mut(&mut f);

        if non_inspectable_entities.contains(NonInspectableEntities::PREFAB) {
            f(&mut self.prefab_markers);
        }

        if non_inspectable_entities.contains(NonInspectableEntities::SERIALIZATION) {
            f(&mut self.serialization_markers);
        }
    }

    /// This loops over every component except for the following:
    /// - Name
    /// - PrefabMarker
    /// - SerializationMarker
    /// - GraphNode
    /// Use `foreach_component_list` to iterate over all.
    fn foreach_component_list_inspectable_mut(&mut self, f: &mut impl FnMut(&mut dyn ComponentListBounds)) {
        f(&mut self.transforms);
        f(&mut self.players);
        f(&mut self.velocities);
        f(&mut self.sprites);
        f(&mut self.sound_sources);
        f(&mut self.bounding_boxes);
        f(&mut self.draw_rectangles);
        // f(&mut self.tilemaps);
        f(&mut self.scene_switchers);
        f(&mut self.text_sources);
        f(&mut self.follows);
        f(&mut self.conversant_npcs);
    }

    // @update_components
    /// This loops over every component, including the non-inspectable ones.
    pub fn foreach_component_list(
        &self,
        non_inspectable_entities: NonInspectableEntities,
        mut f: impl FnMut(&dyn ComponentListBounds),
    ) {
        if non_inspectable_entities.contains(NonInspectableEntities::NAME) {
            f(&self.names);
        }

        self.foreach_component_list_inspectable(&mut f);

        if non_inspectable_entities.contains(NonInspectableEntities::PREFAB) {
            f(&self.prefab_markers);
        }

        if non_inspectable_entities.contains(NonInspectableEntities::SERIALIZATION) {
            f(&self.serialization_markers);
        }
    }

    /// This loops over every component except for the following:
    /// - Name
    /// - PrefabMarker
    /// - SerializationMarker
    /// - GraphNode
    /// Use `foreach_component_list` to iterate over all.
    fn foreach_component_list_inspectable(&self, f: &mut impl FnMut(&dyn ComponentListBounds)) {
        f(&self.transforms);
        f(&self.players);
        f(&self.velocities);
        f(&self.sprites);
        f(&self.sound_sources);
        f(&self.bounding_boxes);
        f(&self.draw_rectangles);
        // f(&self.tilemaps);
        f(&self.scene_switchers);
        f(&self.text_sources);
        f(&self.follows);
        f(&self.conversant_npcs);
    }

    /// We can load anything using this function. The key thing to note here,
    /// however, is that this adds a `SerializationMarker` component to whatever is being
    /// loaded. Ie -- if you load something with this function, it is now serialized.
    #[must_use]
    pub fn load_serialized_entity(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
        scene_graph: &mut SceneGraph,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut AssociatedEntityMap,
        prefabs: &PrefabMap,
    ) -> Option<PostDeserializationRequired> {
        // Make a serialization data thingee on it...
        self.serialization_markers.set_component(
            &entity,
            SerializationMarker::with_id(serialized_entity.id),
            scene_graph,
        );

        // If it's got a prefab, load the prefab. Otherwise,
        // load it like a normal serialized entity:
        if let Some(serialized_prefab_marker) = &serialized_entity.prefab_marker {
            // Base Prefab
            let success = self.load_serialized_prefab(
                entity,
                prefabs.get(&serialized_prefab_marker.inner.prefab_id()).cloned(),
                scene_graph,
                entity_allocator,
                entities,
                marker_map,
            );

            if success.is_none() {
                if Ecs::remove_entity_raw(entity_allocator, entities, self, scene_graph, entity) == false {
                    error!(
                        "We couldn't remove the entity either! Watch out -- weird stuff might happen there."
                    );
                }
                return None;
            }
        }

        // If it had a prefab, now we'll be loading in the overrides...
        Some(self.load_serialized_entity_into_database(entity, serialized_entity, scene_graph, marker_map))
    }

    /// This function loads a prefab directly. Note though, it will not make the resulting
    /// Scene Entity be serialized. To do that, please use `load_serialized_entity`, which
    /// will load the prefab and keep it serialized.
    ///
    /// This function should be used by editor code to instantiate a prefab!
    #[must_use]
    pub fn load_serialized_prefab(
        &mut self,
        main_entity_id: &Entity,
        prefab_maybe: Option<Prefab>,
        scene_graph: &mut SceneGraph,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        marker_map: &mut AssociatedEntityMap,
    ) -> Option<PostDeserializationRequired> {
        if let Some(mut prefab) = prefab_maybe {
            // Load in the Top Level
            let prefab_main_entity_id = prefab.root_id();
            let prefab_id = prefab.prefab_id();

            if let Some(se) = prefab.members.remove(&prefab_main_entity_id) {
                let _ =
                    self.load_serialized_entity_into_database(main_entity_id, se, scene_graph, marker_map);

                self.prefab_markers.set_component(
                    &main_entity_id,
                    PrefabMarker::new(prefab_id, prefab_main_entity_id, None),
                    scene_graph,
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
                        let new_id = Ecs::create_entity_raw(self, entity_allocator, entities);

                        // Load in the Prefab
                        let _ = self.load_serialized_entity_into_database(
                            &new_id,
                            serialized_entity,
                            scene_graph,
                            marker_map,
                        );

                        // Load in our Prefab Parent NodeID.
                        let parent_id: Option<NodeId> = {
                            // None when Root Entity
                            s_node.parent().and_then(|serialized_node_id| {
                                // Probably never None -- indicates graph issue
                                serialized_graph.get(serialized_node_id).and_then(|parent_snode| {
                                    scene_graph_system::find_transform_from_prefab_node(
                                        &mut self.transforms,
                                        &self.prefab_markers,
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
                            if let Some(transform) = self.transforms.get_mut(&new_id) {
                                if let Some(node_id) = transform.inner_mut().scene_graph_node_id() {
                                    parent_id.append(node_id, scene_graph);
                                }
                            }
                        }

                        // Set our Prefab
                        self.prefab_markers.set_component(
                            &new_id,
                            PrefabMarker::new(prefab_id, serialized_id, None),
                            scene_graph,
                        );
                    }

                    None => {
                        error!(
                            "Our Root ID for Prefab {} had a lost child {}",
                            Name::get_name_quick(&self.names, main_entity_id),
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
                Name::get_name_quick(&self.names, main_entity_id)
            );

            None
        }
    }

    pub fn load_yaml_delta_into_database(
        &mut self,
        entity: &Entity,
        key: serde_yaml::Value,
        delta: serde_yaml::Value,
        serialization_id: SerializationId,
        associated_entities: &mut AssociatedEntityMap,
        scene_graph: &mut SceneGraph,
    ) -> PostDeserializationRequired {
        let mut base_serialized_entity =
            serde_yaml::to_value(SerializedEntity::with_serialization_id(serialization_id)).unwrap();

        base_serialized_entity
            .as_mapping_mut()
            .unwrap()
            .insert(key, delta);

        let serialized_entity = serde_yaml::from_value(base_serialized_entity).unwrap();
        self.load_serialized_entity_into_database(entity, serialized_entity, scene_graph, associated_entities)
    }

    /// This actually does the business of unwrapping a serialized entity and putting it inside
    /// the Ecs.
    fn load_serialized_entity_into_database(
        &mut self,
        entity: &Entity,
        serialized_entity: SerializedEntity,
        scene_graph: &mut SceneGraph,
        marker_map: &mut AssociatedEntityMap,
    ) -> PostDeserializationRequired {
        let SerializedEntity {
            bounding_box,
            conversant_npc,
            draw_rectangle,
            follow,
            id: _id,
            marker: _marker, // we handle this in `load_serialized_entity`
            name,
            scene_switcher,
            prefab_marker,
            sound_source,
            sprite,
            text_source,
            // tilemap,
            transform,
            velocity,
            player,
        } = serialized_entity;

        // Helper macro
        macro_rules! transfer_serialized_components {
            ($component_name: ident, $component_database_name: ident) => {
                if let Some(serialized_component) = $component_name {
                    self.$component_database_name.set_component_with_active(
                        &entity,
                        serialized_component.active,
                        serialized_component.inner,
                        scene_graph,
                    );
                }
            };
        }

        // @update_components
        transfer_serialized_components!(prefab_marker, prefab_markers);
        transfer_serialized_components!(name, names);
        transfer_serialized_components!(transform, transforms);
        transfer_serialized_components!(scene_switcher, scene_switchers);
        transfer_serialized_components!(player, players);
        transfer_serialized_components!(sound_source, sound_sources);
        transfer_serialized_components!(bounding_box, bounding_boxes);
        transfer_serialized_components!(draw_rectangle, draw_rectangles);
        transfer_serialized_components!(text_source, text_sources);
        transfer_serialized_components!(velocity, velocities);
        transfer_serialized_components!(sprite, sprites);
        transfer_serialized_components!(follow, follows);
        transfer_serialized_components!(conversant_npc, conversant_npcs);

        // Tilemap Handling
        // if let Some(serialized_component) = tilemap {
        //     let tiles: Vec<Option<Tile>> =
        //         serialization_util::tilemaps::load_tiles(&serialized_component.inner.tiles)
        //             .map_err(|e| {
        //                 error!(
        //                     "Couldn't retrieve tilemaps for {}. Error: {}",
        //                     &serialized_component.inner.tiles.relative_path, e
        //                 )
        //             })
        //             .ok()
        //             .unwrap_or_default();

        //     let tilemap: tilemap::Tilemap = serialized_component.inner.to_tilemap(tiles);

        //     self.tilemaps
        //         .set_component_with_active(entity, tilemap, serialized_component.active);
        // }

        // Singleton Components
        if let Some(singleton_marker) = serialized_entity.marker {
            marker_map.insert(singleton_marker, *entity);
        }

        PostDeserializationRequired
    }

    pub fn post_deserialization(
        &mut self,
        _: PostDeserializationRequired,
        mut f: impl FnMut(&mut dyn ComponentListBounds, &ComponentList<SerializationMarker>),
    ) {
        let s_pointer: *const _ = &self.serialization_markers;
        let bitflag = {
            let mut all_flags = NonInspectableEntities::all();
            all_flags.remove(NonInspectableEntities::SERIALIZATION);
            all_flags
        };
        self.foreach_component_list_mut(bitflag, |component_list| {
            f(component_list, unsafe { &*s_pointer });
        });
    }
}

use bitflags::bitflags;
bitflags! {
    pub struct NonInspectableEntities: u32 {
        const NAME                  =   0b0000_0001;
        const PREFAB                =   0b0000_0010;
        const SERIALIZATION         =   0b0000_0100;
    }
}
