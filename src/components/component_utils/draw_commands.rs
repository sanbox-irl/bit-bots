use super::{
    Camera, ComponentList, DrawRectangle, Entity, RenderingUtility, ResourcesDatabase, Sprite, TextSource,
    Transform, Vec2,
};

#[derive(Default)]
pub struct DrawCommand<'a> {
    pub game_world: Option<GameWorldDrawCommands<'a>>,
    pub imgui: Option<ImGuiDrawCommands<'a>>,
}

pub struct GameWorldDrawCommands<'a> {
    pub text_sources: &'a ComponentList<TextSource>,
    pub sprites: &'a ComponentList<Sprite>,
    pub rects: &'a ComponentList<DrawRectangle>,
    // pub tilemaps: &'a ComponentList<Tilemap>,
    pub transforms: &'a ComponentList<Transform>,
    pub rendering_utility: &'a mut RenderingUtility,
    pub camera_entity: Option<&'a Entity>,
    pub camera: &'a Camera,
    pub resources: &'a ResourcesDatabase,
}

pub struct ImGuiDrawCommands<'a> {
    pub draw_data: &'a imgui::DrawData,
    pub imgui_dimensions: Vec2,
}
