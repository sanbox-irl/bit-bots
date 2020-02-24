use super::{cardinals::CardinalPrime, ComponentBounds, InspectorParameters};

#[derive(
    Debug, SerializableComponent, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName,
)]
#[serde(default)]
pub struct Velocity {
    pub intended_direction: Option<CardinalPrime>,
}

impl ComponentBounds for Velocity {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_direction) = super::imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.intended_direction,
            "Intended Direction",
            ip.uid,
        ) {
            self.intended_direction = new_direction;
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .velocity
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.velocity = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.velocity = None;
    }
}
