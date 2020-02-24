use super::Vec2;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub window_size: Vec2,
    pub imgui_pixel_size: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_size: Vec2::new(1280.0, 720.0),
            imgui_pixel_size: 20.0,
        }
    }
}
