use super::{imgui_system, sprite_resources::SpriteName, ComponentBounds, InspectorParameters};

#[derive(Debug,SerializableComponent, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Player {
    pub standing_sprite: Option<SpriteName>,
    #[serde(skip)]
    pub active: bool,
}

impl ComponentBounds for Player {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_sprite) = imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.standing_sprite,
            "Standing Sprite",
            ip.uid,
        ) {
            self.standing_sprite = new_sprite;
        };

        ip.ui
            .checkbox(&imgui::im_str!("Active##{}", ip.uid), &mut self.active);
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .player
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.player = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.player = None;
    }
}
