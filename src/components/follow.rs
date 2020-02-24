use super::{
    component_utils::{Approach, SerializableEntityReference},
    imgui_system, ComponentBounds, InspectorParameters, Vec2,
};

#[derive(Debug, Clone,SerializableComponent, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Follow {
    pub approach: Approach,
    pub offset: Vec2,

    pub target: SerializableEntityReference,
}

impl ComponentBounds for Follow {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        // Entity!
        self.target.inspect("Target", &ip);

        // Approach Type
        if let Some(new_approach_type) = imgui_system::typed_enum_selection(ip.ui, &self.approach, ip.uid) {
            self.approach = new_approach_type;
        }

        match &mut self.approach {
            Approach::Instant => {}
            Approach::Linear(speed) => {
                ip.ui
                    .drag_float(&imgui::im_str!("Speed##{}", ip.uid), speed)
                    .build();
            }
            Approach::Asymptotic(weight) => {
                ip.ui
                    .drag_float(&imgui::im_str!("Weight##{}", ip.uid), weight)
                    .build();
            }
        }

        // Offset
        self.offset
            .inspector(ip.ui, &imgui::im_str!("Offset##{}", ip.uid));
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .follow
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.follow = Some({
            let mut clone: Follow = self.clone();
            clone.target.entity_id_to_serialized_refs(&serialization_markers);

            super::SerializedComponent { inner: clone, active }
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.follow = None;
    }

    fn post_deserialization(
        &mut self,
        _: super::Entity,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        self.target.serialized_refs_to_entity_id(serialization_markers);
    }
}
