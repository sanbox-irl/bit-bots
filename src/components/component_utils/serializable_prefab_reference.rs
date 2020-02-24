use super::{imgui_system, InspectorParameters};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SerializablePrefabReference {
    pub target: Option<uuid::Uuid>,
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
