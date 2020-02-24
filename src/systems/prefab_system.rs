use super::{
    serialization_util, Component, ComponentDatabase, Ecs, Entity, Name, Prefab, PrefabLoadRequired,
    PrefabMap, PrefabMarker, ResourcesDatabase, SerializedComponent, SerializedEntity, SingletonDatabase,
};
use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use uuid::Uuid;

pub fn commit_blank_prefab(resources: &mut ResourcesDatabase) -> Result<uuid::Uuid> {
    let blank_prefab = Prefab::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = blank_prefab.root_id();
    resources.add_prefab(blank_prefab);
    Ok(id)
}

pub fn commit_new_prefab(
    entity: &Entity,
    component_database: &mut ComponentDatabase,
    singleton_database: &SingletonDatabase,
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
        new_prefab,
        component_database,
        singleton_database,
        resources,
    ) {
        let prefab = Prefab::new(serialized_entity);
        let prefab_id = prefab.root_id();

        // We can do this because we know no one else shares our prefab,
        // and we're sorting out fixing our own overrides below.
        let _ = serialize_and_cache_prefab(prefab, prefab_id, resources);

        // Add our Prefab Marker back to the Original entity we made into a prefab...
        component_database
            .prefab_markers
            .set_component(entity, PrefabMarker::new_main(prefab_id));

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

pub fn instantiate_entity_from_prefab(
    ecs: &mut Ecs,
    prefab_id: uuid::Uuid,
    prefab_map: &PrefabMap,
) -> Entity {
    // Make an entity
    let entity = ecs.create_entity();

    // Instantiate the Prefab
    let success = ecs.component_database.load_serialized_prefab(
        &entity,
        &prefab_id,
        &mut ecs.entity_allocator,
        &mut ecs.entities,
        prefab_map,
        &mut ecs.singleton_database.associated_entities,
    );

    if let Some(post) = success {
        ecs.component_database
            .post_deserialization(post, |component_list, sl| {
                if let Some((inner, _)) = component_list.get_mut(&entity) {
                    inner.post_deserialization(entity, sl);
                }
            });

        // Set our Prefab Marker
        ecs.component_database
            .prefab_markers
            .set_component(&entity, PrefabMarker::new_main(prefab_id));
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
pub fn serialize_and_cache_prefab(
    prefab: Prefab,
    sub_id: Uuid,
    resources: &mut ResourcesDatabase,
) -> PrefabLoadRequired {
    if let Err(e) = serialization_util::prefabs::serialize_prefab(&prefab) {
        error!("Error Creating Prefab: {}", e);
    }

    let main_id = prefab.root_id();

    match serialization_util::prefabs::cycle_prefab(prefab) {
        Ok(prefab) => {
            resources.add_prefab(prefab);
        }
        Err(e) => error!("We couldn't cycle the Prefab! It wasn't saved! {}", e),
    }

    PrefabLoadRequired { main_id, sub_id }
}

/// Use this to finish a prefab serialization. This is a fairly huge operation,
/// so be careful with it.
pub fn post_prefab_serialization(
    ecs: &mut Ecs,
    key: serde_yaml::Value,
    delta: serde_yaml::Value,
    prefab_load: PrefabLoadRequired,
) -> Result<()> {
    let PrefabLoadRequired { main_id, sub_id } = prefab_load;
    let mut post_deserialization = None;
    let mut entities_to_post_deserialize = vec![];

    for entity in ecs.entities.iter() {
        if ecs
            .component_database
            .prefab_markers
            .get(entity)
            .map(|pmc| {
                let pm = pmc.inner();
                pm.main_id() == main_id && pm.sub_id() == sub_id
            })
            .unwrap_or_default()
        {
            // Load the Delta into each existing Prefab inheritor
            let new_post = ecs.component_database.load_yaml_delta_into_database(
                entity,
                key.clone(),
                delta.clone(),
                Default::default(),
                &mut ecs.singleton_database.associated_entities,
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
                    if let Some((inner, _)) = component_list.get_mut(&entity) {
                        inner.post_deserialization(*entity, sl);
                    }
                }
            });
        let serialized_entities: HashMap<Uuid, SerializedEntity> =
            serialization_util::entities::load_all_entities().with_context(|| {
                format!(
                    "We couldn't load Scene {}.",
                    super::scene_system::current_scene_name()
                )
            })?;

        let new_serialized_entities: HashMap<Uuid, SerializedEntity> = {
            let mut serialized_entities_value: YamlValue = serde_yaml::to_value(serialized_entities).unwrap();

            let serialized_entities_map = serialized_entities_value.as_mapping_mut().unwrap();
            for (_, id) in entities_to_post_deserialize {
                // find the entity in the Hashmap
                let id_key = serde_yaml::to_value(id).unwrap();

                // Find the key...
                if let Some(serialized_entity) = serialized_entities_map.get_mut(&id_key) {
                    let entity_as_map = serialized_entity.as_mapping_mut().unwrap();
                    // And put a null in it! in the future, we'll be removing it, but for now
                    // we can do that until we switch over to purely dynamic typed saves
                    entity_as_map.insert(key.clone(), YamlValue::Null);
                }
            }

            serde_yaml::from_value(serialized_entities_value).unwrap()
        };

        serialization_util::entities::commit_all_entities(&new_serialized_entities)?;
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
        let prefab = match resources.prefabs().get(&prefab_component.inner().main_id()) {
            Some(i) => i,
            None => return false,
        };

        let mut serialized_prefab = match prefab.members.get(&prefab_component.inner().sub_id()) {
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
