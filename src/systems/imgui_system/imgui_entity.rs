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
                let name_component = ecs
                    .component_database
                    .names
                    .get_mut_or_default(&entity, &mut ecs.scene_graph);
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
                    &mut ecs.scene_graph,
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
                    ecs.component_database
                        .prefab_markers
                        .unset_component(&entity, &mut ecs.scene_graph);
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

            NameRequestedAction::CreateEntityCommand(create_entity_command) => {
                process_entity_subcommand(create_entity_command, ecs, resources.prefabs())
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
    let scene_mode = scene_system::current_scene_mode();

    // Top menu bar!
    let entity_window = imgui::Window::new(&im_str!("Entity List"))
        .size([200.0, 400.0], imgui::Condition::FirstUseEver)
        .menu_bar(true)
        .opened(open);

    if let Some(entity_inspector_window) = entity_window.begin(&ui_handler.ui) {
        // Top menu bar!
        if let Some(menu_bar) = ui_handler.ui.begin_menu_bar() {
            let ui: &Ui<'_> = &ui_handler.ui;

            // Create a Top Level Entity
            if let Some(sub_command) =
                create_entity_submenu("Create Entity", true, None, resources.prefabs(), ui)
            {
                process_entity_subcommand(sub_command, ecs, resources.prefabs());
            }

            // Get Scene Graph Serialization Status:
            let scene_graph_serialization_status =
                match serialization_util::serialized_scene_graph::load_scene_graph() {
                    Ok(sg) => {
                        let serialized = scene_graph_system::create_serialized_graph(
                            &ecs.scene_graph,
                            &ecs.component_database.serialization_markers,
                        );

                        if sg == serialized {
                            SyncStatus::Synced
                        } else {
                            SyncStatus::OutofSync
                        }
                    }
                    Err(e) => {
                        error!("Couldn't read the scene graph! {}", e);
                        SyncStatus::Headless
                    }
                }
                .imgui_symbol(scene_mode);

            // Debug the Scene Graph
            if let Some(menu_token) =
                ui.begin_menu(&im_str!("Scene Graph {}", scene_graph_serialization_status), true)
            {
                if imgui::MenuItem::new(im_str!("Save")).build(ui) {
                    let ssg = scene_graph_system::create_serialized_graph(
                        &ecs.scene_graph,
                        &ecs.component_database.serialization_markers,
                    );
                    match serialization_util::serialized_scene_graph::save_serialized_scene_graph(ssg) {
                        Ok(()) => info!("Saved Serialized SceneGraph..."),
                        Err(e) => error!("Couldn't save scene graph...{}", e),
                    }
                }

                if imgui::MenuItem::new(im_str!("Log")).build(ui) {
                    ecs.scene_graph.pretty_print(&ecs.component_database.names);
                }

                menu_token.end(ui);
            }

            // Just a good button
            if imgui::MenuItem::new(im_str!("\u{f0c7}")).build(ui) || ui_handler.save_requested() {
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
        let singleton_database = &ecs.singleton_database;
        let component_database = &mut ecs.component_database;

        scene_graph_system::walk_tree_generically(&ecs.scene_graph, |entity, depth, has_children| {
            let serialized_entity: Option<SerializedEntity> = component_database
                .serialization_markers
                .get(entity)
                .and_then(|smc| {
                    SerializedEntity::new(
                        entity,
                        smc.inner().id,
                        component_database,
                        singleton_database,
                        resources,
                    )
                });

            let name_inspector_params = NameInspectorParameters {
                has_children,
                depth,
                prefab_status: component_database
                    .prefab_markers
                    .get(entity)
                    .map(|_| PrefabStatus::PrefabInstance)
                    .unwrap_or_default(),
                being_inspected: ui_handler.stored_ids.contains(entity),
                serialization_status: component_database
                    .serialization_markers
                    .get_mut(entity)
                    .map(|smc| {
                        smc.inner_mut()
                            .get_serialization_status(serialized_entity.as_ref())
                    })
                    .unwrap_or_default(),
                on_scene_graph: component_database
                    .transforms
                    .get(entity)
                    .and_then(|tc| tc.inner().scene_graph_node_id()),
                prefabs: resources.prefabs(),
                scene_mode,
            };

            let (show_children, requested_action) = display_entity_id(
                entity,
                &name_inspector_params,
                &mut component_database.names,
                ui_handler,
            );

            // Record the Requested Action
            if let Some(requested_action) = requested_action {
                *later_action_on_entity = Some((*entity, requested_action));
            }

            show_children
        });

        ui_handler.ui.separator();

        let component_database = &mut ecs.component_database;
        let singleton_database = &mut ecs.singleton_database;
        let entities = &ecs.entities;

        for entity in entities.iter() {
            if component_database
                .transforms
                .get(&entity)
                .map_or(false, |smc| smc.inner().scene_graph_node_id().is_some())
            {
                continue;
            }

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
                on_scene_graph: None,
                prefabs: resources.prefabs(),
                scene_mode,
            };

            let (_, actions) = display_entity_id(entity, &nip, &mut component_database.names, ui_handler);
            if let Some(action) = actions {
                *later_action_on_entity = Some((*entity, action));
            }
        }
        entity_inspector_window.end(&ui_handler.ui);
    }
}

fn display_entity_id(
    entity: &Entity,
    name_inspector_params: &NameInspectorParameters<'_>,
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
