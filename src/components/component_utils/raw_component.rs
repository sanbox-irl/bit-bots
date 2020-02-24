use super::{Component, Entity, GraphNode};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct RawComponent {
    pub graph_node: *mut GraphNode,
    pub entity: Option<Entity>,
}

impl RawComponent {
    pub fn new(graph_node: &mut Component<GraphNode>) -> Self {
        Self {
            graph_node: graph_node.inner_mut(),
            entity: Some(graph_node.entity_id()),
        }
    }

    pub fn is_real(&self) -> bool {
        self.graph_node != std::ptr::null_mut() && self.entity.is_some()
    }
}
