use super::{imgui_component_utils::*, *};
use anyhow::Error;

pub fn entity_list(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
) -> Result<Option<EntitySerializationCommand>, Error> {
    let mut open = true;
    let mut later_action_on_entity: Option<(Entity, NameRequestedAction)> = None;

    if open == false {
        ui_handler.flags.remove(ImGuiFlags::ENTITY_VIEWER);
    }

    imgui_entity_list(ecs, resources, ui_handler, &mut open, &mut later_action_on_entity);

    if let Some((entity, later_action)) = later_action_on_entity {
        match later_action {
            NameRequestedAction::ChangeName(new_name) => {
                let name_component = ecs.component_database.names.get_mut_or_default(&entity);
                name_component.inner_mut().name = new_name;
            }
            NameRequestedAction::ToggleInspect => {
                if ui_handler.stored_ids.contains(&entity) {
                    ui_handler.stored_ids.remove(&entity);
                } else {
                    ui_handler.stored_ids.insert(entity.clone());
                }
            }
            NameRequestedAction::Clone => {
                let new_entity = ecs.clone_entity(&entity);

                let names: *const ComponentList<Name> = &mut ecs.component_database.names;
                if let Some(name) = ecs.component_database.names.get_mut(&new_entity) {
                    name.inner_mut().update_name(new_entity, unsafe { &*names });
                }
            }
            NameRequestedAction::Delete => {
                ecs.remove_entity(&entity);
                ui_handler.stored_ids.remove(&entity);
            }
            NameRequestedAction::GoToPrefab => {
                if let Some(prefab_marker) = ecs.component_database.prefab_markers.get(&entity) {
                    let id = prefab_marker.inner().main_id();
                    if scene_system::set_next_scene(Scene::new_prefab(id)) == false {
                        error!("Couldn't switch to Prefab {}", id);
                        error!("Does a Prefab by that name exist?");
                    }
                } else {
                    error!(
                        "{} requested to view its Prefab, but it had no PrefabMarker!",
                        Name::get_name_quick(&ecs.component_database.names, &entity)
                    );
                }
            }
            NameRequestedAction::PromoteToPrefab => {
                prefab_system::commit_new_prefab(
                    &entity,
                    &mut ecs.component_database,
                    &ecs.singleton_database,
                    resources,
                )?;
            }

            NameRequestedAction::UnpackPrefab => {
                let mut success = false;

                if let Some(prefab_marker) = ecs.component_database.prefab_markers.get(&entity) {
                    if let Some(serialization_marker) =
                        ecs.component_database.serialization_markers.get(&entity)
                    {
                        let serialized_entity = serialization_util::entities::load_committed_entity(
                            &serialization_marker.inner(),
                        );

                        if let Ok(Some(mut serialized_entity)) = serialized_entity {
                            prefab_marker.inner().uncommit_to_scene(&mut serialized_entity);

                            success = serialization_util::entities::commit_entity_to_serialized_scene(
                                serialized_entity,
                            )
                            .is_ok();
                        }
                    }
                }

                if success {
                    ecs.component_database.prefab_markers.unset(&entity);
                } else {
                    error!(
                        "We couldn't unpack entity {}! It should still be safely serialized as a prefab.",
                        Name::get_name_quick(&ecs.component_database.names, &entity)
                    );
                }
            }

            NameRequestedAction::LogPrefab => {
                if let Some(prefab_marker) = ecs.component_database.prefab_markers.get(&entity) {
                    if let Some(prefab) = resources.prefabs().get(&prefab_marker.inner().main_id()) {
                        prefab.log_to_console();
                    } else {
                        info!(
                            "{} had a PrefabMarker but no Prefab was found in the Cache!",
                            Name::get_name_quick(&ecs.component_database.names, &entity)
                        );
                    }
                } else {
                    info!(
                        "{} requested to view its Prefab, but it had no PrefabMarker!",
                        Name::get_name_quick(&ecs.component_database.names, &entity)
                    );
                }
            }
            NameRequestedAction::LogSerializedEntity => {
                if let Some(serialization_marker) =
                    ecs.component_database.serialization_markers.get_mut(&entity)
                {
                    if let Some(cached) = serialization_marker.inner_mut().cached_serialized_entity() {
                        cached.log_to_console();
                    } else {
                        error!(
                            "We didn't have a Cached Serialized Entity. Is there a problem with the caching?"
                        );
                    }
                }
            }
            NameRequestedAction::LogEntity => {
                println!("---Console Dump for {}---", entity);
                ecs.component_database.foreach_component_list_mut(
                    NonInspectableEntities::all(),
                    |comp_list| {
                        comp_list.dump_to_log(&entity);
                    },
                );
                println!("-------------------------");
            }

            NameRequestedAction::EntitySerializationCommand(entity_serialization_command) => {
                let uuid = ecs
                    .component_database
                    .serialization_markers
                    .get(&entity)
                    .map(|smc| smc.inner().id)
                    .unwrap();

                return Ok(Some(EntitySerializationCommand {
                    entity,
                    id: uuid,
                    command_type: entity_serialization_command,
                }));
            }
        }
    }

    Ok(None)
}

/// This is in a separate function to make it clear that ImGui code must always
/// handle its own errors locally. Use `match` or `if let Err` to identify errors,
/// and log them using `error!()`.
fn imgui_entity_list(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
    open: &mut bool,
    later_action_on_entity: &mut Option<(Entity, NameRequestedAction)>,
) {
    // Top menu bar!
    let entity_window = imgui::Window::new(&im_str!("Entity List"))
        .size([200.0, 400.0], imgui::Condition::FirstUseEver)
        .menu_bar(true)
        .opened(open);

    if let Some(entity_inspector_window) = entity_window.begin(&ui_handler.ui) {
        // Top menu bar!
        if let Some(menu_bar) = ui_handler.ui.begin_menu_bar() {
            let ui: &Ui<'_> = &ui_handler.ui;
            // BLANK ENTITY
            if imgui::MenuItem::new(im_str!("Create Blank Entity")).build(ui) {
                ecs.create_entity();
            }

            // PREFABS
            if let Some(prefab_submenu) = ui.begin_menu(im_str!("Instantiate Prefabs"), true) {
                for (prefab_id, prefab) in resources.prefabs().iter() {
                    let name = match &prefab.root_entity().name {
                        Some(sc) => im_str!("{}##MenuItem", &sc.inner.name),
                        None => im_str!("ID: {}##MenuItem", prefab.root_id()),
                    };

                    if imgui::MenuItem::new(&name).build(ui) {
                        let entity = prefab_system::instantiate_entity_from_prefab(
                            ecs,
                            *prefab_id,
                            resources.prefabs(),
                        );
                        if scene_system::current_scene_mode() == SceneMode::Draft {
                            ecs.component_database
                                .serialization_markers
                                .set_component(&entity, SerializationMarker::new());
                        }
                    }
                }

                if resources.prefabs().is_empty() {
                    imgui::MenuItem::new(imgui::im_str!("(None -- Get Crackin'!)"))
                        .enabled(false)
                        .build(ui);
                }

                prefab_submenu.end(ui);
            }

            if imgui::MenuItem::new(im_str!("Serialize Scene")).build(ui) || ui_handler.can_save_scene() {
                match serialization_util::entities::serialize_all_entities(
                    &ecs.entities,
                    &ecs.component_database,
                    &ecs.singleton_database,
                    resources,
                ) {
                    Ok(()) => info!("Serialized Scene"),
                    Err(e) => {
                        error!("Error on Serialization: {}", e);
                    }
                }
            }

            menu_bar.end(ui);
        }

        ui_handler.scene_graph_entities.clear();

        // SCENE GRAPH
        scene_graph::walk_graph_inspect(
            &mut ecs.component_database,
            &mut ecs.singleton_database,
            resources,
            &mut |entity,
                  names,
                  serialization_data,
                  current_serialized_entity,
                  prefabs,
                  mut name_inspector_params| {
                // Update Name Inspector Parameter:
                name_inspector_params.serialization_status = serialization_data
                    .get_mut(entity)
                    .map(|smc| {
                        smc.inner_mut()
                            .get_serialization_status(current_serialized_entity.as_ref())
                    })
                    .unwrap_or_default();

                name_inspector_params.being_inspected = ui_handler.stored_ids.contains(entity);
                name_inspector_params.prefab_status = prefabs
                    .get(entity)
                    .map(|_| PrefabStatus::PrefabInstance)
                    .unwrap_or_default();

                ui_handler.scene_graph_entities.push(*entity);

                let (show_children, requested_action) =
                    display_entity_id(entity, &name_inspector_params, names, ui_handler);

                if let Some(requested_action) = requested_action {
                    *later_action_on_entity = Some((*entity, requested_action));
                }

                show_children
            },
        );

        ui_handler.ui.separator();

        let component_database = &mut ecs.component_database;
        let singleton_database = &mut ecs.singleton_database;
        let entities = &ecs.entities;

        for entity in entities.iter() {
            if ui_handler.scene_graph_entities.contains(entity) == false {
                let serialization_status: SyncStatus = {
                    let serialization_id = component_database
                        .serialization_markers
                        .get(entity)
                        .map(|sc| sc.inner().id);

                    if let Some(s_id) = serialization_id {
                        let se = SerializedEntity::new(
                            entity,
                            s_id,
                            component_database,
                            singleton_database,
                            resources,
                        );

                        Some(
                            component_database
                                .serialization_markers
                                .get_mut(entity)
                                .as_mut()
                                .unwrap()
                                .inner_mut()
                                .get_serialization_status(se.as_ref()),
                        )
                    } else {
                        None
                    }
                    .unwrap_or_default()
                };

                let nip = NameInspectorParameters {
                    prefab_status: component_database
                        .prefab_markers
                        .get(entity)
                        .map(|_| PrefabStatus::PrefabInstance)
                        .unwrap_or_default(),
                    being_inspected: ui_handler.stored_ids.contains(entity),
                    depth: 0,
                    has_children: false,
                    serialization_status,
                };

                let (_, actions) = display_entity_id(entity, &nip, &mut component_database.names, ui_handler);
                if let Some(action) = actions {
                    *later_action_on_entity = Some((*entity, action));
                }
            }
        }
        entity_inspector_window.end(&ui_handler.ui);
    }
}

fn display_entity_id(
    entity: &Entity,
    name_inspector_params: &NameInspectorParameters,
    names: &mut ComponentList<Name>,
    ui_handler: &mut UiHandler<'_>,
) -> (bool, Option<NameRequestedAction>) {
    // Find our ImGui entry list info
    let entity_list_info = ui_handler
        .entity_list_information
        .entry(entity.to_string())
        .or_default();

    let NameInspectorResult {
        show_children,
        requested_action,
    } = Name::inspect(
        names
            .get(entity)
            .map_or(&format!("{}", entity), |name| &name.inner().name),
        entity_list_info,
        name_inspector_params,
        &ui_handler.ui,
        &entity.index().to_string(),
    );

    (show_children, requested_action)
}
