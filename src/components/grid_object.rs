use super::{ComponentBounds, InspectorParameters, Vec2Int};

#[derive(Debug, Clone,SerializableComponent, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct GridObject {
    pub grid_type: GridType,
    #[serde(skip)]
    pub move_to_point_pos: Vec2Int,
    #[serde(skip)]
    pub move_to_point: bool,
    #[serde(skip)]
    pub register: bool,
}

impl GridObject {
    pub fn grid_type(&self) -> GridType {
        self.grid_type
    }
}

impl ComponentBounds for GridObject {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_grid_type) = super::imgui_system::typed_enum_selection(ip.ui, &self.grid_type, ip.uid)
        {
            self.grid_type = new_grid_type;
        }

        if self
            .move_to_point_pos
            .vec2int_inspector(ip.ui, &imgui::im_str!("##Move to Point{}", ip.uid))
        {
            if self.move_to_point_pos.x < 0 {
                self.move_to_point_pos.x = 0;
            }

            if self.move_to_point_pos.y < 0 {
                self.move_to_point_pos.y = 0;
            }
        }

        ip.ui.same_line(0.0);
        if ip
            .ui
            .button(&imgui::im_str!("Move to Point##{}", ip.uid), [0.0, 0.0])
        {
            self.move_to_point = true;
        }

        if ip
            .ui
            .button(&imgui::im_str!("Register Position##{}", ip.uid), [0.0, 0.0])
        {
            self.register = true;
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .grid_object
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.grid_object = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.grid_object = None;
    }
}

#[derive(
    Copy,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::EnumCount,
    typename::TypeName,
)]
pub enum GridType {
    Player,
    Pushable,
    Blockable,
    NonInteractable,
    Flag,
}

impl Default for GridType {
    fn default() -> GridType {
        GridType::Blockable
    }
}
