use super::{imgui_component_utils::InspectorParameters, ComponentBounds, PrefabId, SerializationId};

#[derive(
    Debug,
    SerializableComponent,
    ComponentPostDeserialization,
    Default,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    typename::TypeName,
    Hash,
)]
pub struct PrefabMarker {
    main_id: PrefabId,
    sub_id: SerializationId,
}

impl PrefabMarker {
    pub fn new(main_id: PrefabId, sub_id: SerializationId) -> Self {
        Self { main_id, sub_id }
    }

    pub fn main_id(&self) -> PrefabId {
        self.main_id
    }

    pub fn sub_id(&self) -> SerializationId {
        self.sub_id
    }
}

impl ComponentBounds for PrefabMarker {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(serialized_name) = &ip.prefabs.get(&self.main_id).unwrap().root_entity().name {
            ip.ui
                .text(imgui::im_str!("Original Prefab: {}", serialized_name.inner.name));
        } else {
            ip.ui.text(imgui::im_str!("Original Prefab: {}", self.main_id));
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .prefab_marker
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.prefab_marker = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.prefab_marker = None;
    }
}

/// This is a marker, given out after loading a prefab into the prefab system,
/// which you must use to reload any prefab-inheritors in the current scene.
/// To do that, feed it into `prefab_system::update_prefab_inheritors();
#[must_use]
pub struct PrefabLoadRequired {
    pub prefab_id: PrefabId,
}
