use super::{
    scene_graph::SceneGraph, serialization_util, Component, ComponentDatabase, Ecs, Entity, Name, Prefab,
    PrefabId, PrefabLoadRequired, PrefabMap, PrefabMarker, ResourcesDatabase, SerializationId,
    SerializedComponent, SerializedEntity, SingletonDatabase,
};
use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;

pub fn commit_blank_prefab(resources: &mut ResourcesDatabase) -> Result<PrefabId> {
    let blank_prefab = Prefab::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = *blank_prefab.prefab_id();
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
    let new_prefab = commit_blank_prefab(resources).with_context(|| {
        format!(
            "We create a new Prefab from {}",
            Name::get_name_quick(&component_database.names, entity)
        )
    })?;

    // Create a serialized entity
    if let Some(serialized_entity) = SerializedEntity::new(
        entity,
        SerializationId::new(),
        component_database,
        singleton_database,
        resources,
    ) {
        let prefab = Prefab::new(serialized_entity, new_prefab);
        let prefab_id = *prefab.prefab_id();
        let our_id = *prefab.root_id();

        // We can do this because we know no one else shares our prefab,
        // and we're sorting out fixing our own overrides below.
        let _ = serialize_and_cache_prefab(prefab, resources);

        // Add our Prefab Marker back to the Original entity we made into a prefab...
        component_database.prefab_markers.set_component(
            entity,
            PrefabMarker::new(prefab_id, our_id, None),
            scene_graph,
        );

        // And if it's serialized, let's cycle our Serialization too!
        // We do this to remove the "Overrides" that would otherwise appear
        if let Some(sc) = component_database.serialization_markers.get(entity) {
            serialization_util::entities::serialize_entity_full(
                entity,
                sc.inner().id,
                component_database,
                singleton_database,
                resources,
            );
        }
    }
    Ok(())
}

/// This creates an entity from a prefab into a Scene.
pub fn instantiate_entity_from_prefab(ecs: &mut Ecs, prefab_id: PrefabId, prefab_map: &PrefabMap) -> Entity {
    // Make an entity
    let entity = ecs.create_entity();

    // Instantiate the Prefab
    let success = ecs.component_database.load_serialized_prefab(
        &entity,
        &None,
        prefab_map.get(&prefab_id).cloned(),
        &mut ecs.scene_graph,
        &mut ecs.entity_allocator,
        &mut ecs.entities,
        &mut ecs.singleton_database.associated_entities,
    );

    if let Some(post) = success {
        ecs.component_database
            .post_deserialization(post, |component_list, sl| {
                if let Some((inner, _)) = component_list.get_for_post_deserialization(&entity) {
                    inner.post_deserialization(entity, sl);
                }
            });
    } else {
        if ecs.remove_entity(&entity) == false {
            error!("We couldn't remove the Entity either, so we have a dangler!");
        }
    }

    entity
}

/// Serializes and caches a prefab, but it doesn't perform anything more complicated
/// than that. Use the returned `PrefabLoadRequired` with `post_prefab_serialization`
/// to finish the operation up.
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
        ecs.component_database
            .post_deserialization(pd, |component_list, sl| {
                for (entity, _) in entities_to_post_deserialize.iter_mut() {
                    if let Some((inner, _)) = component_list.get_for_post_deserialization(&entity) {
                        inner.post_deserialization(*entity, sl);
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
