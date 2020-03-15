use super::SerializationId;
use std::collections::HashMap;

type PrefabMemberId = SerializationId;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PrefabChildMap(HashMap<PrefabMemberId, SerializationId>);
impl PrefabChildMap {
    pub fn new() -> PrefabChildMap {
        PrefabChildMap(HashMap::new())
    }

    pub fn get_serialization_id_for_member(&self, member_id: PrefabMemberId) -> Option<&SerializationId> {
        self.0.get(&member_id)
    }

    pub fn set_serializaiton_id_for_member(
        &mut self,
        member_id: PrefabMemberId,
        scene_serialization_id: SerializationId,
    ) {
        self.0.insert(member_id, scene_serialization_id);
    }
}
