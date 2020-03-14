use super::{
    imgui_component_utils::InspectorParameters, imgui_system, Entity, EntitySerializationMap, SerializationId,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SerializableEntityReference {
    #[serde(skip)]
    pub target: Option<Entity>,
    target_serialized_id: Option<SerializationId>,
}

impl SerializableEntityReference {
    pub fn new(entity_id: Entity) -> Self {
        Self {
            target: Some(entity_id),
            target_serialized_id: None,
        }
    }

    pub fn target_serialized_id(&self) -> Option<SerializationId> {
        self.target_serialized_id
    }

    pub fn entity_id_to_serialized_refs(&mut self, serialization_map: &EntitySerializationMap) {
        if let Some(target) = &self.target {
            self.target_serialized_id = serialization_map.get(target).cloned();
        }
    }

    pub fn serialized_refs_to_entity_id(&mut self, serialization_map: &EntitySerializationMap) {
        if let Some(tsi) = &self.target_serialized_id {
            self.target = serialization_map.iter().find_map(|(entity, serialization_id)| {
                if serialization_id == tsi {
                    Some(*entity)
                } else {
                    None
                }
            });

            if self.target.is_none() {
                error!(
                    "We didn't find a target on a serialized entity reference. Did its target have a SerializedData?"
                );
                self.target_serialized_id = None;
            }
        }
    }
}

impl SerializableEntityReference {
    pub fn inspect(&mut self, label: &str, ip: &InspectorParameters<'_, '_>) {
        if let Some(new_target) = imgui_system::select_entity_option(
            label,
            &self.target,
            ip.uid,
            ip.ui,
            ip.entities,
            ip.entity_names,
        ) {
            self.target = new_target;
        }
    }
}
