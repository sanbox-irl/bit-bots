use super::{Component, ComponentList, Entity, GraphNode, RawComponent, SerializationMarker};
use std::ptr;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct TransformParent {
    #[serde(skip)]
    pub target: RawComponent,
    target_serialized_id: Option<uuid::Uuid>,
}

impl Default for TransformParent {
    fn default() -> Self {
        Self {
            target: RawComponent {
                entity: None,
                graph_node: ptr::null_mut(),
            },
            target_serialized_id: None,
        }
    }
}

impl PartialEq for TransformParent {
    fn eq(&self, other: &TransformParent) -> bool {
        if self.is_root() {
            other.is_root()
        } else {
            self.target.graph_node == other.target.graph_node
        }
    }
}

// impl Clone for TransformParent {
//     fn clone(&self) -> Self {

//         let raw_component: RawComponent = RawComponent {
//             entity
//         }
//         unimplemented!()
//     }
// }

impl TransformParent {
    pub fn new(
        graph_node: &mut Component<GraphNode>,
        serialized_data: &Option<&SerializationMarker>,
    ) -> Self {
        Self {
            target: RawComponent::new(graph_node),
            target_serialized_id: serialized_data.map(|sd| sd.id.clone()),
        }
    }

    pub fn blank() -> Self {
        TransformParent {
            target: RawComponent {
                entity: None,
                graph_node: ptr::null_mut(),
            },
            target_serialized_id: None,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent_id().is_none()
    }

    pub fn parent_mut(&mut self) -> Option<&mut GraphNode> {
        if self.target.graph_node == ptr::null_mut() {
            None
        } else {
            unsafe { Some(&mut *self.target.graph_node) }
        }
    }

    pub fn parent_id(&self) -> Option<Entity> {
        self.target.entity
    }

    pub fn serialize(&mut self, serialized_list: &ComponentList<SerializationMarker>) {
        if let Some(target_entity_id) = &self.target.entity {
            if let Some(sd) = serialized_list.get(target_entity_id) {
                self.target_serialized_id = Some(sd.inner().id.clone());
            } else {
                error!("Reference to {:?} is being serialized, but it is not serialized. We will outlive it and follow nothing on deserialization!", target_entity_id);
            }
        }
    }

    pub fn deserialize(
        &mut self,
        graph_nodes: &mut ComponentList<GraphNode>,
        serialized_data: &ComponentList<SerializationMarker>,
    ) {
        if let Some(tsi) = &self.target_serialized_id {
            let entity_id: Option<Entity> = serialized_data
                .iter()
                .find(|sd| &sd.inner().id == tsi)
                .map(|i| i.entity_id());

            if let Some(entity_id) = entity_id {
                if let Some(target_transform_component) = graph_nodes.get_mut(&entity_id) {
                    self.target.graph_node = &mut *target_transform_component.inner_mut();
                } else {
                    error!("We didn't find a target Transform Parent on a serialized transform parent.");
                    self.target.graph_node = ptr::null_mut();
                    self.target_serialized_id = None;
                }
            } else {
                error!(
                        "We didn't find a target on a serialized entity reference. Did its target have a SerializedData?"
                    );
                self.target.graph_node = ptr::null_mut();
                self.target_serialized_id = None;
            }
        }
    }
}
