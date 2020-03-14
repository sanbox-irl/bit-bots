use super::{
    imgui_component_utils::{EntitySerializationCommand, EntitySerializationCommandType},
    SerializationId, *,
};
pub type SerializedHashMap = std::collections::HashMap<SerializationId, SerializedEntity>;

/// Loads all the entities as a SerializedHashMap. If this is a Prefab Scene, it will
/// load `Prefab.Members`, or the `entities_data.yaml` otherwise.
pub fn load_all_entities(scene: &Scene) -> Result<SerializedHashMap, Error> {
    let (scene_entity_path, is_prefab) = path(scene);
    if is_prefab {
        let prefab: Prefab = load_serialized_file(&scene_entity_path)?;
        Ok(prefab.members)
    } else {
        load_serialized_file(&scene_entity_path)
    }
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
            let post = ecs.load_serialized_entity(&command.entity, serialized_entity, resources.prefabs());

            if let Some(post) = post {
                let scene_graph = &ecs.scene_graph;
                ecs.component_database
                    .post_deserialization(post, |component_list, sl| {
                        if let Some((inner, _)) = component_list.get_for_post_deserialization(&command.entity)
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

/// A wrapper about `load_entity_by_id`, pulling out `serialized_data`'s `.id`.
/// Use this as a convenience function!
pub fn load_committed_entity(
    serialized_data: &SerializationMarker,
    scene: &Scene,
) -> Result<Option<SerializedEntity>, Error> {
    load_entity_by_id(&serialized_data.id, scene)
}

/// Gets an Entity from the SerializedHashMap by ID.
pub fn load_entity_by_id(id: &SerializationId, scene: &Scene) -> Result<Option<SerializedEntity>, Error> {
    let mut entities: SerializedHashMap = load_all_entities(scene)?;
    Ok(entities.remove(id))
}

/// The Path to the current Scene. Just a Helper
fn path(scene: &Scene) -> (String, bool) {
    (scene.entity_path(), scene.is_prefab())
}
