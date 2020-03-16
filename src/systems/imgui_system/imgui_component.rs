use super::{imgui_component_utils::*, *};
use anyhow::Error;
use imgui::{Condition, MenuItem, StyleColor, StyleVar, Window};
use imgui_utility::imgui_str;

pub fn entity_inspector(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
) -> Result<Option<EntitySerializationCommand>, Error> {
    let ui: &Ui<'_> = &ui_handler.ui;
    let mut remove_this_entity = None;
    let mut final_post_action: Option<ComponentInspectorPostAction> = None;

    let Ecs {
        component_database,
        singleton_database,
        entity_allocator: _,
        scene_graph,
        entities,
        scene_data,
    } = ecs;

    for entity in ui_handler.stored_ids.iter() {
        let mut window_is_open = true;

        let window_name = {
            match component_database.names.get_mut(entity) {
                Some(name) => im_str!("{} (Scene Entity)###{}", &name.inner().name, entity),
                None => im_str!("{} (Scene Entity)", entity),
            }
        };

        let serialized_prefab = {
            let mut base_entity = SerializedEntity::default();

            if prefab_system::get_serialized_parent_prefab_from_inheritor(
                component_database.prefab_markers.get(entity),
                resources,
                &mut base_entity,
            ) {
                Some(base_entity)
            } else {
                None
            }
        };

        let names = &component_database.names;
        let serialized_entity = scene_data
            .tracked_entities()
            .get(&entity)
            .and_then(|serialization_id| {
                let base_entity = serialized_prefab.clone().unwrap_or_default();
                let cached_se = scene_data
                    .saved_serialized_entities()
                    .get(serialization_id)
                    .cloned()
                    .unwrap_or_default();

                prefab_system::load_override_into_prefab(base_entity, cached_se)
                    .map_err(|e| {
                        error!(
                            "We failed to override our prefab for {} because {}",
                            Name::get_name_quick(names, entity),
                            e
                        );
                    })
                    .ok()
            });

        let should_have_prefab = component_database.prefab_markers.get(entity).is_some();

        let entity_window = Window::new(&window_name)
            .size([600.0, 800.0], Condition::FirstUseEver)
            .position([1200.0, 100.0], Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut window_is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            // This unsafety is not actually unsafe at all -- Rust doesn't yet realize
            // that this method, though it takes `component_database`, doesn't involve
            // the field .names within component_database. If we use names, then this would
            // become a lot trickier.
            let names_raw_pointer: *const _ = &component_database.names;
            component_database.foreach_component_list_mut(
                NonInspectableEntities::empty(),
                |component_list| {
                    let possible_sync_statuses = component_list.get_sync_status(
                        entity,
                        serialized_prefab.as_ref(),
                        should_have_prefab,
                    );

                    let (deferred_serialization_command, delete) = component_list.component_inspector(
                        entity,
                        ecs.scene_data.scene().mode(),
                        possible_sync_statuses,
                        entities,
                        unsafe { &*names_raw_pointer },
                        resources.prefabs(),
                        ui,
                        window_is_open,
                    );

                    if delete {
                        component_list.unset_component(entity, scene_graph);
                    }

                    if let Some(deferred) = deferred_serialization_command {
                        final_post_action = Some(handle_serialization_command(
                            *entity,
                            deferred,
                            serialized_entity.as_ref(),
                            serialized_prefab.as_ref(),
                            component_list,
                        ));
                    }
                },
            );

            let prefab_status: PrefabStatus = if scene_data.scene().is_prefab() {
                PrefabStatus::Prefab
            } else {
                component_database
                    .prefab_markers
                    .get(entity)
                    .map_or(PrefabStatus::None, |pmc| {
                        pmc.inner().prefab_status(resources.prefabs())
                    })
            };

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                // Add Component Menubar
                if let Some(add_component_submenu) = ui.begin_menu(
                    &im_str!(
                        "Add {}",
                        if prefab_status.is_prefab_inheritor() {
                            "Override"
                        } else {
                            "Component"
                        }
                    ),
                    true,
                ) {
                    // Prefab Marker, Name is omitted
                    component_database
                        .foreach_component_list_mut(NonInspectableEntities::empty(), |component_list| {
                            component_list.component_add_button(entity, ui, scene_graph)
                        });

                    add_component_submenu.end(ui);
                }
                menu_bar.end(ui);
            }

            entity_inspector_window.end(ui);
        }

        if window_is_open == false {
            remove_this_entity = Some(*entity);
        }
    }

    // This happens when someone closes a window
    if let Some(entity) = remove_this_entity {
        ui_handler.stored_ids.remove(&entity);
    }

    let entity_command = if let Some(final_post_action) = final_post_action {
        match final_post_action {
            ComponentInspectorPostAction::ComponentCommands(command) => {
                match command.command_type {
                    ComponentSerializationCommandType::Serialize
                    | ComponentSerializationCommandType::StopSerializing => {
                        let serialization_id = scene_data.tracked_entities().get(&command.entity).unwrap();

                        let serialized_entity = scene_data
                            .saved_serialized_entities()
                            .get(serialization_id)
                            .cloned()
                            .unwrap();

                        let mut serialized_yaml = serde_yaml::to_value(serialized_entity)?;

                        // Insert our New Serialization
                        serialized_yaml
                            .as_mapping_mut()
                            .unwrap()
                            .insert(command.key, command.delta);

                        let new_serialized_entity: SerializedEntity =
                            serde_yaml::from_value(serialized_yaml)?;

                        scene_data.serialize_entity(command.entity, new_serialized_entity);

                        // If we're in a Prefab Scene, update our Prefab Cache!
                        if scene_data.scene().is_prefab() {
                            // We can have two kinds of Prefabs -- SceneGraph prefabs and
                            // non SceneGraph prefabs. We need to support both.
                            // In both cases, we assume that we only have one "root":
                            // for SceneGraph types, we only have one RootNode
                            // for NonSceneGraph types, we only have one entity at all!

                            // Standard SceneGraph (has transform) prefabs:
                            let prefab_id = if let Some(first_root) = ecs.scene_graph.iter_roots().nth(0) {
                                component_database
                                    .prefab_markers
                                    .get(first_root.inner())
                                    .map(|pmc| pmc.inner().prefab_id())
                            } else {
                                // We assume it's a non-SceneGraph Prefab, which means it's the only
                                // entity in da scene!
                                ecs.component_database
                                    .prefab_markers
                                    .iter()
                                    .nth(0)
                                    .map(|only_prefab_marker| only_prefab_marker.inner().prefab_id())
                            };

                            if let Some(prefab_id) = prefab_id {
                                let cached_prefab =
                                    resources.prefabs_mut().unwrap().get_mut(&prefab_id).unwrap();

                                cached_prefab
                                    .members
                                    .insert(*serialization_id, new_serialized_entity);
                            } else {
                                // This is very unlikely!
                                error!("We were in a Prefab Scene but we couldn't find our Prefab.");
                                error!("The live game probably has a malformed Prefab cached right now!");
                            }
                        }
                    }

                    ComponentSerializationCommandType::Revert
                    | ComponentSerializationCommandType::RevertToParentPrefab => {
                        let serialization_id = scene_data
                            .tracked_entities()
                            .get(&command.entity)
                            .cloned()
                            .unwrap();

                        let ComponentSerializationCommand {
                            command_type: _,
                            delta,
                            entity,
                            key,
                        } = command;

                        let post_deserialization = component_database.load_yaml_delta_into_database(
                            &entity,
                            key,
                            delta,
                            serialization_id,
                            &mut singleton_database.associated_entities,
                            scene_graph,
                        );

                        component_database.post_deserialization(
                            post_deserialization,
                            scene_data.tracked_entities(),
                            |component_list, sl| {
                                if let Some((inner, _)) = component_list.get_for_post_deserialization(&entity)
                                {
                                    inner.post_deserialization(entity, sl, scene_graph);
                                }
                            },
                        )
                    }

                    ComponentSerializationCommandType::ApplyOverrideToParentPrefab => {
                        let (main_id, sub_id) = component_database
                            .prefab_markers
                            .get(&command.entity)
                            .map(|pm| (pm.inner().prefab_id(), pm.inner().member_id()))
                            .unwrap();

                        let mut prefab = serialization_util::prefabs::load_prefab(&main_id)?.unwrap();
                        let (new_member, _diff): (SerializedEntity, _) = {
                            let mut member_yaml =
                                serde_yaml::to_value(prefab.members.get(&sub_id).cloned().unwrap())?;

                            let diff = member_yaml
                                .as_mapping_mut()
                                .unwrap()
                                .insert(command.key.clone(), command.delta.clone());

                            (serde_yaml::from_value(member_yaml)?, diff)
                        };

                        let prefab_id = prefab.prefab_id();
                        let member_id = new_member.id;

                        // Add in our Member and Cache the Prefab
                        prefab.members.insert(new_member.id, new_member);
                        let prefab_reload_required =
                            prefab_system::serialize_and_cache_prefab(prefab, resources);

                        prefab_system::update_prefab_inheritor_component(
                            prefab_reload_required,
                            prefab_id,
                            member_id,
                            command.key,
                            command.delta,
                            ecs,
                            resources,
                        )?;
                    }
                }

                None
            }
            ComponentInspectorPostAction::EntityCommands(entity_command) => Some(entity_command),
        }
    } else {
        None
    };

    Ok(entity_command)
}

#[must_use]
pub fn component_inspector_raw<T>(
    comp: &mut Component<T>,
    scene_mode: SceneMode,
    prefab_sync_status: SyncStatus,
    entities: &[Entity],
    entity_names: &ComponentList<Name>,
    prefabs: &PrefabMap,
    ui: &Ui<'_>,
    is_open: bool,
    can_right_click: bool,
    mut f: impl FnMut(&mut T, InspectorParameters<'_, '_>),
) -> (Option<ComponentSerializationCommandType>, bool)
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    let mut requested_action = None;
    let mut delete = false;

    let name = super::imgui_system::typed_text_ui::<T>();
    let uid = &format!("{}{}", comp.entity_id(), &T::type_name());

    let default_color = ui.style_color(imgui::StyleColor::Text);
    let alpha_amount = if comp.is_active == false {
        Some(ui.push_style_var(imgui::StyleVar::Alpha(0.6)))
    } else {
        None
    };

    let text_color_token = ui.push_style_color(
        imgui::StyleColor::Text,
        prefab_sync_status.imgui_color(scene_mode),
    );

    ui.tree_node(&imgui::ImString::new(&name))
        .default_open(true)
        .frame_padding(false)
        .build(|| {
            imgui_utility::wrap_style_var(ui, StyleVar::Alpha(1.0), || {
                // This is the Hover here:
                if ui.is_item_hovered() {
                    ui.tooltip_text(match prefab_sync_status {
                        SyncStatus::Unsynced => "This Entity does not inherit from a Prefab.",
                        SyncStatus::Headless => "This Componet is HEADLESS to its PREFAB!",
                        SyncStatus::OutofSync => "Overriding Prefab Parent",
                        SyncStatus::Synced => "Synced to Prefab Parent",
                    });
                }

                // Sadly, we lack destructure assign
                if can_right_click {
                    let right_click_actions = component_inspector_right_click(
                        ui,
                        uid,
                        &mut comp.is_active,
                        default_color,
                        prefab_sync_status,
                    );
                    requested_action = right_click_actions.0;
                    delete = right_click_actions.1;
                }
            });

            if comp.is_active {
                imgui_system::wrap_style_color_var(ui, imgui::StyleColor::Text, default_color.into(), || {
                    let inspector_parameters = InspectorParameters {
                        is_open,
                        uid,
                        ui,
                        entities,
                        entity_names,
                        prefabs,
                    };
                    f(comp.inner_mut(), inspector_parameters);
                });
            }
        });

    text_color_token.pop(ui);
    if let Some(alpha_token) = alpha_amount {
        alpha_token.pop(ui);
    }

    (requested_action, delete)
}

fn component_inspector_right_click(
    ui: &Ui<'_>,
    uid: &str,
    is_active: &mut bool,
    default_color: ImColor,
    prefab_sync_status: SyncStatus,
) -> (Option<ComponentSerializationCommandType>, bool) {
    let mut requested_action = None;
    let mut delete = false;

    imgui_system::right_click_popup(ui, uid, || {
        imgui_utility::wrap_style_color_var(ui, StyleColor::Text, default_color.into(), || {
            MenuItem::new(&im_str!("Is Active##{}", uid)).build_with_ref(ui, is_active);

            if MenuItem::new(&im_str!("Delete##{}", uid)).build(ui) {
                delete = true;
            }

            ui.separator();

            if MenuItem::new(&imgui_str("Apply Overrides To Prefab", uid))
                .enabled(prefab_sync_status == SyncStatus::OutofSync)
                .build(ui)
            {
                requested_action = Some(ComponentSerializationCommandType::ApplyOverrideToParentPrefab);
            }

            if MenuItem::new(&imgui_str("Revert to Prefab", uid))
                .enabled(prefab_sync_status == SyncStatus::OutofSync)
                .build(ui)
            {
                requested_action = Some(ComponentSerializationCommandType::RevertToParentPrefab);
            }
        });
    });

    (requested_action, delete)
}

// impl<T> ComponentList<T>
// where
//     T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
// {
//     pub fn serialization_option_raw(
//         &self,
//         ui: &imgui::Ui<'_>,
//         entity_id: &Entity,
//         serialized_markers: &ComponentList<SerializationMarker>,
//     ) -> Option<ComponentSerializationCommandType> {
//         lazy_static::lazy_static! {
//             static ref SERIALIZE: &'static ImStr = im_str!("Serialize");
//             static ref DESERIALIZE: &'static ImStr = im_str!("Stop Serializing");
//             static ref REVERT: &'static ImStr = im_str!("Revert");
//         }

//         let type_name = ImString::new(imgui_system::typed_text_ui::<T>());
//         let component_exists = self.get(entity_id).is_some();
//         let mut output = None;

//         if serialized_markers.get(entity_id).is_some() {
//             if let Some(serde_menu) = ui.begin_menu(&type_name, component_exists) {
//                 if self.get(entity_id).is_some() {
//                     // Serialize
//                     if MenuItem::new(&SERIALIZE).build(ui) {
//                         output = Some(ComponentSerializationCommandType::Serialize);
//                     }

//                     // Deserialize
//                     if MenuItem::new(&DESERIALIZE).build(ui) {
//                         output = Some(ComponentSerializationCommandType::StopSerializing);
//                     }

//                     // Revert
//                     if MenuItem::new(&REVERT).build(ui) {
//                         output = Some(ComponentSerializationCommandType::Revert);
//                     }
//                 }
//                 serde_menu.end(ui);
//             }
//         }

//         output
//     }
// }

fn handle_serialization_command(
    entity: Entity,
    command_type: ComponentSerializationCommandType,
    serialized_entity: Option<&SerializedEntity>,
    serialized_prefab: Option<&SerializedEntity>,
    component_list: &dyn ComponentListBounds,
) -> ComponentInspectorPostAction {
    match command_type {
        ComponentSerializationCommandType::Serialize => {
            let mut delta = component_list.create_yaml_component(&entity);

            // Is our new delta the same as our Parents Component?
            // If it is, we're going to make our Delta NULL
            if let Some(serialized_prefab) = serialized_prefab {
                let mut serialized_prefab_as_yaml = serde_yaml::to_value(serialized_prefab.clone()).unwrap();
                if let Some(parent_component) = serialized_prefab_as_yaml
                    .as_mapping_mut()
                    .unwrap()
                    .remove(&component_list.get_yaml_component_key())
                {
                    if parent_component == delta {
                        delta = serde_yaml::Value::Null;
                    }
                }
            }

            ComponentInspectorPostAction::ComponentCommands(ComponentSerializationCommand {
                delta,
                command_type,
                key: component_list.get_yaml_component_key(),
                entity,
            })
        }
        ComponentSerializationCommandType::StopSerializing => {
            ComponentInspectorPostAction::ComponentCommands(ComponentSerializationCommand {
                delta: serde_yaml::Value::Null,
                command_type,
                key: component_list.get_yaml_component_key(),
                entity,
            })
        }
        ComponentSerializationCommandType::Revert => {
            let delta = {
                if let Some(serialized_entity) = serialized_entity {
                    component_list.get_yaml_component(serialized_entity)
                } else {
                    Default::default()
                }
            };
            ComponentInspectorPostAction::ComponentCommands(ComponentSerializationCommand {
                delta,
                command_type,
                key: component_list.get_yaml_component_key(),
                entity,
            })
        }
        ComponentSerializationCommandType::ApplyOverrideToParentPrefab => {
            ComponentInspectorPostAction::ComponentCommands(ComponentSerializationCommand {
                delta: component_list.create_yaml_component(&entity),
                command_type,
                key: component_list.get_yaml_component_key(),
                entity,
            })
        }
        ComponentSerializationCommandType::RevertToParentPrefab => {
            let delta = {
                if let Some(serialized_prefab) = serialized_prefab {
                    component_list.get_yaml_component(serialized_prefab)
                } else {
                    Default::default()
                }
            };
            ComponentInspectorPostAction::ComponentCommands(ComponentSerializationCommand {
                delta,
                command_type,
                key: component_list.get_yaml_component_key(),
                entity,
            })
        }
    }
}
