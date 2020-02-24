use strum_macros::EnumIter;

#[derive(
    Debug,
    strum_macros::Display,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
pub enum CardinalPrime {
    Right,
    Up,
    Left,
    Down,
}

#[derive(
    Debug,
    strum_macros::Display,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
pub enum FacingHorizontal {
    Right,
    Left,
}

#[derive(
    Debug,
    strum_macros::Display,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
pub enum FacingVertical {
    Up,
    Down,
}

impl Default for CardinalPrime {
    fn default() -> Self {
        Self::Right
    }
}

impl Default for FacingHorizontal {
    fn default() -> Self {
        Self::Right
    }
}

impl Default for FacingVertical {
    fn default() -> Self {
        Self::Up
    }
}

use super::imgui_system;
pub fn inspect_facing(
    ui: &imgui::Ui<'_>,
    uid: &str,
    facing_horizontal: &mut FacingHorizontal,
    facing_vertical: &mut FacingVertical,
) -> bool {
    let mut dirty = false;

    if let Some(new_horizontal) = imgui_system::typed_enum_selection(ui, facing_horizontal, uid) {
        dirty = true;
        *facing_horizontal = new_horizontal;
    }

    if let Some(new_vertical) = imgui_system::typed_enum_selection(ui, facing_vertical, uid) {
        dirty = true;
        *facing_vertical = new_vertical;
    }

    dirty
}
