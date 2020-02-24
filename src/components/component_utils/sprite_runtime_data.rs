use super::{
    cardinals::{self, FacingHorizontal, FacingVertical},
    Color, DrawOrder, Vec2,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SpriteRunningData {
    pub draw_order: DrawOrder,
    pub facing_horizontal: FacingHorizontal,
    pub facing_vertical: FacingVertical,
    pub scale: Vec2,
    pub tint: Color,
    pub current_frame: usize,
    pub frame_time: f32,
    pub is_animating: bool,
}

impl Default for SpriteRunningData {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            facing_horizontal: FacingHorizontal::default(),
            facing_vertical: FacingVertical::default(),
            draw_order: DrawOrder::default(),
            tint: Color::default(),

            current_frame: usize::default(),
            frame_time: f32::default(),
            is_animating: bool::default(),
        }
    }
}

impl SpriteRunningData {
    pub fn inspect(&mut self, ui: &imgui::Ui<'_>, uid: &str) {
        cardinals::inspect_facing(ui, uid, &mut self.facing_horizontal, &mut self.facing_vertical);
        self.tint.inspect(ui, "Tint", uid);
        self.draw_order.inspect(ui, uid);
        self.scale.inspector(ui, &imgui::im_str!("Scale##{}", uid));

        // FRAME TIME
        ui.input_float(&imgui::im_str!("Frame Time##{}", uid), &mut self.frame_time)
            .build();

        ui.checkbox(&imgui::im_str!("Is Animating##{}", uid), &mut self.is_animating);
    }
}
