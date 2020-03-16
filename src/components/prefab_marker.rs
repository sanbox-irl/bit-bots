use super::{
    imgui_component_utils::{InspectorParameters, PrefabStatus},
    ComponentBounds, PrefabId, PrefabMap, SerializationId,
};

#[derive(
    Debug,
    ComponentPostDeserialization,
    SerializableComponent,
    Default,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    typename::TypeName,
)]
pub struct PrefabMarker {
    prefab_id: PrefabId,
    member_id: SerializationId,
}

impl PrefabMarker {
    pub fn new(prefab_id: PrefabId, member_id: SerializationId) -> Self {
        Self { prefab_id, member_id }
    }

    pub fn prefab_id(&self) -> PrefabId {
        self.prefab_id
    }

    pub fn member_id(&self) -> SerializationId {
        self.member_id
    }

    pub fn prefab_status(&self, prefab_map: &PrefabMap) -> PrefabStatus {
        prefab_map
            .get(&self.prefab_id)
            .map_or(PrefabStatus::None, |prefab| {
                if prefab.root_id() == self.member_id {
                    PrefabStatus::PrefabInstanceRoot
                } else {
                    PrefabStatus::PrefabInstanceSecondary
                }
            })
    }
}

impl ComponentBounds for PrefabMarker {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(serialized_name) = &ip.prefabs.get(&self.prefab_id).unwrap().root_entity().name {
            ip.ui
                .text(imgui::im_str!("Original Prefab: {}", serialized_name.inner.name));
        } else {
            ip.ui.text(imgui::im_str!("Original Prefab: {}", self.prefab_id));
        }
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .prefab_marker
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(&self, se: &mut super::SerializedEntity, active: bool, _: &super::TrackedEntitiesMap) {
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
pub struct PrefabLoadRequired;
