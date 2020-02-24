use super::imgui_system;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use strum_macros::{EnumCount, EnumIter, EnumString};

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, typename::TypeName)]
pub struct DrawOrder {
    pub draw_layer: DrawLayer,
    pub order: isize,
}

impl DrawOrder {
    pub fn new(draw_layer: DrawLayer, order: isize) -> Self {
        Self { draw_layer, order }
    }

    pub fn inspect(&mut self, ui: &imgui::Ui<'_>, uid: &str) {
        if let Some(new_draw_layer) = imgui_system::typed_enum_selection(ui, &self.draw_layer, uid) {
            self.draw_layer = new_draw_layer;
        }

        imgui_system::input_isize(ui, "Order", uid, &mut self.order);
    }

    pub fn to_f32(&self) -> f32 {
        let layer: f32 = ToPrimitive::to_i32(&self.draw_layer).unwrap() as f32;
        let draw_order = self.order as f32 / 10.0;

        layer + draw_order
    }
}

use std::cmp::Ordering;
impl PartialOrd for DrawOrder {
    fn partial_cmp(&self, other: &DrawOrder) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawOrder {
    fn cmp(&self, other: &DrawOrder) -> Ordering {
        self.draw_layer
            .cmp(&other.draw_layer)
            .then(self.order.cmp(&other.order))
    }
}

impl From<f32> for DrawOrder {
    fn from(o: f32) -> DrawOrder {
        let flat_number = o as i32;
        let remainder = ((o - (flat_number as f32)) * 10.0) as isize;

        DrawOrder {
            draw_layer: FromPrimitive::from_i32(flat_number).unwrap(),
            order: remainder,
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    EnumString,
    EnumCount,
    Serialize,
    Deserialize,
    typename::TypeName,
    FromPrimitive,
    ToPrimitive,
)]
#[strum(serialize_all = "snake_case")]
pub enum DrawLayer {
    Background = 0,
    Instance = 1,
    Foreground = 2,
}

impl Default for DrawLayer {
    fn default() -> Self {
        DrawLayer::Instance
    }
}
