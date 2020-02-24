use super::{ComponentBounds, InspectorParameters};

#[derive(Debug,SerializableComponent, Default, Clone, Serialize, Deserialize, PartialEq, Eq, typename::TypeName)]
pub struct SceneSwitcher {
    pub target_scene: String,
}

impl ComponentBounds for SceneSwitcher {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        let mut scene_name = imgui::ImString::new(&self.target_scene);

        if ip
            .ui
            .input_text(&imgui::im_str!("Scene##{}", ip.uid), &mut scene_name)
            .resize_buffer(true)
            .build()
        {
            self.target_scene = scene_name.to_string();
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .scene_switcher
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.scene_switcher = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.scene_switcher = None;
    }
}
