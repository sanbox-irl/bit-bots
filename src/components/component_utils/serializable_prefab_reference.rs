use super::{imgui_component_utils::InspectorParameters, imgui_system, PrefabId};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SerializablePrefabReference {
    pub target: Option<PrefabId>,
}

impl SerializablePrefabReference {
    pub fn inspect(&mut self, label: &str, ip: &InspectorParameters<'_, '_>) {
        if let Some(new_target) =
            imgui_system::select_prefab_entity(label, &self.target, ip.uid, ip.ui, ip.prefabs)
        {
            self.target = new_target;
        }
    }
}
