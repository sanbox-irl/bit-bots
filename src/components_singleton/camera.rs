use super::{imgui_system, HardwareInterface, InspectorParameters, SingletonBounds, Vec2};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Camera {
    pub native_resolution: Vec2,
    pub zoom_level: f32,
    pub current_mode: CameraMode,
    #[serde(skip)]
    display_size: Option<Vec2>,
    pub default_position: Vec2,
}

impl Camera {
    pub fn initialize_with_hwi(&mut self, hwi: &HardwareInterface) {
        self.display_size = Some(hwi.window.get_window_size());
    }

    pub fn ingame_camera_size(&self) -> Vec2 {
        self.native_resolution / self.zoom_level
    }

    pub fn display_size(&self) -> Vec2 {
        self.display_size.unwrap()
    }

    pub fn display_to_world_position(&self, display_pos: Vec2, camera_position: Vec2) -> Vec2 {
        // This is all kinda @Hacky but it works!
        let transformed_to_world = {
            let window_size = self
                .display_size
                .expect("The Window size was not set for the camera!");

            let refactored_display_pos =
                Vec2::new(display_pos.x, -display_pos.y + window_size.y) - Vec2::new(720.0, 120.0);

            let percentage_of_screen = refactored_display_pos.cwise_div(Vec2::new(480.0, 840.0));
            info!("Percentage of screen: {}", percentage_of_screen);
            info!("--");

            let clip_space = Vec2::new(
                percentage_of_screen.x * 2.0 - 1.0,
                percentage_of_screen.y * 2.0 - 1.0,
            );

            let in_game_size = self.ingame_camera_size() / 2.0;
            let mut ret = clip_space;

            ret.x *= in_game_size.x;
            ret.y *= in_game_size.y;

            ret
        };

        transformed_to_world + camera_position
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            native_resolution: Vec2::new(1280.0, 720.0),
            zoom_level: 1.0,
            current_mode: CameraMode::Standard,
            display_size: None,
            default_position: Vec2::ZERO,
        }
    }
}

impl SingletonBounds for Camera {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        self.native_resolution
            .inspector(ui, &imgui::im_str!("Native Resolution##{}", uid));

        let drag_float = ui
            .drag_float(&imgui::im_str!("Zoom##{}", uid), &mut self.zoom_level)
            .build();
        if drag_float {
            self.zoom_level = f32::max(0.0, self.zoom_level);
        }

        if let Some(new_mode) = imgui_system::typed_enum_selection(ui, &self.current_mode, uid) {
            self.current_mode = new_mode;
        }

        self.default_position
            .inspector(ui, imgui::im_str!("Default Position"));

        imgui_system::help_marker(
            ui,
            "This is only used when we don't have an associated entity. Do not rely on it!",
        );
    }
}

use strum_macros::{EnumIter, EnumString};
#[derive(Debug, Serialize, Clone, Deserialize, typename::TypeName, EnumIter, EnumString)]
pub enum CameraMode {
    Standard,
    Debug,
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Standard
    }
}
