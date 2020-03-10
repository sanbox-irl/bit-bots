use super::{
    scene_graph::{NodeId, SceneGraph, SerializedNodeId, SerializedSceneGraph},
    serialization_util, Component, ComponentDatabase, Ecs, Entity, Prefab, PrefabId, PrefabLoadRequired,
    PrefabMap, PrefabMarker, ResourcesDatabase, SerializationId, SerializedComponent, SerializedEntity,
    SingletonDatabase,
};
use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

pub fn commit_blank_prefab(resources: &mut ResourcesDatabase) -> Result<PrefabId> {
    let blank_prefab = Prefab::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = blank_prefab.prefab_id();
    resources.add_prefab(blank_prefab);

    Ok(id)
}

/// Create a new prefab based on an existing entity within a scene.
/// The entity will become a Prefab Inheritor, and its serialization
/// will be updated to reflect that.
pub fn commit_new_prefab(
    entity: &Entity,
    component_database: &mut ComponentDatabase,
    singleton_database: &SingletonDatabase,
    scene_graph: &mut SceneGraph,
    resources: &mut ResourcesDatabase,
) -> Result<()> {
    let root_member_id = SerializationId::new();

    // Create our Root Entity...
    if let Some(serialized_entity) = SerializedEntity::new(
        entity,
        root_member_id,
        component_database,
        singleton_database,
        resources,
    ) {
        let mut members = HashMap::new();
        let mut serialized_graph = SerializedSceneGraph::new();
        let prefab_id = PrefabId::new();
        let root_serialized_graph_id = serialized_graph.instantiate_node(root_member_id);

        fn commit_entity_to_prefab(
            entity_id: &Entity,
            serialized_entity: SerializedEntity,
            prefab_id: PrefabId,
            member_id: SerializationId,
            prefab_members: &mut HashMap<SerializationId, SerializedEntity>,
            child_map: Option<HashMap<SerializationId, SerializationId>>,
            resources: &ResourcesDatabase,
            singleton_database: &SingletonDatabase,
            scene_graph: &mut SceneGraph,
            component_database: &mut ComponentDatabase,
        ) {
            prefab_members.insert(member_id, serialized_entity);

            // Have the Main Entity gets its markers!
            component_database.prefab_markers.set_component(
                entity_id,
                PrefabMarker::new(prefab_id, member_id, child_map),
                scene_graph,
            );

            if let Some(sc) = component_database.serialization_markers.get(entity_id) {
                serialization_util::entities::serialize_entity_full(
                    entity_id,
                    sc.inner().id,
                    component_database,
                    singleton_database,
                    resources,
                );
            }
        }

        // Load children into the Prefab...
        if let Some(scene_graph_id) = component_database
            .transforms
            .get(entity)
            .and_then(|tc| tc.inner().scene_graph_node_id())
        {
            fn create_prefab_serialization(
                node_id: NodeId,
                parent_id: SerializedNodeId,
                prefab_id: PrefabId,
                serialized_graph: &mut SerializedSceneGraph,
                prefab_members: &mut HashMap<SerializationId, SerializedEntity>,
                child_map: &mut HashMap<SerializationId, SerializationId>,
                component_database: &mut ComponentDatabase,
                singleton_database: &SingletonDatabase,
                scene_graph: &SceneGraph,
                raw_scene_graph_handle: *mut SceneGraph,
                resources: &ResourcesDatabase,
            ) {
                for child in node_id.children(scene_graph) {
                    let child: super::scene_graph::NodeId = child;
                    if let Some(node) = scene_graph.get(child) {
                        let entity = node.inner();
                        let member_id = SerializationId::new();

                        // Fun Scene Graph stuff!
                        let serialized_node = serialized_graph.instantiate_node(member_id);
                        parent_id.append(serialized_node, serialized_graph);

                        if let Some(serialized_entity) = SerializedEntity::new(
                            entity,
                            member_id,
                            component_database,
                            singleton_database,
                            resources,
                        ) {
                            if let Some(our_serialized_id) = component_database
                                .serialization_markers
                                .get(entity)
                                .map(|sc| sc.inner().id)
                            {
                                child_map.insert(member_id, our_serialized_id);
                            }

                            commit_entity_to_prefab(
                                entity,
                                serialized_entity,
                                prefab_id,
                                member_id,
                                prefab_members,
                                None,
                                resources,
                                singleton_database,
                                unsafe { &mut *raw_scene_graph_handle },
                                component_database,
                            );

                            create_prefab_serialization(
                                child,
                                serialized_node,
                                prefab_id,
                                serialized_graph,
                                prefab_members,
                                child_map,
                                component_database,
                                singleton_database,
                                scene_graph,
                                raw_scene_graph_handle,
                                resources,
                            );
                        } else {
                            error!("We couldn't make our Root Entity a SerializedEntity! Normally that means Prefab Corruption.");
                        }
                    }
                }
            }
            // We know this unsafeness is sound because `PrefabMarker`
            // does not alter the scene graph at all.
            let raw_scene_graph_handle: *mut SceneGraph = scene_graph;
            let mut child_map = HashMap::new();

            create_prefab_serialization(
                scene_graph_id,
                root_serialized_graph_id,
                prefab_id,
                &mut serialized_graph,
                &mut members,
                &mut child_map,
                component_database,
                singleton_database,
                scene_graph,
                raw_scene_graph_handle,
                resources,
            );

            // Commit the Parent to the Prefab
            commit_entity_to_prefab(
                entity,
                serialized_entity,
                prefab_id,
                root_member_id,
                &mut members,
                Some(child_map),
                resources,
                singleton_database,
                scene_graph,
                component_database,
            );
        }

        let prefab = Prefab::new(members, serialized_graph, PrefabId::new());

        // We can do this because we know no one else shares our prefab,
        // and we're sorting out fixing our own overrides below.
        let _ = serialize_and_cache_prefab(prefab, resources);
    } else {
        error!("We couldn't make our Root Entity a SerializedEntity! Normally that means Prefab Corruption.");
    }
    Ok(())
}

/// This creates an entity from a prefab into a Scene.
pub fn instantiate_entity_from_prefab(ecs: &mut Ecs, prefab_id: PrefabId, prefab_map: &PrefabMap) -> Entity {
    // Make an entity
    let entity = ecs.create_entity();

    // If we're in Draft mode, let's make an ID:
    let serialization_id = if super::scene_system::current_scene_mode() == super::SceneMode::Draft {
        let serialization_id = SerializationId::new();
        ecs.component_database.serialization_markers.set_component(
            &entity,
            super::SerializationMarker::with_id(serialization_id),
            &mut ecs.scene_graph,
        );
        Some(serialization_id)
    } else {
        None
    };

    // Instantiate the Prefab
    let success = ecs.component_database.load_serialized_prefab(
        &entity,
        serialization_id.map(|sid| (sid, &None)),
        prefab_map.get(&prefab_id).cloned(),
        &mut ecs.scene_graph,
        &mut ecs.entity_allocator,
        &mut ecs.entities,
        &mut ecs.singleton_database.associated_entities,
    );

    if let Some(post) = success {
        let scene_graph = &ecs.scene_graph;
        ecs.component_database
            .post_deserialization(post, |component_list, sl| {
                if let Some((inner, _)) = component_list.get_for_post_deserialization(&entity) {
                    inner.post_deserialization(entity, sl, scene_graph);
                }
            });
    } else {
        if ecs.remove_entity(&entity) == false {
            error!("We couldn't remove the Entity either, so we have a dangler!");
        }
    }

    entity
}

/// Serializes and caches a prefab, cycling it through the Serde. This is done because data made from Live
/// Data might not be in the state that we'd expect it to be in if we store it live. Live -> Live Storage -> Live
/// is bad. We need a stop in Serde town!
pub fn serialize_and_cache_prefab(prefab: Prefab, resources: &mut ResourcesDatabase) -> PrefabLoadRequired {
    if let Err(e) = serialization_util::prefabs::serialize_prefab(&prefab) {
        error!("Error Creating Prefab: {}", e);
    }

    match serialization_util::prefabs::cycle_prefab(prefab) {
        Ok(prefab) => {
            resources.add_prefab(prefab);
        }
        Err(e) => error!("We couldn't cycle the Prefab! It wasn't saved! {}", e),
    }

    PrefabLoadRequired
}

/// Use this to finish committing an override to a prefab. This only handles
/// a difference on COMPONENTS. If you add a child to a prefab, and
/// want to add that to the prefab, fucking good luck buddy
pub fn update_prefab_inheritor_component(
    _: PrefabLoadRequired,
    prefab_id: PrefabId,
    member_id: SerializationId,
    key: serde_yaml::Value,
    delta: serde_yaml::Value,
    ecs: &mut Ecs,
    resources: &ResourcesDatabase,
) -> Result<()> {
    let mut post_deserialization = None;
    let mut entities_to_post_deserialize = vec![];

    for entity in ecs.entities.iter() {
        if ecs
            .component_database
            .prefab_markers
            .get(entity)
            .map_or(false, |pmc| {
                let pm = pmc.inner();
                pm.prefab_id() == prefab_id && pm.member_id() == member_id
            })
        {
            // Load the Delta into each existing Prefab inheritor
            let new_post = ecs.component_database.load_yaml_delta_into_database(
                entity,
                key.clone(),
                delta.clone(),
                Default::default(),
                &mut ecs.singleton_database.associated_entities,
                &mut ecs.scene_graph,
            );

            // Reload the serialization after the fact
            post_deserialization = Some(new_post);
            entities_to_post_deserialize.push((
                *entity,
                ecs.component_database
                    .serialization_markers
                    .get(entity)
                    .map(|se| se.inner().id),
            ));
        }
    }

    if let Some(pd) = post_deserialization {
        let scene_graph = &ecs.scene_graph;

        ecs.component_database
            .post_deserialization(pd, |component_list, sl| {
                for (entity, _) in entities_to_post_deserialize.iter_mut() {
                    if let Some((inner, _)) = component_list.get_for_post_deserialization(&entity) {
                        inner.post_deserialization(*entity, sl, scene_graph);
                    }
                }
            });

        let mut serialized_entities =
            serialization_util::entities::load_all_entities().with_context(|| {
                format!(
                    "We couldn't load Scene {}.",
                    super::scene_system::current_scene_name()
                )
            })?;

        for (entity, serialization_id) in entities_to_post_deserialize.iter_mut() {
            if let Some(serialized_entity) = serialization_id.and_then(|si| serialized_entities.get_mut(&si))
            {
                if let Some(new_se) = SerializedEntity::new(
                    entity,
                    serialized_entity.id,
                    &ecs.component_database,
                    &ecs.singleton_database,
                    resources,
                ) {
                    *serialized_entity = new_se;
                }
            }
        }

        serialization_util::entities::commit_all_entities(&serialized_entities)?;
    }

    Ok(())
}

/// This gets the parent prefab of a given inheritor.
/// To make this simpler, imagine Player's parent Prefab is
/// Actor. If Player's entity was passed into this method,
/// a Serialized Actor would come out.
///
/// Returns a **flag** indicating if a prefab was found,
/// which will have been loaded into the SerializedEntity provided.
pub fn get_serialized_parent_prefab_from_inheritor(
    maybe_prefab_marker: Option<&Component<PrefabMarker>>,
    resources: &ResourcesDatabase,
    serialized_entity: &mut SerializedEntity,
) -> bool {
    if let Some(prefab_component) = maybe_prefab_marker {
        let prefab = match resources.prefabs().get(&prefab_component.inner().prefab_id()) {
            Some(i) => i,
            None => return false,
        };

        let mut serialized_prefab = match prefab.members.get(&prefab_component.inner().member_id()) {
            Some(sp) => sp.clone(),
            None => return false,
        };

        serialized_prefab.prefab_marker = Some(SerializedComponent {
            active: true,
            inner: prefab_component.inner().clone(),
        });

        *serialized_entity = serialized_prefab;
        true
    } else {
        false
    }
}

/// This uses the *experimental* idea of some dynamic typings in YAML! These unwraps *should*
/// be safe, as we know that SerializedEntity can be safely serialized and deserialized.
pub fn load_override_into_prefab(
    prefab_serialized_entity: SerializedEntity,
    se_override: SerializedEntity,
) -> Result<SerializedEntity> {
    let mut prefab_serialized_yaml = serde_yaml::to_value(prefab_serialized_entity).unwrap();
    let se_override_yaml = serde_yaml::to_value(se_override).unwrap();

    let prefab_serialized_value_as_map = prefab_serialized_yaml.as_mapping_mut().unwrap();

    if let YamlValue::Mapping(mapping) = se_override_yaml {
        for (key, value) in mapping {
            if value != serde_yaml::Value::Null {
                prefab_serialized_value_as_map.insert(key, value);
            }
        }
    }

    serde_yaml::from_value(prefab_serialized_yaml)
        .with_context(|| format!("We could not transform a composed YAML SE back to SE",))
}
