use super::{
    scene_graph::SerializedSceneGraph, serialization_util::entities::SerializedHashMap, PrefabId,
    SerializationId, SerializedEntity,
};
use std::collections::HashMap;

/// Where the Key in the HashMap is the same as the MainID in the Prefab.
pub type PrefabMap = HashMap<PrefabId, Prefab>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Prefab {
    prefab_id: PrefabId,
    valid: bool,
    pub members: SerializedHashMap,
    pub serialized_graph: SerializedSceneGraph,
}

impl Prefab {
    /// Creates a new Prefab with only a single member, which will
    /// also be the RootEntity. The PrefabId can be supplied (to write over
    /// another prefab) or it can just be given a randomly made one.
    pub fn new(root_entity: SerializedEntity, prefab_id: PrefabId) -> Prefab {
        let root_serialized_id = root_entity.id;
        let members = maplit::hashmap! {
            root_serialized_id => root_entity
        };

        let mut serialized_graph = SerializedSceneGraph::new();
        serialized_graph.instantiate_node(root_serialized_id);

        Prefab {
            prefab_id,
            members,
            serialized_graph,
            valid: true,
        }
    }

    pub fn new_blank() -> Prefab {
        let serialized_id = SerializationId::new();

        let members = maplit::hashmap! {
            serialized_id => SerializedEntity::with_serialization_id(serialized_id)
        };

        let mut serialized_graph = SerializedSceneGraph::new();
        serialized_graph.instantiate_node(serialized_id);

        Prefab {
            prefab_id: PrefabId::new(),
            members,
            serialized_graph,
            valid: true,
        }
    }

    pub fn prefab_id(&self) -> &PrefabId {
        &self.prefab_id
    }

    pub fn root_id(&self) -> &SerializationId {
        self.serialized_graph.iter_roots().nth(0).unwrap().inner()
    }

    pub fn root_entity(&self) -> &SerializedEntity {
        println!("Members are {:#?}", self.members);
        &self.members.get(self.root_id()).unwrap()
    }

    pub fn root_entity_mut(&mut self) -> &mut SerializedEntity {
        let root_id = *self.root_id();
        self.members.get_mut(&root_id).unwrap()
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn log_to_console(&self) {
        println!("---Console Log for {}---", self.prefab_id);
        println!("{:#?}", self);
        println!("------------------------");
    }
}
