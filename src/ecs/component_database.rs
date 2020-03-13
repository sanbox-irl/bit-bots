use super::{scene_graph::*, *};

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
    pub fn register_entity(&mut self, entity: Entity) {
        let index = entity.index();
        if index == self.size {
            self.foreach_component_list_mut(NonInspectableEntities::all(), |list| list.expand_list());
            self.size = index + 1;
        }
    }

    pub fn deregister_entity(&mut self, entity: &Entity, scene_graph: &SceneGraph) {
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
    pub fn load_serialized_entity_into_database(
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
