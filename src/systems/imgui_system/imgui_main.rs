use super::{
    imgui_component_utils::{CreateEntityCommand, CreateEntityCommandType},
    *,
};

pub fn imgui_main(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    hardware_interfaces: &mut HardwareInterface,
    ui_handler: &mut UiHandler<'_>,
    time_keeper: &TimeKeeper,
    next_scene: &mut Option<Scene>,
) {
    let mut entity_serialization_command = None;

    *next_scene = main_menu_bar(
        hardware_interfaces
            .input
            .kb_input
            .is_pressed(winit::event::VirtualKeyCode::F1),
        ui_handler,
        ecs.scene_data.scene(),
    );

    // Scene Entity Inspector
    if ui_handler.flags.contains(ImGuiFlags::ENTITY_VIEWER) {
        match imgui_entity::entity_list(ecs, resources, ui_handler, next_scene) {
            Ok(sc) => {
                if let Some(sc) = sc {
                    entity_serialization_command = Some(sc)
                }
            }
            Err(e) => {
                error!("Error processing a NameResult Request! {}", e);
            }
        }
    }

    // Window for Each Entity
    match imgui_component::entity_inspector(ecs, resources, ui_handler) {
        Ok(sc) => {
            if let Some(sc) = sc {
                entity_serialization_command = Some(sc)
            }
        }
        Err(e) => {
            error!("Error processing a NameResult Request! {}", e);
        }
    }

    // Singleton
    imgui_utility::create_window(ui_handler, ImGuiFlags::SINGLETONS, |ui_handler| {
        imgui_singleton::singleton_inspector(
            &mut ecs.singleton_database,
            &ecs.component_database.names,
            &ecs.entities,
            resources.prefabs(),
            ui_handler,
        )
    });

    // Time Keeper
    imgui_utility::create_window(ui_handler, ImGuiFlags::TIME_KEEPER, |ui_handler| {
        time_keeper.create_imgui_window(ui_handler)
    });

    // Resources Windows
    imgui_resources::create_resources_windows(
        resources,
        ui_handler,
        ecs.scene_data.scene().mode(),
        next_scene,
    );

    // Demo window!
    if ui_handler.flags.contains(ImGuiFlags::IMGUI_EXAMPLE) {
        let mut is_closed = false;
        ui_handler.ui.show_demo_window(&mut is_closed);
        if is_closed {
            ui_handler.flags.remove(ImGuiFlags::IMGUI_EXAMPLE);
        }
    }

    // Logger
    imgui_utility::create_window(ui_handler, ImGuiFlags::LOGGER, |ui_handler| {
        imgui_logging_tool(ui_handler, ecs)
    });

    if let Some(sc) = entity_serialization_command {
        if let Err(e) = ecs.process_serialized_command(sc, resources) {
            error!("Error Processing Serialized Command: {}", e);
        }
    }

    // Always last here...
    if ui_handler.ui.io().want_capture_mouse {
        hardware_interfaces.input.mouse_input.clear();
        hardware_interfaces.input.mouse_input.clear_held();
    }
    if ui_handler.ui.io().want_capture_keyboard {
        hardware_interfaces.input.kb_input.clear();
        hardware_interfaces.input.kb_input.held_keys.clear();
    }
}

fn main_menu_bar(toggle_main_menu_bar: bool, ui_handler: &mut UiHandler<'_>, scene: &Scene) -> Option<Scene> {
    if toggle_main_menu_bar {
        ui_handler.flags.toggle(ImGuiFlags::MAIN_MENU_BAR);
    }

    let mut ret = None;

    if ui_handler.flags.contains(ImGuiFlags::MAIN_MENU_BAR) {
        // MENU
        let ui = &ui_handler.ui;
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            // SCENE

            if let Some(menu) = ui.begin_menu(&im_str!("{}", scene), true) {
                scene_change(
                    "Switch Scene",
                    ui,
                    &mut ui_handler.scene_changing_info.switch_scene_name,
                    |new_name| {
                        let new_scene = Scene::new(new_name.to_string());
                        if scene_system::scene_exists(&new_scene) {
                            ret = Some(new_scene);
                        } else {
                            error!("Couldn't switch to Scene {}", new_name);
                            error!("Does a Scene by that name exist?");
                        }
                    },
                );

                scene_change(
                    "Create Scene",
                    ui,
                    &mut ui_handler.scene_changing_info.create_scene,
                    |new_name| match scene_system::create_scene(new_name) {
                        Ok(made_scene) => {
                            if made_scene == false {
                                error!("Couldn't create Scene {}", new_name);
                                error!("Does another scene already exist with that name?");
                            }
                        }
                        Err(e) => {
                            error!("Couldn't create Scene {}", new_name);
                            error!("E: {}", e);
                        }
                    },
                );

                scene_change(
                    "Delete Scene",
                    ui,
                    &mut ui_handler.scene_changing_info.delete_scene_name,
                    |new_name| match scene_system::delete_scene(&new_name) {
                        Ok(deleted_scene) => {
                            if deleted_scene == false {
                                error!("Couldn't delete Scene {}", new_name);
                                error!("Does a Scene with that name exist?");
                            }
                        }
                        Err(e) => {
                            error!("Couldn't delete Scene {}", new_name);
                            error!("E: {}", e);
                        }
                    },
                );

                menu.end(ui);
            }

            // INSPECTORS
            if let Some(menu) = ui.begin_menu(im_str!("Inspectors"), true) {
                menu_option(
                    im_str!("Component Inspector"),
                    ImGuiFlags::ENTITY_VIEWER,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Singleton Inspector"),
                    ImGuiFlags::SINGLETONS,
                    ui,
                    &mut ui_handler.flags,
                );

                menu.end(ui);
            }

            // PANELS
            if let Some(other_windows) = ui.begin_menu(im_str!("Assets"), true) {
                menu_option(
                    im_str!("Sprite Inspector"),
                    ImGuiFlags::SPRITE_RESOURCE,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Tile Set Inspector"),
                    ImGuiFlags::TILEMAP_RESOURCE,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Prefab Inspector"),
                    ImGuiFlags::PREFAB_INSPECTOR,
                    ui,
                    &mut ui_handler.flags,
                );

                other_windows.end(ui);
            }

            // UTILITIES
            if let Some(utility_bar) = ui.begin_menu(im_str!("Utilities"), true) {
                menu_option(
                    im_str!("Time Keeper"),
                    ImGuiFlags::TIME_KEEPER,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Game Config Inspector"),
                    ImGuiFlags::GAME_CONFIG,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Demo Window"),
                    ImGuiFlags::IMGUI_EXAMPLE,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(im_str!("Logger"), ImGuiFlags::LOGGER, ui, &mut ui_handler.flags);

                utility_bar.end(ui);
            }

            menu_bar.end(ui);
        }
    }

    ret
}

fn menu_option(imstr: &imgui::ImStr, flag: ImGuiFlags, ui: &Ui<'_>, flags_to_change: &mut ImGuiFlags) {
    if imgui::MenuItem::new(imstr)
        .selected(flags_to_change.contains(flag))
        .build(ui)
    {
        flags_to_change.toggle(flag);
    }
}

fn scene_change<F: FnMut(&str)>(prompt: &str, ui: &imgui::Ui<'_>, scene_name: &mut String, mut on_click: F) {
    let im_prompt = imgui::ImString::new(prompt);

    if let Some(scene_submenu) = ui.begin_menu(&im_prompt, true) {
        let mut im_scene_name = imgui::im_str!("{}", scene_name);
        if ui
            .input_text(&im_str!("##NoLabel{}", im_prompt), &mut im_scene_name)
            .resize_buffer(true)
            .build()
        {
            *scene_name = im_scene_name.to_string();
        }

        ui.same_line(0.0);
        if ui.button(&im_prompt, [0.0, 0.0]) {
            on_click(scene_name);
        }

        scene_submenu.end(ui);
    }
}

fn imgui_logging_tool(ui_handler: &mut UiHandler<'_>, ecs: &mut Ecs) -> bool {
    let mut is_opened = true;

    let ui = &mut ui_handler.ui;
    let time_keeper_window = imgui::Window::new(im_str!("Logging Tool"))
        .size(Vec2::new(400.0, 100.0).into(), imgui::Condition::FirstUseEver)
        .opened(&mut is_opened);

    if let Some(window) = time_keeper_window.begin(ui) {
        if ui.button(im_str!("Log ComponentDatabase"), [0.0, 0.0]) {
            ecs.log_component_database();
        }
        if ui.button(im_str!("Log SingletonDatabase"), [0.0, 0.0]) {
            println!("{:#?}", ecs.singleton_database);
        }
        if ui.button(im_str!("Log SceneGraph"), [0.0, 0.0]) {
            ecs.scene_graph.pretty_print(&ecs.component_database.names);
        }
        window.end(ui);
    }

    is_opened
}

pub(super) fn process_entity_subcommand(
    create_entity: CreateEntityCommand,
    ecs: &mut Ecs,
    resources: &ResourcesDatabase,
) {
    let entity = match create_entity.command_type {
        CreateEntityCommandType::CreateBlank => ecs.create_entity(),
        CreateEntityCommandType::CreatePrefab(prefab_id) => {
            prefab_system::instantiate_entity_from_prefab(ecs, prefab_id, resources.prefabs())
        }
    };

    if let Some(parent) = create_entity.parent_id {
        let my_transform_c = ecs
            .component_database
            .transforms
            .set_component_default(&entity, &mut ecs.scene_graph);

        if let Some(node_id) = my_transform_c.inner().scene_graph_node_id() {
            parent.append(node_id, &mut ecs.scene_graph);
        }
    }

    // We don't actually need this check, but let's not waste our time making the SE if
    // we don't be able to use it, ya know?
    if ecs.scene_data.scene().mode() == SceneMode::Draft {
        if let Some(serialized_entity) = SerializedEntity::new(
            &entity,
            SerializationId::new(),
            &ecs.component_database,
            &ecs.singleton_database,
            &ecs.scene_data,
            resources,
        ) {
            ecs.scene_data.serialize_entity(entity, serialized_entity);
        }
    }
}
