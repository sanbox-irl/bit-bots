use super::{imgui_component_utils::*, *};
use anyhow::Error;
use imgui::{Condition, ImStr, ImString, MenuItem, StyleColor, StyleVar, Window};
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
        entities,
    } = ecs;

    let scene_is_prefab = {
        let scene_data = scene_system::CURRENT_SCENE.lock().unwrap();
        scene_data.is_prefab()
    };

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
        let serialized_entity = component_database
            .serialization_markers
            .get_mut(entity)
            .and_then(|se| {
                let base_entity = serialized_prefab.clone().unwrap_or_default();

                let cached_se: SerializedEntity = se
                    .inner_mut()
                    .cached_serialized_entity()
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
        let should_have_serialized_entity = component_database.serialization_markers.get(entity).is_some();

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
                        serialized_entity.as_ref(),
                        serialized_prefab.as_ref(),
                        should_have_serialized_entity,
                        should_have_prefab,
                    );

                    if let Some(command_type) = component_list.component_inspector(
                        entity,
                        possible_sync_statuses,
                        entities,
                        unsafe { &*names_raw_pointer },
                        resources.prefabs(),
                        ui,
                        window_is_open,
                    ) {
                        final_post_action = Some(handle_serialization_command(
                            *entity,
                            command_type,
                            serialized_entity.as_ref(),
                            serialized_prefab.as_ref(),
                            component_list,
                        ));
                    }
                },
            );

            let prefab_status = if scene_is_prefab {
                PrefabStatus::Prefab
            } else {
                if component_database.prefab_markers.get(entity).is_some() {
                    PrefabStatus::PrefabInstance
                } else {
                    PrefabStatus::None
                }
            };

            // Serialization Inspector
            if let Some(s_marker) = component_database.serialization_markers.get_mut(entity) {
                let (_, delete) = component_inspector_raw(
                    s_marker,
                    SyncStatus::Synced,
                    SyncStatus::Unsynced,
                    entities,
                    &component_database.names,
                    resources.prefabs(),
                    ui,
                    window_is_open,
                    false,
                    |inner, ip| {
                        if inner.entity_inspector_results(ip) {
                            final_post_action = Some(ComponentInspectorPostAction::EntityCommands(
                                EntitySerializationCommand {
                                    entity: *entity,
                                    id: inner.id,
                                    command_type: EntitySerializationCommandType::Overwrite,
                                },
                            ))
                        }
                    },
                );

                if delete {
                    final_post_action = Some(ComponentInspectorPostAction::EntityCommands(
                        EntitySerializationCommand {
                            entity: *entity,
                            id: s_marker.inner().id,
                            command_type: EntitySerializationCommandType::StopSerializing,
                        },
                    ));
                }
            }

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                // Add Component Menubar
                if let Some(add_component_submenu) = ui.begin_menu(
                    &im_str!(
                        "Add {}",
                        if prefab_status == PrefabStatus::PrefabInstance {
                            "Override"
                        } else {
                            "Component"
                        }
                    ),
                    true,
                ) {
                    // @update_components exception
                    let had_transform = component_database.transforms.get(entity).is_some();

                    // Prefab Marker, Name, Graph Node is omitted
                    component_database.foreach_component_list_mut(
                        NonInspectableEntities::SERIALIZATION,
                        |component_list| component_list.component_add_button(entity, ui),
                    );

                    if had_transform == false {
                        if let Some(new_transform) = component_database.transforms.get_mut(entity) {
                            scene_graph::add_to_scene_graph(
                                new_transform,
                                &component_database.serialization_markers,
                            );
                        }
                    }

                    add_component_submenu.end(ui);
                }

                // Serialization Menubar
                if let Some(serialization_submenu) = ui.begin_menu(
                    im_str!("Serialize"),
                    component_database.serialization_markers.get(entity).is_some(),
                ) {
                    if let Some(comp) = component_database.serialization_markers.get(entity) {
                        if let Some(post_inspector) = serialization_menu(
                            comp.inner(),
                            ui,
                            entity,
                            component_database,
                            serialized_entity.as_ref(),
                            serialized_prefab.as_ref(),
                        ) {
                            final_post_action = Some(post_inspector);
                        };
                    }

                    serialization_submenu.end(ui);
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
                        let uuid = component_database
                            .serialization_markers
                            .get(&command.entity)
                            .map(|sm| sm.inner().id)
                            .unwrap();

                        let mut serialized_yaml = serde_yaml::to_value(
                            serialization_util::entities::load_entity_by_id(&uuid)?.unwrap(),
                        )?;

                        // Insert our New Serialization
                        serialized_yaml
                            .as_mapping_mut()
                            .unwrap()
                            .insert(command.key, command.delta);

                        let new_serialized_entity: SerializedEntity =
                            serde_yaml::from_value(serialized_yaml)?;

                        serialization_util::entities::commit_entity_to_serialized_scene(
                            new_serialized_entity.clone(),
                        )?;

                        if scene_is_prefab {
                            // THIS IS A CRAZY PERSON WAY TO DO THIS. EVERYTHING CAN GO WRONG
                            // LETS DO IT:
                            let root_entity_id: Entity =
                                scene_graph::ROOT_NODES.lock().unwrap().children.as_ref().unwrap()[0]
                                    .target
                                    .unwrap();
                            let root_uuid = component_database
                                .serialization_markers
                                .get(&root_entity_id)
                                .map(|pmc| pmc.inner().id)
                                .unwrap();

                            let prefab = resources.prefabs_mut().unwrap().get_mut(&root_uuid).unwrap();
                            prefab.members.insert(uuid, new_serialized_entity);
                        }
                    }
                    ComponentSerializationCommandType::Revert
                    | ComponentSerializationCommandType::RevertToParentPrefab => {
                        let uuid = component_database
                            .serialization_markers
                            .get(&command.entity)
                            .map(|sm| sm.inner().id)
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
                            uuid,
                            &mut singleton_database.associated_entities,
                        );

                        component_database.post_deserialization(post_deserialization, |component_list, sl| {
                            if let Some((inner, _)) = component_list.get_mut(&entity) {
                                inner.post_deserialization(entity, sl);
                            }
                        })
                    }

                    ComponentSerializationCommandType::ApplyOverrideToParentPrefab => {
                        let (main_id, sub_id) = component_database
                            .prefab_markers
                            .get(&command.entity)
                            .map(|pm| (pm.inner().main_id(), pm.inner().sub_id()))
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

                        prefab.members.insert(new_member.id, new_member);

                        let prefab_reload_required =
                            prefab_system::serialize_and_cache_prefab(prefab, sub_id, resources);

                        prefab_system::post_prefab_serialization(
                            ecs,
                            command.key,
                            command.delta,
                            prefab_reload_required,
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

pub fn serialization_menu(
    serialized_marker: &SerializationMarker,
    ui: &Ui<'_>,
    entity: &Entity,
    component_database: &ComponentDatabase,
    current_serialized_entity: Option<&SerializedEntity>,
    current_prefab_parent: Option<&SerializedEntity>,
) -> Option<ComponentInspectorPostAction> {
    let mut post_action: Option<ComponentInspectorPostAction> = None;

    component_database.foreach_component_list(
        NonInspectableEntities::NAME | NonInspectableEntities::PREFAB | NonInspectableEntities::GRAPH_NODE,
        |component_list| {
            if let Some(command_type) =
                component_list.serialization_option(ui, entity, &component_database.serialization_markers)
            {
                post_action = Some(handle_serialization_command(
                    *entity,
                    command_type,
                    current_serialized_entity,
                    current_prefab_parent,
                    component_list,
                ));
            }
        },
    );

    ui.separator();

    // REVERT SAVE
    if ui.button(im_str!("Revert"), [0.0, 0.0]) {
        post_action = Some(ComponentInspectorPostAction::EntityCommands(
            EntitySerializationCommand {
                entity: *entity,
                id: serialized_marker.id,
                command_type: EntitySerializationCommandType::Revert,
            },
        ));
    }

    // OVERWRITE
    ui.same_line(0.0);
    if ui.button(im_str!("Overwrite"), [0.0, 0.0]) {
        post_action = Some(ComponentInspectorPostAction::EntityCommands(
            EntitySerializationCommand {
                entity: *entity,
                id: serialized_marker.id,
                command_type: EntitySerializationCommandType::Revert,
            },
        ));
    }

    post_action
}

#[must_use]
pub fn component_inspector_raw<T>(
    comp: &mut Component<T>,
    serialization_sync_status: SyncStatus,
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

    let scene_mode = scene_system::current_scene_mode();
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
                        serialization_sync_status,
                        prefab_sync_status,
                    );
                    requested_action = right_click_actions.0;
                    delete = right_click_actions.1;
                }

                // Handle the Warning!
                match serialization_sync_status {
                    SyncStatus::Unsynced => {
                        imgui_system::wrap_style_color_var(
                            ui,
                            imgui::StyleColor::Text,
                            imgui_utility::yellow_warning_color(),
                            || {
                                if scene_mode == SceneMode::Draft {
                                    imgui_utility::help_marker_generic(
                                        ui,
                                        imgui_utility::WARNING_ICON,
                                        format!("{} is not committed to the Scene!", name),
                                    )
                                }
                            },
                        );
                    }
                    SyncStatus::Headless => {
                        imgui_system::wrap_style_color_var(
                            ui,
                            imgui::StyleColor::Text,
                            imgui_utility::red_warning_color(),
                            || {
                                imgui_utility::help_marker_generic(
                                    ui,
                                    imgui_utility::WARNING_ICON,
                                    format!(
                                "{} is Headless! It has a serialization marker, but no serialization found!",
                                name
                            ),
                                )
                            },
                        );
                    }
                    SyncStatus::OutofSync => {
                        imgui_system::wrap_style_color_var(
                            ui,
                            imgui::StyleColor::Text,
                            imgui_utility::yellow_warning_color(),
                            || {
                                imgui_utility::help_marker_generic(
                                    ui,
                                    imgui_utility::WARNING_ICON,
                                    format!("{} is out of Sync with its Serialization!", name),
                                );
                            },
                        );
                    }
                    SyncStatus::Synced => {
                        imgui_system::wrap_style_color_var(
                            ui,
                            imgui::StyleColor::Text,
                            imgui_utility::green_color(),
                            || {
                                imgui_utility::help_marker_generic(
                                    ui,
                                    imgui_utility::SYNCED_ICON,
                                    format!("{} is synced to its Serialization!", name),
                                );
                            },
                        );
                    }
                };
            });

            if comp.is_active {
                imgui_system::wrap_style_color_var(ui, imgui::StyleColor::Text, default_color, || {
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
    serialization_sync_status: SyncStatus,
    prefab_sync_status: SyncStatus,
) -> (Option<ComponentSerializationCommandType>, bool) {
    let mut requested_action = None;
    let mut delete = false;

    imgui_system::right_click_popup(ui, uid, || {
        imgui_utility::wrap_style_color_var(ui, StyleColor::Text, default_color, || {
            MenuItem::new(&im_str!("Is Active##{}", uid)).build_with_ref(ui, is_active);

            if MenuItem::new(&im_str!("Delete##{}", uid)).build(ui) {
                delete = true;
            }

            ui.separator();

            if MenuItem::new(&imgui_str("Serialize", uid))
                .enabled(serialization_sync_status == SyncStatus::OutofSync)
                .build(ui)
            {
                requested_action = Some(ComponentSerializationCommandType::Serialize);
            }

            if MenuItem::new(&imgui_str("Stop Serializing", uid))
                .enabled(serialization_sync_status.is_synced_at_all())
                .build(ui)
            {
                requested_action = Some(ComponentSerializationCommandType::StopSerializing);
            }

            if MenuItem::new(&imgui_str("Revert to Serialization", uid))
                .enabled(serialization_sync_status == SyncStatus::Unsynced)
                .build(ui)
            {
                requested_action = Some(ComponentSerializationCommandType::Revert);
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

impl<T> ComponentList<T>
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    pub fn serialization_option_raw(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        serialized_markers: &ComponentList<SerializationMarker>,
    ) -> Option<ComponentSerializationCommandType> {
        lazy_static::lazy_static! {
            static ref SERIALIZE: &'static ImStr = im_str!("Serialize");
            static ref DESERIALIZE: &'static ImStr = im_str!("Stop Serializing");
            static ref REVERT: &'static ImStr = im_str!("Revert");
        }

        let type_name = ImString::new(imgui_system::typed_text_ui::<T>());
        let component_exists = self.get(entity_id).is_some();
        let mut output = None;

        if serialized_markers.get(entity_id).is_some() {
            if let Some(serde_menu) = ui.begin_menu(&type_name, component_exists) {
                if self.get(entity_id).is_some() {
                    // Serialize
                    if MenuItem::new(&SERIALIZE).build(ui) {
                        output = Some(ComponentSerializationCommandType::Serialize);
                    }

                    // Deserialize
                    if MenuItem::new(&DESERIALIZE).build(ui) {
                        output = Some(ComponentSerializationCommandType::StopSerializing);
                    }

                    // Revert
                    if MenuItem::new(&REVERT).build(ui) {
                        output = Some(ComponentSerializationCommandType::Revert);
                    }
                }
                serde_menu.end(ui);
            }
        }

        output
    }
}

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
