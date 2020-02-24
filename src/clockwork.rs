use super::{
    systems::grid_system::Grid, systems::*, Ecs, HardwareInterface, ImGui, ImGuiDrawCommands,
    ResourcesDatabase, TimeKeeper,
};
use anyhow::Error;
use array2d::Array2D;

pub struct Clockwork {
    pub ecs: Ecs,
    pub action_map: ActionMap,
    pub grid: grid_system::Grid,
    pub hardware_interfaces: HardwareInterface,
    pub resources: ResourcesDatabase,
    pub time_keeper: TimeKeeper,
}

impl Clockwork {
    pub fn new() -> Result<Self, Error> {
        // Create Hardware Interfaces and Resources Handler
        let mut resources = ResourcesDatabase::new();
        let mut hardware_interfaces = HardwareInterface::new(&resources.config)?;
        resources.initialize(&mut hardware_interfaces.renderer)?;

        let (ecs, grid) = Clockwork::start_scene(&mut resources, &mut hardware_interfaces)?;

        Ok(Clockwork {
            ecs,
            hardware_interfaces,
            resources,
            action_map: ActionMap::default(),
            time_keeper: TimeKeeper::new(),
            grid,
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
            let scene_mode = scene_system::current_scene_mode();
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
                    self.ecs.update(&mut self.grid, &self.action_map)?;
                    self.ecs
                        .update_resources(&self.resources, self.time_keeper.delta_time);
                }
                self.time_keeper.accumulator -= self.time_keeper.delta_time;
            }

            // RENDER
            self.pre_render()?;
            self.render(ui_handler)?;

            // CHANGE SCENE?
            self.check_scene_change(&mut imgui)?;
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
        scene_graph::walk_graph(
            &mut self.ecs.component_database.transforms,
            &self.ecs.component_database.graph_nodes,
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

    fn check_scene_change(&mut self, imgui: &mut ImGui) -> Result<(), Error> {
        let should_change_scene = {
            let next_scene = scene_system::NEXT_SCENE.lock().unwrap();
            next_scene.is_some()
        };

        if should_change_scene {
            let (ecs, grid) = Clockwork::start_scene(&mut self.resources, &mut self.hardware_interfaces)?;
            self.ecs = ecs;
            self.grid = grid;

            // Clear up the ImGui
            imgui.meta_data.entity_list_information.clear();
            imgui.meta_data.entity_vec.clear();
            imgui.meta_data.stored_ids.clear();
        }

        Ok(())
    }

    fn start_scene(
        resources: &mut ResourcesDatabase,
        hardware_interfaces: &mut HardwareInterface,
    ) -> Result<(Ecs, Grid), Error> {
        // Change the Scene Name!
        {
            let next_scene = scene_system::NEXT_SCENE.lock().unwrap().take();
            let mut value = scene_system::CURRENT_SCENE.lock().unwrap();
            if let Some(next_scene) = next_scene {
                info!("Loading {}", next_scene);
                *value = next_scene;
            }
        }

        // Grid
        let mut grid = Array2D::filled_with(None, 5, 10);
        scene_graph::clear_root();

        // Initialize the ECS
        let mut ecs = Ecs::new(&resources.prefabs())?;
        ecs.game_start(resources, hardware_interfaces, &mut grid)?;

        info!("..Scene Loaded!");

        Ok((ecs, grid))
    }
}
