use super::{imgui_system, utilities::math, Vec2, Vec2Int};
use strum_macros::{EnumIter, EnumString};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Origin {
    horizontal: HorizontalOrigin,
    vertical: VerticalOrigin,
}

impl Origin {
    pub fn new(horizontal: HorizontalOrigin, vertical: VerticalOrigin) -> Origin {
        Origin { horizontal, vertical }
    }

    pub fn gfx_adjustment(&self, dimensions: Vec2Int) -> Vec2 {
        self.update_origins_new_dimensions(dimensions)
    }

    pub fn sprite_location_relative(&self, dimensions: Vec2Int) -> Vec2 {
        let mut gfx_adjustment = Vec2::ZERO;

        gfx_adjustment.x = dimensions.x as f32
            * -match self.horizontal {
                HorizontalOrigin::Left => 0.0,
                HorizontalOrigin::Center => 0.5,
                HorizontalOrigin::Right => 1.0,
                HorizontalOrigin::Custom(x) => x,
            };

        gfx_adjustment.y = dimensions.y as f32
            * -match self.vertical {
                VerticalOrigin::Top => 0.0,
                VerticalOrigin::Center => 0.5,
                VerticalOrigin::Bottom => 1.0,
                VerticalOrigin::Custom(x) => x,
            };

        gfx_adjustment
    }
    // GETTERS
    pub fn horizontal(&self) -> HorizontalOrigin {
        self.horizontal
    }

    pub fn vertical(&self) -> VerticalOrigin {
        self.vertical
    }

    // SETTERS
    pub fn set_horizontal_origin(&mut self, horizontal: HorizontalOrigin, dimensions: Vec2Int) {
        self.horizontal = horizontal;
        self.set_adjustments(self.horizontal, self.vertical, dimensions);
    }

    pub fn set_vertical_origin(&mut self, vertical: VerticalOrigin, dimensions: Vec2Int) {
        self.vertical = vertical;
        self.set_adjustments(self.horizontal, self.vertical, dimensions);
    }

    // UPDATE DIMENSIONS
    pub fn update_origins_new_dimensions(&self, dimensions: Vec2Int) -> Vec2 {
        self.set_adjustments(self.horizontal, self.vertical, dimensions)
    }

    fn set_adjustments(
        &self,
        horizontal: HorizontalOrigin,
        vertical: VerticalOrigin,
        dimensions: Vec2Int,
    ) -> Vec2 {
        let x = dimensions.x as f32
            * match horizontal {
                HorizontalOrigin::Left => 0.0,
                HorizontalOrigin::Center => 0.5,
                HorizontalOrigin::Right => 1.0,
                HorizontalOrigin::Custom(x) => x,
            };

        let y = dimensions.y as f32
            * match vertical {
                VerticalOrigin::Top => 1.0,
                VerticalOrigin::Center => 0.5,
                VerticalOrigin::Bottom => 0.0,
                VerticalOrigin::Custom(x) => x,
            };

        Vec2::new(x, y)
    }

    pub fn inspect(&mut self, ui: &imgui::Ui<'_>, uid: &str, sprite_dimensions: Vec2Int) -> bool {
        ui.text("Origin");

        let mut dirty = false;

        // Horizontal
        if let Some(new_horizontal) = imgui_system::typed_enum_selection(ui, &self.horizontal, uid) {
            dirty = true;
            self.set_horizontal_origin(new_horizontal, sprite_dimensions);
        }

        if let HorizontalOrigin::Custom(c) = &mut self.horizontal {
            if ui
                .input_float(&imgui::im_str!("Custom Origin##{}", uid), c)
                .build()
            {
                dirty = true;
                math::clamped(c, 0.0, 1.0);
            }
        }

        // Vertical
        if let Some(new_vertical) = imgui_system::typed_enum_selection(ui, &self.vertical, uid) {
            dirty = true;

            self.set_vertical_origin(new_vertical, sprite_dimensions);
        }

        if let VerticalOrigin::Custom(c) = &mut self.vertical {
            if ui
                .input_float(&imgui::im_str!("Custom Origin##{}", uid), c)
                .build()
            {
                dirty = true;
                math::clamped(c, 0.0, 1.0);
            }
        }

        dirty
    }
}

impl Into<(HorizontalOrigin, VerticalOrigin)> for Origin {
    fn into(self) -> (HorizontalOrigin, VerticalOrigin) {
        (self.horizontal, self.vertical)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, EnumString, EnumIter, typename::TypeName)]
pub enum HorizontalOrigin {
    Left,
    Center,
    Right,
    Custom(f32),
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, EnumIter, typename::TypeName)]
pub enum VerticalOrigin {
    Top,
    Center,
    Bottom,
    Custom(f32),
}

impl Default for HorizontalOrigin {
    fn default() -> Self {
        Self::Center
    }
}

impl Default for VerticalOrigin {
    fn default() -> Self {
        Self::Center
    }
}
