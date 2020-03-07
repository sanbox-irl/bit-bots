use super::{
    scene_graph::SerializedSceneGraph, serialization_util::entities::SerializedHashMap, PrefabId,
    SerializationId, SerializedEntity,
};
use std::collections::HashMap;

/// Where the Key in the HashMap is the same as the MainID in the Prefab.
pub type PrefabMap = HashMap<PrefabId, Prefab>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Prefab {
    root_id: PrefabId,
    valid: bool,
    pub members: SerializedHashMap,
    pub serialized_graph: SerializedSceneGraph,
}

impl Prefab {
    /// Creates a new Prefab with only a single member, which will
    /// also be the RootEntity
    pub fn new(root_entity: SerializedEntity) -> Prefab {
        let root_serialized_id = root_entity.id;
        let members = maplit::hashmap! {
            root_serialized_id => root_entity
        };

        let mut serialized_graph = SerializedSceneGraph::new();
        serialized_graph.instantiate_node(root_serialized_id);

        Prefab {
            root_id: PrefabId(root_serialized_id.inner()),
            members,
            serialized_graph,
            valid: true,
        }
    }

    pub fn new_blank() -> Prefab {
        let serialized_id = SerializationId::new();

        let members = maplit::hashmap! {
            serialized_id => SerializedEntity {
                id: serialized_id,
                ..Default::default()
            }
        };

        let mut serialized_graph = SerializedSceneGraph::new();
        serialized_graph.instantiate_node(serialized_id);

        Prefab {
            root_id: PrefabId(serialized_id.inner()),
            members,
            serialized_graph,
            valid: true,
        }
    }

    pub fn root_entity(&self) -> &SerializedEntity {
        &self.members[&self.root_serialization_id()]
    }

    pub fn root_entity_mut(&mut self) -> &mut SerializedEntity {
        self.members.get_mut(&self.root_serialization_id()).unwrap()
    }

    pub fn root_id(&self) -> PrefabId {
        self.root_id
    }

    pub fn root_serialization_id(&self) -> SerializationId {
        SerializationId(self.root_id.inner())
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn log_to_console(&self) {
        println!("---Console Log for {}---", self.root_id);
        println!("{:#?}", self);
        println!("------------------------");
    }
}
