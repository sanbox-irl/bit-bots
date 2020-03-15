use super::{systems::*, Ecs, HardwareInterface, ImGui, ImGuiDrawCommands, ResourcesDatabase, TimeKeeper};
use anyhow::Error;

pub struct Clockwork {
    pub ecs: Ecs,
    pub action_map: ActionMap,
    pub hardware_interfaces: HardwareInterface,
    pub resources: ResourcesDatabase,
    pub time_keeper: TimeKeeper,
    pub next_scene: Option<Scene>,
}

impl Clockwork {
    pub fn new() -> Result<Self, Error> {
        // Create Hardware Interfaces and Resources Handler
        let mut resources = ResourcesDatabase::new();
        let mut hardware_interfaces = HardwareInterface::new(&resources.config)?;
        resources.initialize(&mut hardware_interfaces.renderer)?;

        let ecs = Clockwork::start_scene(
            resources.config.default_scene,
            &mut resources,
            &mut hardware_interfaces,
        )?;

        Ok(Clockwork {
            ecs,
            hardware_interfaces,
            resources,
            action_map: ActionMap::default(),
            time_keeper: TimeKeeper::default(),
            next_scene: None,
        })
    }

    pub fn main_loop(&mut self) -> Result<(), Error> {
        // TICK STRUCTS
        let mut imgui = ImGui::new(
            &self.ecs.entity_allocator,
            &self.hardware_interfaces.window,
            &self.resources.config,
        );
        renderer_system::initialize_imgui(&mut self.hardware_interfaces.renderer, &mut imgui)?;

        loop {
            let scene_mode = self.ecs.scene_data.scene().mode();
            self.time_keeper.start_frame();

            // GET INPUT PER FRAME
            input_system::poll_events(
                &mut self.hardware_interfaces.input,
                &mut self.hardware_interfaces.window.events_loop,
                &self.hardware_interfaces.window.window,
                &mut imgui,
            );

            if self.hardware_interfaces.input.end_requested {
                break;
            }

            let mut ui_handler = imgui.begin_frame(
                &self.hardware_interfaces.window,
                self.hardware_interfaces
                    .input
                    .kb_input
                    .is_pressed(winit::event::VirtualKeyCode::S),
                self.time_keeper.delta_time,
            )?;

            imgui_system::imgui_main(
                &mut self.ecs,
                &mut self.resources,
                &mut self.hardware_interfaces,
                &mut ui_handler,
                &self.time_keeper,
                &mut self.next_scene,
            );

            if scene_mode == SceneMode::Draft {
                // tilemap_system::update_tilemaps_and_tilesets(
                //     &mut self.ecs.component_database.tilemaps,
                //     &mut self.ecs.component_database.transforms,
                //     &mut self.resources.tilesets,
                //     &mut self.resources.sprites,
                //     &self.hardware_interfaces.input,
                //     &self.ecs.singleton_database,
                // );
            }

            // Make the Action Map:
            self.action_map.update(&self.hardware_interfaces.input.kb_input);

            // Update
            while self.time_keeper.accumulator >= self.time_keeper.delta_time {
                if scene_mode == SceneMode::Playing {
                    self.ecs.update(&self.action_map)?;
                    self.ecs
                        .update_resources(&self.resources, self.time_keeper.delta_time);
                }
                self.time_keeper.accumulator -= self.time_keeper.delta_time;
            }

            // RENDER
            self.pre_render()?;
            self.render(ui_handler)?;

            // CHANGE SCENE?
            if self.check_scene_change()? {
                // Clear up the ImGui
                imgui.meta_data.entity_list_information.clear();
                imgui.meta_data.entity_vec.clear();
                imgui.meta_data.stored_ids.clear();
            }
        }

        imgui.save_meta_data()?;

        Ok(())
    }

    pub fn pre_render(&mut self) -> Result<(), Error> {
        renderer_system::pre_draw(
            &mut self.ecs.component_database,
            &mut self.resources,
            &mut self.hardware_interfaces.renderer,
        )?;

        Ok(())
    }

    pub fn render(&mut self, ui_handler: UiHandler<'_>) -> Result<(), Error> {
        // Update transform by walking the scene graph...
        scene_graph_system::update_transforms_via_scene_graph(
            &mut self.ecs.component_database.transforms,
            &self.ecs.scene_graph,
        );

        let mut draw_commands = DrawCommand::default();
        self.ecs.render(&mut draw_commands, &self.resources);
        draw_commands.imgui = Some(ImGuiDrawCommands {
            draw_data: ui_handler.ui.render(),
            imgui_dimensions: ui_handler
                .platform
                .scale_size_from_winit(
                    &self.hardware_interfaces.window.window,
                    self.hardware_interfaces
                        .window
                        .window
                        .inner_size()
                        .to_logical(self.hardware_interfaces.window.window.scale_factor()),
                )
                .into(),
        });

        renderer_system::render(
            &mut self.hardware_interfaces.renderer,
            &self.hardware_interfaces.window,
            &mut draw_commands,
        )?;

        Ok(())
    }

    fn check_scene_change(&mut self) -> Result<bool, Error> {
        Ok(if let Some(scene) = self.next_scene.take() {
            self.ecs = Clockwork::start_scene(scene, &mut self.resources, &mut self.hardware_interfaces)?;
            true
        } else {
            false
        })
    }

    fn start_scene(
        scene: Scene,
        resources: &mut ResourcesDatabase,
        hardware_interfaces: &mut HardwareInterface,
    ) -> Result<Ecs, Error> {
        info!("Loading {}...", scene);

        // Initialize the ECS and Scene Graph
        let mut ecs = Ecs::new(SceneData::new(scene)?, &resources.prefabs())?;
        ecs.game_start(resources, hardware_interfaces)?;

        info!("..Scene Loaded!");

        Ok(ecs)
    }
}
